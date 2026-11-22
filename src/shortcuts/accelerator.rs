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
** Lecture et écriture des raccourcis au format GTK (`<Control><Shift>k`), sans GTK initialisé.
*/

use super::KeyBinding;
use gtk4::gdk::Key;

/// Écrit un raccourci au format GTK à partir du nom de la touche.
pub(crate) fn format_accelerator(ctrl: bool, shift: bool, alt: bool, key_name: &str) -> String {
    let ctrl = if ctrl { "<Control>" } else { "" };
    let shift = if shift { "<Shift>" } else { "" };
    let alt = if alt { "<Alt>" } else { "" };
    format!("{ctrl}{shift}{alt}{key_name}")
}

/// Lit un raccourci au format GTK ; renvoie `None` si la touche est inconnue.
pub(crate) fn parse_accelerator(text: &str) -> Option<KeyBinding> {
    let (mut ctrl, mut shift, mut alt) = (false, false, false);
    let mut rest = text.trim();
    while let Some(stripped) = rest.strip_prefix('<') {
        let (modifier, after) = stripped.split_once('>')?;
        match modifier.to_ascii_lowercase().as_str() {
            "control" | "ctrl" | "primary" => ctrl = true,
            "shift" => shift = true,
            "alt" => alt = true,
            _ => return None,
        }
        rest = after;
    }
    let key = Key::from_name(rest)?;
    Some(KeyBinding::new(key, ctrl, shift, alt))
}
