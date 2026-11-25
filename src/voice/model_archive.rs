/*
**  _                                              _      ___    ___
** | |                                            | |    |__ \  / _ \
** | |_Created _       _ __   _ __    ___    __ _ | |__     ) || (_) |
** | '_ \ | | | |     | '_ \ | '_ \  / _ \  / _` || '_ \   / /  \__, |
** | |_) || |_| |     | | | || | | || (_) || (_| || | | | / /_    / /
** |_.__/  \__, |     |_| |_||_| |_| \___/  \__,_||_| |_||____|  /_/
**          __/ |     on 2026-11-22.
**         |___/
**
**
** Extraction sûre d'une archive tar.bz2 de modèle : seuls les quatre fichiers utiles, jamais hors du dossier.
*/

use super::install_error::ModelInstallError;
use super::model_catalog::ModelEntry;
use bzip2::read::BzDecoder;
use std::fs;
use std::io::Read;
use std::path::{Component, Path};

/// Refuse tout chemin absolu ou contenant `..`.
fn ensure_safe(path: &Path) -> Result<(), ModelInstallError> {
    let unsafe_part = path.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    });
    if unsafe_part {
        return Err(ModelInstallError::UnsafeArchive(path.display().to_string()));
    }
    Ok(())
}

/// Nom du fichier utile désigné par ce chemin d'archive, s'il en est un.
fn wanted_file<'a>(path: &'a Path, entry: &ModelEntry) -> Option<&'a str> {
    let name = path.file_name()?.to_str()?;
    entry.files.contains(&name).then_some(name)
}

/// Vérifie que les quatre fichiers sont présents après extraction.
fn verify_complete(target: &Path, entry: &ModelEntry) -> Result<(), ModelInstallError> {
    match entry.files.iter().find(|f| !target.join(f).is_file()) {
        Some(missing) => Err(ModelInstallError::Incomplete((*missing).to_string())),
        None => Ok(()),
    }
}

/// Extrait les fichiers du modèle dans `root/<dossier>` ; l'archive est lue sans jamais écrire ailleurs.
pub fn extract_model(
    reader: impl Read,
    root: &Path,
    entry: &ModelEntry,
) -> Result<(), ModelInstallError> {
    let mut archive = tar::Archive::new(BzDecoder::new(reader));
    let target = root.join(entry.folder);
    fs::create_dir_all(&target)?;
    for item in archive.entries()? {
        let mut item = item?;
        let path = item.path()?.into_owned();
        ensure_safe(&path)?;
        if let Some(name) =
            wanted_file(&path, entry).filter(|_| item.header().entry_type().is_file())
        {
            item.unpack(target.join(name))?;
        }
    }
    verify_complete(&target, entry)
}
