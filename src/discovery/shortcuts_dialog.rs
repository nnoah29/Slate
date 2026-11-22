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
** Fenêtre d'aide listant tous les raccourcis clavier par catégorie.
*/

use crate::shortcuts::{ShortcutManager, all_actions};
use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::{ShortcutsDialog, ShortcutsItem, ShortcutsSection};

pub const CATEGORIES: [&str; 5] = ["Fichier", "Édition", "Formatage", "Affichage", "Aide"];

/// Construit une section d'aide pour une catégorie d'actions.
fn build_section(category: &str, mgr: &ShortcutManager) -> ShortcutsSection {
    let section = ShortcutsSection::new(Some(category));
    for action in all_actions()
        .into_iter()
        .filter(|a| a.category() == category)
    {
        if let Some(accel) = mgr.accelerator_for(action) {
            section.add(ShortcutsItem::new(action.label(), &accel));
        }
    }
    section
}

/// Affiche la fenêtre des raccourcis au-dessus de la fenêtre parente.
pub fn present_shortcuts_dialog(parent: &impl IsA<gtk4::Widget>, mgr: &ShortcutManager) {
    let dialog = ShortcutsDialog::new();
    for category in CATEGORIES {
        dialog.add(build_section(category, mgr));
    }
    dialog.present(Some(parent));
}
