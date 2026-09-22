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
** Document state tracking and modification status management.
*/

use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentStatus {
    Clean,
    Dirty,
    Saving,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Document {
    path: Option<PathBuf>,
    status: DocumentStatus,
    last_mtime: Option<SystemTime>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new_untitled()
    }
}

impl Document {
    pub fn new_untitled() -> Self {
        Self {
            path: None,
            status: DocumentStatus::Clean,
            last_mtime: None,
        }
    }

    pub fn from_path(path: PathBuf, mtime: SystemTime) -> Self {
        Self {
            path: Some(path),
            status: DocumentStatus::Clean,
            last_mtime: Some(mtime),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    #[allow(dead_code)]
    pub fn status(&self) -> &DocumentStatus {
        &self.status
    }

    pub fn is_dirty(&self) -> bool {
        matches!(self.status, DocumentStatus::Dirty)
    }

    pub fn mark_clean(&mut self, mtime: SystemTime) {
        self.status = DocumentStatus::Clean;
        self.last_mtime = Some(mtime);
    }

    pub fn mark_dirty(&mut self) {
        if self.status != DocumentStatus::Dirty {
            self.status = DocumentStatus::Dirty;
        }
    }

    pub fn mark_saving(&mut self) {
        self.status = DocumentStatus::Saving;
    }

    pub fn mark_error(&mut self, err: impl Into<String>) {
        self.status = DocumentStatus::Error(err.into());
    }

    pub fn last_mtime(&self) -> Option<SystemTime> {
        self.last_mtime
    }

    pub fn set_last_mtime(&mut self, mtime: Option<SystemTime>) {
        self.last_mtime = mtime;
    }

    pub fn display_name(&self) -> String {
        match &self.path {
            Some(path) => path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Document")
                .to_string(),
            None => "Sans titre".to_string(),
        }
    }

    pub fn window_title(&self) -> String {
        let name = self.display_name();
        if self.is_dirty() {
            format!("{name} *")
        } else {
            name
        }
    }
}
