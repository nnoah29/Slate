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
** Types partagés de la dictée : langues, événements du reconnaisseur, problèmes.
*/

use serde::{Deserialize, Serialize};
use std::fmt;

/// Langues dictables ; chacune a son propre modèle de reconnaissance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VoiceLanguage {
    #[default]
    Fr,
    En,
}

#[allow(dead_code)]
impl VoiceLanguage {
    /// Code court de la langue (`fr`, `en`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Fr => "fr",
            Self::En => "en",
        }
    }

    /// Retrouve une langue à partir de son code court.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "fr" => Some(Self::Fr),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    /// Langue pour une locale système ; tout ce qui n'est pas anglais retombe sur le français.
    pub fn from_locale(locale: &str) -> Self {
        if locale.to_lowercase().starts_with("en") {
            Self::En
        } else {
            Self::Fr
        }
    }
}

/// Texte produit par le reconnaisseur : provisoire (peut changer) ou définitif (fin de phrase).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecognizerEvent {
    Partial(String),
    Final(String),
}

/// Raison pour laquelle une dictée n'a pas démarré ou s'est arrêtée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceProblem {
    Busy,
    ModelMissing,
    MicrophoneMissing,
    MicrophoneUnavailable(String),
    RecognizerFailed(String),
}

impl fmt::Display for VoiceProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => write!(f, "Une dictée est déjà en cours."),
            Self::ModelMissing => write!(f, "Le modèle de dictée n'est pas installé."),
            Self::MicrophoneMissing => write!(f, "Micro introuvable (parecord est requis)."),
            Self::MicrophoneUnavailable(why) => write!(f, "Micro indisponible : {why}"),
            Self::RecognizerFailed(why) => write!(f, "Échec de la reconnaissance : {why}"),
        }
    }
}

impl std::error::Error for VoiceProblem {}
