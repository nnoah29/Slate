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
** Dictée dans la fenêtre : Espace maintenu, Ctrl+D, Échap, téléchargement du modèle et messages.
*/

use crate::voice::controller::DictationMode;
use crate::voice::model_dialog::{self, InstallCallback};
use crate::voice::push_to_talk::PttAction;
use crate::voice::types::VoiceProblem;
use crate::window::SlateWindow;
use gtk4::prelude::*;
use gtk4::{gdk, glib};
use std::rc::Rc;

const BLOCKING_MODIFIERS: gdk::ModifierType = gdk::ModifierType::CONTROL_MASK
    .union(gdk::ModifierType::ALT_MASK)
    .union(gdk::ModifierType::SHIFT_MASK)
    .union(gdk::ModifierType::SUPER_MASK)
    .union(gdk::ModifierType::META_MASK);

/// Touches qui ne comptent pas comme une frappe : modificateurs seuls.
fn is_modifier_key(key: gdk::Key) -> bool {
    use gdk::Key as K;
    [
        K::Shift_L,
        K::Shift_R,
        K::Control_L,
        K::Control_R,
        K::Alt_L,
        K::Alt_R,
        K::Super_L,
        K::Super_R,
        K::Meta_L,
        K::Meta_R,
        K::Caps_Lock,
        K::Num_Lock,
        K::ISO_Level3_Shift,
    ]
    .contains(&key)
}

impl SlateWindow {
    /// Contrôleur clavier en phase de capture : il voit Espace avant le champ de texte.
    pub(crate) fn setup_dictation_keys(self: &Rc<Self>) {
        let keys = gtk4::EventControllerKey::new();
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, state| match weak.upgrade() {
            Some(this) => this.on_dictation_key_pressed(key, state),
            None => glib::Propagation::Proceed,
        });
        let weak = Rc::downgrade(self);
        keys.connect_key_released(move |_, key, _, _| {
            if let (gdk::Key::space, Some(this)) = (key, weak.upgrade()) {
                this.on_space_released();
            }
        });
        self.window.add_controller(keys);
    }

    /// Dit si Espace maintenu peut déclencher la dictée : édition, champ actif, modèle présent, aucun modificateur.
    fn dictation_eligible(&self, state: gdk::ModifierType) -> bool {
        let editing = self.stack.visible_child_name().as_deref() == Some("editor");
        let enabled = self.state.borrow().config.voice_push_to_talk;
        let installed = self.model_store.is_installed(self.dictation.language());
        !state.intersects(BLOCKING_MODIFIERS)
            && enabled
            && editing
            && installed
            && !self.dictation.is_active()
            && self.editor_view.view().has_focus()
    }

    /// Appui sur une touche : Espace est suivie, Échap annule la dictée, toute autre frappe écrit l'espace en attente.
    fn on_dictation_key_pressed(
        self: &Rc<Self>,
        key: gdk::Key,
        state: gdk::ModifierType,
    ) -> glib::Propagation {
        if key == gdk::Key::Escape && self.dictation.is_active() {
            self.ptt.borrow_mut().cancel();
            self.dictation.cancel();
            return glib::Propagation::Stop;
        }
        if key != gdk::Key::space {
            let flush = !is_modifier_key(key)
                && self.ptt.borrow_mut().other_key() == PttAction::InsertSpace;
            if flush {
                self.cancel_hold_timer();
                self.insert_typed_space();
            }
            return glib::Propagation::Proceed;
        }
        let repeat = self.ptt.borrow().is_active();
        let eligible = self.dictation_eligible(state);
        let action = self.ptt.borrow_mut().key_down(eligible, repeat);
        match action {
            PttAction::Swallow => {
                if !repeat {
                    self.arm_hold_timer();
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        }
    }

    /// Espace relâchée : écrit l'espace d'un appui court, ou termine la dictée.
    fn on_space_released(self: &Rc<Self>) {
        self.cancel_hold_timer();
        let action = self.ptt.borrow_mut().key_up();
        match action {
            PttAction::InsertSpace => self.insert_typed_space(),
            PttAction::StopDictation => self.dictation.stop(),
            _ => {}
        }
    }

    /// Invalide la minuterie de maintien en cours.
    fn cancel_hold_timer(&self) {
        self.ptt_generation
            .set(self.ptt_generation.get().wrapping_add(1));
    }

    /// Démarre la dictée si Espace est encore tenue après le délai de maintien.
    fn arm_hold_timer(self: &Rc<Self>) {
        self.cancel_hold_timer();
        let (weak, generation) = (Rc::downgrade(self), self.ptt_generation.get());
        let delay = self.ptt.borrow().hold_delay();
        glib::timeout_add_local_once(delay, move || {
            let Some(this) = weak
                .upgrade()
                .filter(|t| t.ptt_generation.get() == generation)
            else {
                return;
            };
            let action = this.ptt.borrow_mut().hold_elapsed();
            if action == PttAction::StartDictation {
                this.start_push_to_talk();
            }
        });
    }

    /// Lance la dictée phrase par phrase ; en cas d'échec, Espace redevient une touche ordinaire.
    fn start_push_to_talk(self: &Rc<Self>) {
        if let Err(problem) = self.dictation.start(DictationMode::PushToTalk) {
            self.ptt.borrow_mut().cancel();
            self.show_toast(&problem.to_string());
        }
    }

    /// Écrit une espace comme le ferait le champ de texte (remplace la sélection éventuelle).
    fn insert_typed_space(&self) {
        let buffer = self.editor_view.buffer().buffer();
        buffer.delete_selection(true, true);
        buffer.insert_interactive_at_cursor(" ", true);
    }

    /// Ctrl+D : arrête la dictée en cours, ou démarre une dictée continue.
    pub(crate) fn toggle_dictation(self: &Rc<Self>) {
        if self.dictation.is_active() {
            self.dictation.stop();
            return;
        }
        if self.stack.visible_child_name().as_deref() != Some("editor") {
            self.show_toast("Passez en mode édition (Ctrl+E) pour dicter.");
            return;
        }
        match self.dictation.start(DictationMode::Continuous) {
            Ok(()) => {}
            Err(VoiceProblem::ModelMissing) => self.offer_voice_model(),
            Err(problem) => self.show_toast(&problem.to_string()),
        }
    }

    /// Propose de télécharger le modèle de la langue courante, après confirmation.
    fn offer_voice_model(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        let on_done: InstallCallback = Rc::new(move |result| {
            let Some(this) = weak.upgrade() else { return };
            match result {
                Ok(()) => this.show_toast(
                    "Modèle installé : maintenez Espace ou appuyez sur Ctrl+D pour dicter.",
                ),
                Err(error) => this.show_toast(&error.to_string()),
            }
        });
        let store = self.model_store.clone();
        model_dialog::offer_download(&self.window, store, self.dictation.language(), on_done);
    }
}
