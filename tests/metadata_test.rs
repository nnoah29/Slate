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
** Unit tests for word, line, and character count analytics.
*/

use slate::document::DocumentStats;

#[test]
fn test_document_stats_empty() {
    let stats = DocumentStats::compute("");
    assert_eq!(stats.char_count, 0);
    assert_eq!(stats.word_count, 0);
    assert_eq!(stats.line_count, 0);
    assert_eq!(stats.reading_time_secs, 0);
}

#[test]
fn test_document_stats_content() {
    let content =
        "# Titre\n\nVoici un paragraphe de test avec plusieurs mots pour compter la lecture.\n";
    let stats = DocumentStats::compute(content);
    assert_eq!(stats.line_count, 3);
    assert_eq!(stats.word_count, 14);
    assert!(stats.char_count > 0);
    assert!(stats.reading_time_secs >= 1);
}
