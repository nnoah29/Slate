/*
**  _                                              _      ___    ___
** | |                                            | |    |__ \  / _ \
** | |_Created _       _ __   _ __    ___    __ _ | |__     ) || (_) |
** | '_ \ | | | |     | '_ \ | '_ \  / _ \  / _` || '_ \   / /  \__, |
** | |_) || |_| |     | | | || | | || (_) || (_| || | | | / /_    / /
** |_.__/  \__, |     |_| |_||_| |_| \___/  \__,_||_| |_||____|  /_/
**          __/ |     on 2026-09-22.
**         |___/
**
** Unit tests for atomic disk persistence, conflict detection, and diffs.
*/

use slate::storage::{check_external_modification, generate_diff, read_file, write_file_atomic};
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn test_write_and_read_atomic() {
    let tmp = NamedTempFile::new().expect("create temp file");
    let path = tmp.path();

    let text =
        "# Ma note\n\nAvec des accents éèêë, des caractères japonais 日本語 et des emojis 🚀 ✨\n";
    let mtime1 = write_file_atomic(path, text).expect("write file");

    let (content, mtime2) = read_file(path).expect("read file");
    assert_eq!(content, text);
    assert_eq!(mtime1, mtime2);
}

#[test]
fn test_read_nonexistent() {
    let path = std::path::Path::new("/tmp/slate_non_existent_12345.md");
    let result = read_file(path);
    assert!(result.is_err());
}

#[test]
fn test_external_modification_detection() {
    let tmp = NamedTempFile::new().expect("create temp file");
    let path = tmp.path();

    let initial = "Version initiale";
    let mtime = write_file_atomic(path, initial).expect("write initial");

    let modified = check_external_modification(path, mtime).expect("check mod");
    assert!(!modified);

    thread::sleep(Duration::from_millis(1100));

    fs::write(path, "Version modifiée ailleurs").expect("external write");

    let modified_after = check_external_modification(path, mtime).expect("check mod after");
    assert!(modified_after);
}

#[test]
fn test_generate_diff() {
    let disk = "Ligne 1\nLigne 2 modifiée\nLigne 3\n";
    let memory = "Ligne 1\nLigne 2 locale\nLigne 3\n";
    let diff = generate_diff(disk, memory);

    assert!(diff.contains("- Ligne 2 modifiée"));
    assert!(diff.contains("+ Ligne 2 locale"));
}
