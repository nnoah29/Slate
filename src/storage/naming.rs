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
** Nom de fichier automatique des nouvelles notes et résolution des collisions.
*/

use std::path::{Path, PathBuf};

const MAX_STEM_CHARS: usize = 60;
const FORBIDDEN: [char; 9] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// Dérive un nom de fichier lisible de la première ligne non vide de la note.
pub fn note_file_stem(text: &str, fallback: &str) -> String {
    let first_line = text.lines().map(str::trim).find(|l| !l.is_empty());
    let cleaned = first_line.map(clean_line).unwrap_or_default();
    if cleaned.is_empty() {
        fallback.to_string()
    } else {
        cleaned
    }
}

/// Retire le balisage Markdown de tête et les caractères interdits dans un nom.
fn clean_line(line: &str) -> String {
    let without_markup = line.trim_start_matches(['#', '>', '-', '*', '+', ' ']);
    let safe: String = without_markup
        .chars()
        .filter(|c| !FORBIDDEN.contains(c) && !c.is_control())
        .collect();
    let truncated: String = safe.chars().take(MAX_STEM_CHARS).collect();
    truncated.trim().trim_matches('.').to_string()
}

/// Renvoie un chemin `.md` libre dans le dossier, en suffixant `(n)` si besoin.
pub fn unique_note_path(dir: &Path, stem: &str) -> PathBuf {
    let first = dir.join(format!("{stem}.md"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}).md")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}
