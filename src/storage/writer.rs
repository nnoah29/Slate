use super::StorageError;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

pub fn write_file_atomic(path: &Path, content: &str) -> Result<SystemTime, StorageError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| StorageError::Io {
            path: parent.display().to_string(),
            source: e,
        })?;
    }

    let file_stem = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("slate_note");
    let temp_name = format!(".{file_stem}.tmp.{}", std::process::id());
    let temp_path = parent.join(temp_name);

    let write_res = (|| -> Result<(), std::io::Error> {
        let mut file = File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = fs::remove_file(&temp_path);
        return Err(StorageError::Io {
            path: temp_path.display().to_string(),
            source: e,
        });
    }

    // Atomically replace original file
    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(StorageError::Io {
            path: path.display().to_string(),
            source: e,
        });
    }

    let mtime = fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or_else(|_| SystemTime::now());

    Ok(mtime)
}
