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
    let content = "# Titre\n\nVoici un paragraphe de test avec plusieurs mots pour compter la lecture.\n";
    let stats = DocumentStats::compute(content);
    assert_eq!(stats.line_count, 3);
    assert_eq!(stats.word_count, 14);
    assert!(stats.char_count > 0);
    assert!(stats.reading_time_secs >= 1);
}
