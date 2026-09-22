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
** Strict UTF-8 file reading and filesystem metadata inspection.
*/

use super::StorageError;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub fn read_file(path: &Path) -> Result<(String, SystemTime), StorageError> {
    if !path.exists() {
        return Err(StorageError::NotFound(path.display().to_string()));
    }

    let metadata = fs::metadata(path).map_err(|e| StorageError::Io {
        path: path.display().to_string(),
        source: e,
    })?;

    let mtime = metadata.modified().unwrap_or_else(|_| SystemTime::now());

    let bytes = fs::read(path).map_err(|e| StorageError::Io {
        path: path.display().to_string(),
        source: e,
    })?;

    let content = String::from_utf8(bytes)
        .map_err(|_| StorageError::InvalidUtf8(path.display().to_string()))?;

    Ok((content, mtime))
}
