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
** Dossier d'installation des modèles de dictée (~/.config/Slate/voice-models/) et leur état.
*/

use super::model_catalog::entry_for;
use super::types::VoiceLanguage;
use crate::config::AppConfig;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Dossier racine des modèles et opérations d'état sur chaque langue.
#[derive(Debug, Clone)]
pub struct ModelStore {
    root: PathBuf,
}

impl ModelStore {
    /// Crée un magasin sur un dossier racine donné.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Dossier par défaut : `voice-models` dans le répertoire de configuration de Slate.
    pub fn default_root() -> Option<PathBuf> {
        AppConfig::config_dir_path().map(|dir| dir.join("voice-models"))
    }

    /// Dossier racine des modèles.
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Dossier du modèle d'une langue.
    pub fn model_dir(&self, language: VoiceLanguage) -> PathBuf {
        self.root.join(entry_for(language).folder)
    }

    /// Vrai si les quatre fichiers du modèle sont présents.
    pub fn is_installed(&self, language: VoiceLanguage) -> bool {
        let dir = self.model_dir(language);
        entry_for(language)
            .files
            .iter()
            .all(|f| dir.join(f).is_file())
    }

    /// Supprime le modèle d'une langue, sans toucher aux autres.
    pub fn remove(&self, language: VoiceLanguage) -> io::Result<()> {
        let dir = self.model_dir(language);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
}
