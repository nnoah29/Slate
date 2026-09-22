use slate::config::{AppConfig, AutoSaveInterval, ThemeMode, MAX_RECENT_FILES};
use std::path::PathBuf;

#[test]
fn test_config_defaults() {
    let config = AppConfig::default();
    assert_eq!(config.theme, ThemeMode::Dark);
    assert_eq!(config.font_size, 12.0);
    assert_eq!(config.auto_save_interval, AutoSaveInterval::Sec15);
    assert!(config.recent_files.is_empty());
}

#[test]
fn test_config_serialization() {
    let mut config = AppConfig::default();
    config.theme = ThemeMode::Dark;
    config.font_size = 18.0;
    config.add_recent_file(PathBuf::from("/tmp/note.md"));

    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: AppConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.theme, ThemeMode::Dark);
    assert_eq!(deserialized.font_size, 18.0);
    assert_eq!(deserialized.recent_files.len(), 1);
}

#[test]
fn test_config_validation_and_recent_files_limit() {
    let mut config = AppConfig::default();
    config.font_size = 2.0; // too small
    config.window_width = 100; // too small
    config.validate();

    assert_eq!(config.font_size, slate::config::MIN_FONT_SIZE);
    assert_eq!(config.window_width, 320);

    // Test recent files capping
    for i in 0..30 {
        config.recent_files.push(PathBuf::from(format!("/tmp/note_{i}.md")));
    }
    config.validate();
    assert_eq!(config.recent_files.len(), MAX_RECENT_FILES);
}
