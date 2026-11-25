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
** Téléchargement HTTP d'un modèle avec progression, puis vérification de la taille et du SHA-256.
*/

use super::install_error::ModelInstallError;
use super::model_archive::extract_model;
use super::model_catalog::ModelEntry;
use super::model_store::ModelStore;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

const CHUNK: usize = 64 * 1024;

/// Écrit le flux dans `dest` en calculant son empreinte ; renvoie (octets reçus, empreinte hexadécimale).
fn stream_to_file(
    mut reader: impl Read,
    dest: &Path,
    total: u64,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<(u64, String), ModelInstallError> {
    let mut file = File::create(dest)?;
    let mut hasher = Sha256::new();
    let (mut buffer, mut received) = (vec![0u8; CHUNK], 0u64);
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        received += read as u64;
        progress(received, total);
    }
    let digest: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok((received, digest))
}

/// Télécharge l'archive dans `dest` ; en cas d'erreur ou d'intégrité fausse, ne laisse aucun fichier.
fn download_archive(
    entry: &ModelEntry,
    dest: &Path,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<(), ModelInstallError> {
    let response = ureq::get(entry.url)
        .call()
        .map_err(|e| ModelInstallError::Network(e.to_string()))?;
    let reader = response.into_body().into_reader();
    let result = stream_to_file(reader, dest, entry.archive_bytes, progress);
    let checked = result.and_then(|(got, digest)| check_integrity(entry, got, &digest));
    if checked.is_err() {
        let _ = fs::remove_file(dest);
    }
    checked
}

/// Compare taille et empreinte reçues à celles du catalogue.
fn check_integrity(entry: &ModelEntry, got: u64, digest: &str) -> Result<(), ModelInstallError> {
    if got != entry.archive_bytes {
        return Err(ModelInstallError::BadSize {
            expected: entry.archive_bytes,
            got,
        });
    }
    if !digest.eq_ignore_ascii_case(entry.sha256) {
        return Err(ModelInstallError::BadChecksum);
    }
    Ok(())
}

/// Télécharge, vérifie puis extrait le modèle ; en cas d'échec, rien ne reste dans le dossier des modèles.
pub fn install_model(
    store: &ModelStore,
    entry: &ModelEntry,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<(), ModelInstallError> {
    fs::create_dir_all(store.root())?;
    let part = store.root().join(format!(".{}.part", entry.folder));
    download_archive(entry, &part, progress)?;
    let result = extract_model(File::open(&part)?, store.root(), entry);
    let _ = fs::remove_file(&part);
    if result.is_err() {
        let _ = fs::remove_dir_all(store.root().join(entry.folder));
    }
    result
}
