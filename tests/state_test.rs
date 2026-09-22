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
** Unit tests for document lifecycle and clean/dirty state transitions.
*/

use slate::document::state::{Document, DocumentStatus};
use std::path::PathBuf;
use std::time::SystemTime;

#[test]
fn test_untitled_document_state() {
    let mut doc = Document::new_untitled();
    assert_eq!(doc.display_name(), "Sans titre");
    assert_eq!(doc.window_title(), "Sans titre");
    assert!(!doc.is_dirty());

    doc.mark_dirty();
    assert!(doc.is_dirty());
    assert_eq!(doc.window_title(), "Sans titre *");

    doc.mark_clean(SystemTime::now());
    assert!(!doc.is_dirty());
    assert_eq!(doc.window_title(), "Sans titre");
}

#[test]
fn test_path_document_state() {
    let path = PathBuf::from("/home/user/notes/projet.md");
    let mut doc = Document::from_path(path.clone(), SystemTime::now());
    assert_eq!(doc.display_name(), "projet.md");
    assert_eq!(doc.window_title(), "projet.md");
    assert_eq!(doc.path(), Some(path.as_path()));
    assert!(!doc.is_dirty());

    doc.mark_dirty();
    assert!(doc.is_dirty());
    assert_eq!(doc.window_title(), "projet.md *");

    doc.mark_saving();
    assert_eq!(doc.status(), &DocumentStatus::Saving);

    doc.mark_error("Permission refusée");
    assert_eq!(
        doc.status(),
        &DocumentStatus::Error("Permission refusée".to_string())
    );
}
