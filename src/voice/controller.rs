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
** Pilote une dictée dans la note : moteur, insertion, indicateur, minuteries, garde du modèle en mémoire.
*/

use super::buffer_target::BufferTarget;
use super::engine::{EngineEvent, VoiceEngine};
use super::indicator::DictationIndicator;
use super::inserter::Inserter;
use super::types::{RecognizerEvent, VoiceLanguage, VoiceProblem};
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

const POLL_MS: u64 = 40;
const MAX_DICTATION_SECONDS: u32 = 300;

/// Façon de dicter : en continu jusqu'à un nouvel ordre, ou tant qu'Espace est tenue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictationMode {
    Continuous,
    PushToTalk,
}

/// Une dictée à la fois dans la note : démarrage, arrêt, annulation et garde du modèle.
pub struct DictationController {
    engine: RefCell<VoiceEngine>,
    inserter: RefCell<Inserter>,
    buffer: gtk4::TextBuffer,
    indicator: Rc<DictationIndicator>,
    toast: Rc<dyn Fn(&str)>,
    mode: Cell<DictationMode>,
    active: Cell<bool>,
    wrote_text: Cell<bool>,
    group_open: Cell<bool>,
    language: Cell<VoiceLanguage>,
    keep_minutes: Cell<u32>,
    generation: Cell<u32>,
}

impl DictationController {
    /// Contrôleur sur le buffer de la note, avec son moteur, son indicateur et un moyen d'afficher un message.
    pub fn new(
        engine: VoiceEngine,
        buffer: gtk4::TextBuffer,
        indicator: Rc<DictationIndicator>,
        toast: Rc<dyn Fn(&str)>,
        language: VoiceLanguage,
        keep_minutes: u32,
    ) -> Rc<Self> {
        Rc::new(Self {
            engine: RefCell::new(engine),
            inserter: RefCell::new(Inserter::default()),
            buffer,
            indicator,
            toast,
            mode: Cell::new(DictationMode::Continuous),
            active: Cell::new(false),
            wrote_text: Cell::new(false),
            group_open: Cell::new(false),
            language: Cell::new(language),
            keep_minutes: Cell::new(keep_minutes),
            generation: Cell::new(0),
        })
    }

    /// Dit si une dictée est en cours (préparation, écoute ou fin de traitement).
    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    /// Langue de dictée courante.
    pub fn language(&self) -> VoiceLanguage {
        self.language.get()
    }

    /// Démarre une dictée à la position du curseur.
    pub fn start(self: &Rc<Self>, mode: DictationMode) -> Result<(), VoiceProblem> {
        if self.active.get() {
            return Err(VoiceProblem::Busy);
        }
        self.engine.borrow_mut().start(self.language.get())?;
        self.generation.set(self.generation.get() + 1);
        *self.inserter.borrow_mut() = Inserter::default();
        self.mode.set(mode);
        self.wrote_text.set(false);
        self.active.set(true);
        self.buffer.begin_user_action();
        self.group_open.set(true);
        self.indicator.show_preparing();
        self.arm_poll();
        self.arm_limit();
        Ok(())
    }

    /// Arrête le micro ; le texte restant est validé puis la dictée se termine d'elle-même.
    pub fn stop(&self) {
        if self.active.get() {
            self.engine.borrow_mut().stop();
        }
    }

    /// Annule : rien de plus n'est écrit et le modèle est libéré.
    pub fn cancel(self: &Rc<Self>) {
        if !self.active.get() {
            return;
        }
        self.engine.borrow_mut().cancel();
        self.finish(false);
    }

    /// Fermeture de l'application : annule et attend la libération des modèles.
    pub fn shutdown(self: &Rc<Self>) {
        self.cancel();
        self.engine.borrow_mut().shutdown();
    }

    /// Lance la pompe qui lit les événements du moteur ; elle s'arrête d'elle-même à la fin de la dictée.
    fn arm_poll(self: &Rc<Self>) {
        let (weak, generation) = (Rc::downgrade(self), self.generation.get());
        glib::timeout_add_local(Duration::from_millis(POLL_MS), move || {
            let Some(this) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if !this.active.get() || this.generation.get() != generation {
                return glib::ControlFlow::Break;
            }
            this.tick();
            glib::ControlFlow::Continue
        });
    }

    /// Arrête la dictée au bout de cinq minutes.
    fn arm_limit(self: &Rc<Self>) {
        let (weak, generation) = (Rc::downgrade(self), self.generation.get());
        glib::timeout_add_seconds_local_once(MAX_DICTATION_SECONDS, move || {
            if let Some(this) = weak.upgrade().filter(|t| t.generation.get() == generation) {
                this.stop();
            }
        });
    }

    /// Traite les événements arrivés depuis le dernier passage.
    fn tick(self: &Rc<Self>) {
        let events = self.engine.borrow_mut().poll();
        for event in events {
            self.handle(event);
        }
    }

    /// Un événement du moteur : niveau, texte, échec ou fin.
    fn handle(self: &Rc<Self>, event: EngineEvent) {
        match event {
            EngineEvent::Level(level) => self.on_level(level),
            EngineEvent::Text(RecognizerEvent::Partial(text)) => self.write(&text, false),
            EngineEvent::Text(RecognizerEvent::Final(text)) => self.write(&text, true),
            EngineEvent::Failed(problem) => {
                (self.toast)(&problem.to_string());
                self.finish(false);
            }
            EngineEvent::Stopped => self.finish(true),
        }
    }

    /// Anime la pastille ; elle passe à « À l'écoute » dès que le modèle est prêt.
    fn on_level(&self, level: f32) {
        self.indicator.set_level(level);
        if self.engine.borrow().is_ready() {
            self.indicator.show_listening();
        }
    }

    /// Écrit un texte provisoire ou définitif dans la note.
    fn write(&self, text: &str, commit: bool) {
        let mut target = BufferTarget::new(&self.buffer);
        let mut inserter = self.inserter.borrow_mut();
        if commit {
            inserter.commit(&mut target, text);
        } else {
            inserter.show_partial(&mut target, text);
        }
        if !text.trim().is_empty() {
            self.wrote_text.set(true);
        }
    }

    /// Fin de dictée : regroupe l'annulation, masque la pastille, rend l'espace si rien n'a été dit.
    fn finish(self: &Rc<Self>, completed: bool) {
        self.active.set(false);
        self.inserter.borrow_mut().release();
        if self.group_open.replace(false) {
            self.buffer.end_user_action();
        }
        self.indicator.hide();
        let silent_push = self.mode.get() == DictationMode::PushToTalk && !self.wrote_text.get();
        if completed && silent_push {
            self.buffer.insert_at_cursor(" ");
        }
        self.schedule_model_release();
    }

    /// Garde le modèle en mémoire quelques minutes pour que la dictée suivante démarre sans attente.
    fn schedule_model_release(self: &Rc<Self>) {
        let minutes = self.keep_minutes.get();
        if minutes == 0 {
            self.engine.borrow_mut().release_model();
            return;
        }
        let (weak, generation) = (Rc::downgrade(self), self.generation.get());
        glib::timeout_add_seconds_local_once(minutes * 60, move || {
            let Some(this) = weak.upgrade() else { return };
            if !this.active.get() && this.generation.get() == generation {
                this.engine.borrow_mut().release_model();
            }
        });
    }
}
