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
** Indicateur de dictée : pastille « Préparation… / À l'écoute » et barre de niveau sonore.
*/

use gtk4::prelude::*;

/// Pastille affichée pendant une dictée, en bas à droite de la note.
pub struct DictationIndicator {
    root: gtk4::Box,
    label: gtk4::Label,
    level: gtk4::LevelBar,
}

impl DictationIndicator {
    /// Crée l'indicateur, masqué ; il ne capte jamais les clics.
    pub fn new() -> Self {
        let root = gtk4::Box::builder()
            .spacing(10)
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::End)
            .margin_end(14)
            .margin_bottom(14)
            .visible(false)
            .can_target(false)
            .css_classes(["dictation-indicator"])
            .build();
        let label = gtk4::Label::new(None);
        let level = gtk4::LevelBar::for_interval(0.0, 1.0);
        level.set_valign(gtk4::Align::Center);
        level.set_size_request(64, -1);
        root.append(&label);
        root.append(&level);
        Self { root, label, level }
    }

    /// Widget à placer en surimpression.
    pub fn widget(&self) -> &gtk4::Box {
        &self.root
    }

    /// Micro ouvert, modèle en cours de chargement.
    pub fn show_preparing(&self) {
        self.label.set_text("● Préparation…");
        self.level.set_value(0.0);
        self.root.set_visible(true);
    }

    /// Modèle prêt : la parole est reconnue.
    pub fn show_listening(&self) {
        self.label.set_text("● À l'écoute");
    }

    /// Niveau sonore de 0 à 1.
    pub fn set_level(&self, level: f32) {
        self.level.set_value(f64::from(level.clamp(0.0, 1.0)));
    }

    /// Masque la pastille.
    pub fn hide(&self) {
        self.root.set_visible(false);
    }
}

impl Default for DictationIndicator {
    fn default() -> Self {
        Self::new()
    }
}
