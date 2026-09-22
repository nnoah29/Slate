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
** User configuration management, XDG file persistence, and validation.
*/

use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_FONT_SIZE: f64 = 12.0;
pub const MIN_FONT_SIZE: f64 = 8.0;
pub const MAX_FONT_SIZE: f64 = 48.0;
pub const MAX_RECENT_FILES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    System,
    Light,
    #[default]
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutoSaveInterval {
    Disabled,
    Sec5,
    #[default]
    Sec15,
    Sec30,
    Min1,
}

impl AutoSaveInterval {
    pub fn as_seconds(&self) -> Option<u32> {
        match self {
            Self::Disabled => None,
            Self::Sec5 => Some(5),
            Self::Sec15 => Some(15),
            Self::Sec30 => Some(30),
            Self::Min1 => Some(60),
        }
    }

    #[allow(dead_code)]
    pub fn from_seconds(secs: Option<u32>) -> Self {
        match secs {
            None | Some(0) => Self::Disabled,
            Some(s) if s <= 5 => Self::Sec5,
            Some(s) if s <= 15 => Self::Sec15,
            Some(s) if s <= 30 => Self::Sec30,
            _ => Self::Min1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub font_size: f64,
    pub font_family: Option<String>,
    pub auto_save_interval: AutoSaveInterval,
    pub window_width: i32,
    pub window_height: i32,
    pub is_maximized: bool,
    #[serde(default)]
    pub save_directory: Option<PathBuf>,
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,
    #[serde(default)]
    pub custom_shortcuts: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            font_size: DEFAULT_FONT_SIZE,
            font_family: Some("Iosevka Nerd Font".to_string()),
            auto_save_interval: AutoSaveInterval::Sec15,
            window_width: 515,
            window_height: 338,
            is_maximized: false,
            save_directory: BaseDirs::new().map(|b| b.home_dir().to_path_buf()),
            recent_files: Vec::new(),
            custom_shortcuts: HashMap::new(),
        }
    }
}

impl AppConfig {
    pub fn resolved_save_directory(&self) -> Option<PathBuf> {
        let path = self.save_directory.as_ref()?;
        if let Ok(stripped) = path.strip_prefix("~") {
            return BaseDirs::new().map(|b| b.home_dir().join(stripped));
        }
        Some(path.clone())
    }

    pub fn config_dir_path() -> Option<PathBuf> {
        BaseDirs::new().map(|base| base.config_dir().join("Slate"))
    }

    pub fn legacy_config_file_path() -> Option<PathBuf> {
        BaseDirs::new().map(|base| base.config_dir().join("slate").join("config.json"))
    }

    pub fn config_file_path() -> Option<PathBuf> {
        Self::config_dir_path().map(|dir| dir.join("config.json"))
    }

    pub fn load_from(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {e}"))?;
        let mut config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {e}"))?;
        config.validate();
        Ok(config)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            return Err(format!("Failed to create config directory: {err}"));
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;
        fs::write(path, json).map_err(|e| format!("Failed to write config file: {e}"))?;
        Ok(())
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_file_path() else {
            return Self::default();
        };

        if path.exists() {
            return Self::load_from(&path).unwrap_or_else(|err| {
                tracing::warn!("{err}. Using defaults.");
                Self::default()
            });
        }

        if let Some(legacy) = Self::legacy_config_file_path()
            && legacy.exists()
        {
            let config = Self::load_from(&legacy).unwrap_or_default();
            let _ = config.save();
            return config;
        }

        let config = Self::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::config_file_path() else {
            return Err("Unable to determine config directory".into());
        };
        self.save_to(&path)
    }

    pub fn validate(&mut self) {
        self.font_size = self.font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
        self.window_width = self.window_width.max(320);
        self.window_height = self.window_height.max(240);
        if self.recent_files.len() > MAX_RECENT_FILES {
            self.recent_files.truncate(MAX_RECENT_FILES);
        }
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        let canonical = path.canonicalize().unwrap_or(path);
        self.recent_files.retain(|p| p != &canonical);
        self.recent_files.insert(0, canonical);
        if self.recent_files.len() > MAX_RECENT_FILES {
            self.recent_files.truncate(MAX_RECENT_FILES);
        }
    }

    #[allow(dead_code)]
    pub fn remove_recent_file(&mut self, path: &Path) {
        self.recent_files.retain(|p| p != path);
    }

    #[allow(dead_code)]
    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }
}
