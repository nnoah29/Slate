/*
**  _                                              _      ___    ___
** | |                                            | |    |__ \  / _ \
** | |_Created _       _ __   _ __    ___    __ _ | |__     ) || (_) |
** | '_ \ | | | |     | '_ \ | '_ \  / _ \  / _` || '_ \   / /  \__, |
** | |_) || |_| |     | | | || | | || (_) || (_| || | | | / /_    / /
** |_.__/  \__, |     |_| |_||_| |_| \___/  \__,_||_| |_||____|  /_/
**          __/ |     on 2026-11-22.
**         |___/
**
**
** Tests du téléchargement vérifié et de l'extraction sûre, contre un serveur local 127.0.0.1 (T4).
*/

use bzip2::Compression;
use bzip2::write::BzEncoder;
use sha2::{Digest, Sha256};
use slate::voice::install_error::ModelInstallError;
use slate::voice::model_catalog::ModelEntry;
use slate::voice::model_download::install_model;
use slate::voice::model_store::ModelStore;
use slate::voice::types::VoiceLanguage;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

const FILES: [&str; 4] = ["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"];

fn append_file(builder: &mut tar::Builder<BzEncoder<Vec<u8>>>, path: &str, data: &[u8]) {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    let bytes = path.as_bytes();
    header.as_old_mut().name[..bytes.len()].copy_from_slice(bytes);
    header.set_cksum();
    builder.append(&header, data).expect("append");
}

fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let encoder = BzEncoder::new(Vec::new(), Compression::fast());
    let mut builder = tar::Builder::new(encoder);
    for (path, data) in entries {
        append_file(&mut builder, path, data);
    }
    builder.into_inner().expect("tar").finish().expect("bz2")
}

fn good_archive() -> Vec<u8> {
    let items: Vec<(String, &[u8])> = FILES
        .iter()
        .map(|f| (format!("fake-model/{f}"), b"data".as_slice()))
        .collect();
    let refs: Vec<(&str, &[u8])> = items.iter().map(|(p, d)| (p.as_str(), *d)).collect();
    archive(&refs)
}

fn serve(body: Vec<u8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let url = format!(
        "http://{}/model.tar.bz2",
        listener.local_addr().expect("addr")
    );
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0u8; 2048];
            let _ = stream.read(&mut request);
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
    });
    url
}

fn entry_for_body(body: &[u8], url: String, sha_override: Option<&str>) -> ModelEntry {
    let digest: String = Sha256::digest(body)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let sha: &'static str = Box::leak(sha_override.map_or(digest, str::to_string).into_boxed_str());
    ModelEntry {
        language: VoiceLanguage::Fr,
        url: Box::leak(url.into_boxed_str()),
        archive_bytes: body.len() as u64,
        sha256: sha,
        folder: "fake-model",
        files: FILES,
    }
}

#[test]
fn test_valid_archive_is_installed_with_progress() {
    let body = good_archive();
    let entry = entry_for_body(&body, serve(body.clone()), None);
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().to_path_buf());
    let mut last = (0, 0);
    install_model(&store, &entry, &mut |done, total| last = (done, total)).expect("install");
    assert_eq!(last, (body.len() as u64, body.len() as u64));
    for file in FILES {
        assert!(tmp.path().join("fake-model").join(file).is_file(), "{file}");
    }
    assert!(!tmp.path().join(".fake-model.part").exists());
}

#[test]
fn test_wrong_checksum_is_refused_and_leaves_nothing() {
    let body = good_archive();
    let wrong = "0".repeat(64);
    let entry = entry_for_body(&body, serve(body.clone()), Some(&wrong));
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().to_path_buf());
    let err = install_model(&store, &entry, &mut |_, _| {}).expect_err("refused");
    assert_eq!(err, ModelInstallError::BadChecksum);
    assert_eq!(std::fs::read_dir(tmp.path()).expect("dir").count(), 0);
}

#[test]
fn test_path_traversal_in_archive_is_refused() {
    let body = archive(&[("../evil.txt", b"x"), ("fake-model/encoder.onnx", b"x")]);
    let entry = entry_for_body(&body, serve(body.clone()), None);
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().join("models"));
    let err = install_model(&store, &entry, &mut |_, _| {}).expect_err("refused");
    assert!(matches!(err, ModelInstallError::UnsafeArchive(_)));
    assert!(!tmp.path().join("evil.txt").exists());
    assert!(!store.is_installed(VoiceLanguage::Fr));
}

#[test]
fn test_incomplete_archive_is_refused() {
    let body = archive(&[("fake-model/encoder.onnx", b"x")]);
    let entry = entry_for_body(&body, serve(body.clone()), None);
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().to_path_buf());
    let err = install_model(&store, &entry, &mut |_, _| {}).expect_err("refused");
    assert!(matches!(err, ModelInstallError::Incomplete(_)));
    assert!(!tmp.path().join("fake-model").exists());
}

#[test]
fn test_unreachable_server_gives_network_error() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let url = format!("http://{}/x", listener.local_addr().expect("addr"));
    drop(listener);
    let entry = entry_for_body(b"abc", url, None);
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().to_path_buf());
    let err = install_model(&store, &entry, &mut |_, _| {}).expect_err("network");
    assert!(matches!(err, ModelInstallError::Network(_)));
}
