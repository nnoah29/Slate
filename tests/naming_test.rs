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
** Tests du nommage automatique des notes dans le dossier dédié.
*/

use slate::storage::naming::{note_file_stem, unique_note_path};
use std::fs;

#[test]
fn test_stem_uses_first_heading_without_markup() {
    assert_eq!(
        note_file_stem("# Idées de projet\n\ntexte", "Note"),
        "Idées de projet"
    );
}

#[test]
fn test_stem_skips_leading_blank_lines_and_list_markers() {
    assert_eq!(
        note_file_stem("\n\n- courses du jour", "Note"),
        "courses du jour"
    );
}

#[test]
fn test_stem_removes_forbidden_characters() {
    assert_eq!(note_file_stem("a/b:c*d?e", "Note"), "abcde");
}

#[test]
fn test_stem_truncates_long_titles() {
    let long = "x".repeat(200);
    assert_eq!(note_file_stem(&long, "Note").chars().count(), 60);
}

#[test]
fn test_stem_falls_back_when_empty() {
    assert_eq!(note_file_stem("###  \n", "Note du jour"), "Note du jour");
}

#[test]
fn test_unique_path_adds_md_extension() {
    let dir = tempfile::tempdir().expect("tmp");
    let path = unique_note_path(dir.path(), "Titre");
    assert_eq!(path, dir.path().join("Titre.md"));
}

#[test]
fn test_unique_path_never_overwrites_existing_note() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(dir.path().join("Titre.md"), "a").expect("write");
    fs::write(dir.path().join("Titre (2).md"), "b").expect("write");
    let path = unique_note_path(dir.path(), "Titre");
    assert_eq!(path, dir.path().join("Titre (3).md"));
}
