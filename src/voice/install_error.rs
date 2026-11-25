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
** Erreurs d'installation d'un modèle : réseau, intégrité, archive piégée, disque.
*/

use std::fmt;

/// Raison pour laquelle un modèle n'a pas pu être installé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelInstallError {
    Network(String),
    BadSize { expected: u64, got: u64 },
    BadChecksum,
    UnsafeArchive(String),
    Incomplete(String),
    Io(String),
}

impl fmt::Display for ModelInstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(why) => write!(f, "Téléchargement impossible : {why}"),
            Self::BadSize { expected, got } => {
                write!(
                    f,
                    "Taille inattendue : {got} octets reçus sur {expected} attendus."
                )
            }
            Self::BadChecksum => write!(f, "L'empreinte SHA-256 du modèle est incorrecte."),
            Self::UnsafeArchive(path) => write!(f, "Archive refusée (chemin dangereux) : {path}"),
            Self::Incomplete(file) => write!(f, "Archive incomplète : {file} manquant."),
            Self::Io(why) => write!(f, "Erreur disque : {why}"),
        }
    }
}

impl std::error::Error for ModelInstallError {}

impl From<std::io::Error> for ModelInstallError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
