use directories::ProjectDirs;
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
            recent_files: Vec::new(),
            custom_shortcuts: HashMap::new(),
        }
    }
}

impl AppConfig {
    pub fn config_file_path() -> Option<PathBuf> {
        ProjectDirs::from("com.github.slate", "Slate", "slate")
            .map(|proj| proj.config_dir().join("config.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_file_path() else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                Ok(mut config) => {
                    config.validate();
                    config
                }
                Err(err) => {
                    tracing::warn!("Failed to parse config file: {err}. Using defaults.");
                    Self::default()
                }
            },
            Err(err) => {
                tracing::warn!("Failed to read config file: {err}. Using defaults.");
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::config_file_path() else {
            return Err("Unable to determine config directory".into());
        };

        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(format!("Failed to create config directory: {err}"));
            }
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write config file: {e}"))?;
        Ok(())
    }

    pub fn validate(&mut self) {
        if self.font_size < MIN_FONT_SIZE {
            self.font_size = MIN_FONT_SIZE;
        } else if self.font_size > MAX_FONT_SIZE {
            self.font_size = MAX_FONT_SIZE;
        }
        if self.window_width < 320 {
            self.window_width = 320;
        }
        if self.window_height < 240 {
            self.window_height = 240;
        }
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
