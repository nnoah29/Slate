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
** Tests du catalogue de modèles et du dossier d'installation (T3).
*/

use slate::voice::model_catalog::{ModelEntry, entry_for};
use slate::voice::model_store::ModelStore;
use slate::voice::types::VoiceLanguage;
use std::fs;

fn install(store: &ModelStore, entry: &ModelEntry) {
    let dir = store.model_dir(entry.language);
    fs::create_dir_all(&dir).expect("dir");
    for file in entry.files {
        fs::write(dir.join(file), "x").expect("file");
    }
}

#[test]
fn test_catalog_entries_are_pinned_and_from_official_source() {
    for language in [VoiceLanguage::Fr, VoiceLanguage::En] {
        let entry = entry_for(language);
        assert_eq!(entry.language, language);
        assert_eq!(entry.sha256.len(), 64);
        assert!(entry.sha256.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            entry
                .url
                .starts_with("https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/")
        );
        assert!(entry.url.ends_with(".tar.bz2"));
        assert_eq!(
            entry.files,
            ["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"]
        );
    }
}

#[test]
fn test_default_root_lives_in_slate_config_directory() {
    let root = ModelStore::default_root().expect("root");
    assert!(root.ends_with("Slate/voice-models"));
}

#[test]
fn test_model_is_installed_only_when_all_files_exist() {
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().to_path_buf());
    assert!(!store.is_installed(VoiceLanguage::Fr));
    install(&store, entry_for(VoiceLanguage::Fr));
    assert!(store.is_installed(VoiceLanguage::Fr));
    assert!(!store.is_installed(VoiceLanguage::En));
    fs::remove_file(store.model_dir(VoiceLanguage::Fr).join("tokens.txt")).expect("rm");
    assert!(!store.is_installed(VoiceLanguage::Fr));
}

#[test]
fn test_remove_deletes_the_language_folder_only() {
    let tmp = tempfile::tempdir().expect("tmp");
    let store = ModelStore::new(tmp.path().to_path_buf());
    install(&store, entry_for(VoiceLanguage::Fr));
    install(&store, entry_for(VoiceLanguage::En));
    store.remove(VoiceLanguage::Fr).expect("remove");
    assert!(!store.is_installed(VoiceLanguage::Fr));
    assert!(store.is_installed(VoiceLanguage::En));
}
