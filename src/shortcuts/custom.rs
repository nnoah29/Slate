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
** Application des raccourcis personnalisés lus dans la configuration.
*/

use super::accelerator::parse_accelerator;
use super::{Action, ShortcutManager};
use std::collections::HashMap;

impl ShortcutManager {
    /// Remplace les raccourcis d'une action par ceux du fichier de config (`id` → `<Control>k`).
    pub fn apply_custom(&mut self, custom: &HashMap<String, String>) {
        for (id, accel) in custom {
            let Some(action) = Action::from_id(id) else {
                tracing::warn!("Raccourci personnalisé ignoré : action inconnue « {id} »");
                continue;
            };
            let Some(binding) = parse_accelerator(accel) else {
                tracing::warn!("Raccourci personnalisé ignoré : « {accel} » est invalide");
                continue;
            };
            self.remove_action(action);
            self.bind(
                binding.key,
                binding.ctrl,
                binding.shift,
                binding.alt,
                action,
            );
        }
    }
}
