pub mod reader;
pub mod writer;

pub use reader::read_file;
pub use writer::write_file_atomic;

use similar::{ChangeTag, TextDiff};
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    #[allow(dead_code)]
    PermissionDenied(String),
    InvalidUtf8(String),
    Io { path: String, source: std::io::Error },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "Le fichier n'a pas été trouvé : {path}"),
            Self::PermissionDenied(path) => write!(f, "Permission refusée pour le fichier : {path}"),
            Self::InvalidUtf8(path) => write!(f, "Le fichier n'est pas un texte UTF-8 valide : {path}"),
            Self::Io { path, source } => write!(f, "Erreur d'accès à {path} : {source}"),
        }
    }
}

impl std::error::Error for StorageError {}

pub fn check_external_modification(path: &Path, recorded_mtime: SystemTime) -> Result<bool, StorageError> {
    if !path.exists() {
        return Ok(false);
    }
    let metadata = fs::metadata(path).map_err(|e| StorageError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    let current_mtime = metadata.modified().unwrap_or(recorded_mtime);

    // If current mtime on disk is strictly newer than recorded mtime
    Ok(current_mtime > recorded_mtime)
}

pub fn generate_diff(disk_content: &str, memory_content: &str) -> String {
    let diff = TextDiff::from_lines(disk_content, memory_content);
    let mut output = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };
        output.push_str(sign);
        output.push_str(change.value());
    }

    output
}
