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
** Unit tests for configuration defaults, serialization, and boundary limits.
*/

use slate::config::{AppConfig, AutoSaveInterval, MAX_RECENT_FILES, ThemeMode};
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
    let mut config = AppConfig {
        theme: ThemeMode::Dark,
        font_size: 18.0,
        ..Default::default()
    };
    config.add_recent_file(PathBuf::from("/tmp/note.md"));

    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: AppConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.theme, ThemeMode::Dark);
    assert_eq!(deserialized.font_size, 18.0);
    assert_eq!(deserialized.recent_files.len(), 1);
}

#[test]
fn test_config_validation_and_recent_files_limit() {
    let mut config = AppConfig {
        font_size: 2.0,
        window_width: 100,
        ..Default::default()
    };
    config.validate();

    assert_eq!(config.font_size, slate::config::MIN_FONT_SIZE);
    assert_eq!(config.window_width, 320);

    for i in 0..30 {
        config
            .recent_files
            .push(PathBuf::from(format!("/tmp/note_{i}.md")));
    }
    config.validate();
    assert_eq!(config.recent_files.len(), MAX_RECENT_FILES);
}

#[test]
fn test_config_path_target() {
    let path = AppConfig::config_file_path();
    assert!(path.is_some());
    let unwrapped = path.unwrap();
    assert!(unwrapped.ends_with("Slate/config.json"));
}

#[test]
fn test_config_load_save_roundtrip() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let config_path = temp_dir.path().join("config.json");

    let config = AppConfig {
        font_size: 16.0,
        ..Default::default()
    };
    config.save_to(&config_path).expect("save");

    let loaded = AppConfig::load_from(&config_path).expect("load");
    assert_eq!(loaded.font_size, 16.0);
}
