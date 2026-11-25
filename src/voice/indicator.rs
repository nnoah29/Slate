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
** Indicateur de dictée : un caractère-bloc posé à droite du curseur d'écriture, animé par la voix.
*/

use gtk4::prelude::*;

use super::indicator_glyph::glyph_for_level;
use super::indicator_position::caret_in_overlay;

/// Caractère-bloc suivant le curseur pendant une dictée ; il ne capte jamais les clics.
pub struct DictationIndicator {
    glyph: gtk4::Label,
    overlay: gtk4::Overlay,
    view: gtk4::TextView,
}

impl DictationIndicator {
    /// Crée l'indicateur, masqué, en surimpression sur `overlay` ; `view` donne la position du curseur.
    pub fn new(overlay: &gtk4::Overlay, view: &gtk4::TextView) -> Self {
        let glyph = gtk4::Label::builder()
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Start)
            .visible(false)
            .can_target(false)
            .css_classes(["dictation-glyph"])
            .build();
        overlay.add_overlay(&glyph);
        Self {
            glyph,
            overlay: overlay.clone(),
            view: view.clone(),
        }
    }

    /// Micro ouvert, modèle en cours de chargement : bloc bas et atténué.
    pub fn show_preparing(&self) {
        self.glyph.set_text("▁");
        self.glyph.add_css_class("preparing");
        self.reposition();
    }

    /// Modèle prêt : le bloc prend la couleur d'accent.
    pub fn show_listening(&self) {
        self.glyph.remove_css_class("preparing");
    }

    /// Niveau sonore de 0 à 1 : change la hauteur du bloc et le replace au curseur.
    pub fn set_level(&self, level: f32) {
        self.glyph.set_text(&glyph_for_level(level).to_string());
        self.reposition();
    }

    /// Masque le bloc.
    pub fn hide(&self) {
        self.glyph.set_visible(false);
    }

    /// Replace le bloc juste après le curseur ; masqué si le curseur n'est pas visible.
    pub fn reposition(&self) {
        let Some((x, y)) = caret_in_overlay(&self.view, &self.overlay) else {
            self.glyph.set_visible(false);
            return;
        };
        self.glyph.set_margin_start(x + 3);
        self.glyph.set_margin_top(y);
        self.glyph.set_visible(true);
    }
}
