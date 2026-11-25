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
** Décide si la fermeture ou le changement de note doit proposer d'enregistrer.
*/

/// Vrai si des modifications méritent une invite : une note sans nom encore vide n'en demande pas.
pub fn needs_save_prompt(is_dirty: bool, has_path: bool, text: &str) -> bool {
    is_dirty && (has_path || !text.trim().is_empty())
}
