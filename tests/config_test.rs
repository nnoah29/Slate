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
    assert_eq!(config.blur, slate::config::DEFAULT_BLUR);
    assert_eq!(config.opacity, slate::config::DEFAULT_OPACITY);
    assert!(config.recent_files.is_empty());
}

#[test]
fn test_config_serialization() {
    let mut config = AppConfig {
        theme: ThemeMode::Dark,
        font_size: 18.0,
        blur: 15.0,
        opacity: 0.8,
        save_directory: Some(PathBuf::from("/home/test/notes")),
        ..Default::default()
    };
    config.add_recent_file(PathBuf::from("/tmp/note.md"));

    let json = serde_json::to_string(&config).expect("serialize");
    let deserialized: AppConfig = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(deserialized.theme, ThemeMode::Dark);
    assert_eq!(deserialized.font_size, 18.0);
    assert_eq!(deserialized.blur, 15.0);
    assert_eq!(deserialized.opacity, 0.8);
    assert_eq!(
        deserialized.save_directory,
        Some(PathBuf::from("/home/test/notes"))
    );
    assert_eq!(deserialized.recent_files.len(), 1);
}

#[test]
fn test_config_aliases_deserialization() {
    let raw_json = r#"{"background_blur": 20.0, "background_opacity": 0.6}"#;
    let config: AppConfig = serde_json::from_str(raw_json).expect("deserialize aliases");
    assert_eq!(config.blur, 20.0);
    assert_eq!(config.opacity, 0.6);
}

#[test]
fn test_config_validation_and_recent_files_limit() {
    let mut config = AppConfig {
        font_size: 2.0,
        window_width: 100,
        blur: 250.0,
        opacity: 1.5,
        ..Default::default()
    };
    config.validate();

    assert_eq!(config.font_size, slate::config::MIN_FONT_SIZE);
    assert_eq!(config.window_width, 320);
    assert_eq!(config.blur, 100.0);
    assert_eq!(config.opacity, 1.0);

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

#[test]
fn test_config_save_directory_default_and_resolved() {
    let mut config = AppConfig::default();
    assert!(config.save_directory.is_some());

    config.save_directory = Some(PathBuf::from("~/Notes"));
    let resolved = config.resolved_save_directory();
    assert!(resolved.is_some());
    assert!(resolved.unwrap().ends_with("Notes"));
}

#[test]
fn test_custom_css_generation() {
    let css_no_blur = slate::app::generate_custom_css(0.5, 0.0);
    assert!(!css_no_blur.contains("backdrop-filter"));
    assert!(css_no_blur.contains("rgba(39, 38, 38, 0.50)"));

    let css_blur = slate::app::generate_custom_css(0.7, 12.0);
    assert!(css_blur.contains("backdrop-filter: blur(12.0px);"));
    assert!(css_blur.contains("rgba(39, 38, 38, 0.70)"));
}
