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
** Moteur de dictée : micro ouvert avant le modèle, réserve audio rejouée, arrêt jusqu'au bout, annulation.
*/

use super::backlog::Backlog;
use super::microphone::{MicMessage, Microphone};
use super::model_store::ModelStore;
use super::recognizer_worker::{RecognizerWorker, WorkerCommand, WorkerMessage};
use super::types::{RecognizerEvent, VoiceLanguage, VoiceProblem};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;

const BACKLOG_SAMPLES: usize = 16_000 * 30;

/// Ce que l'interface doit savoir après chaque `poll`.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    Level(f32),
    Text(RecognizerEvent),
    Failed(VoiceProblem),
    Stopped,
}

/// Source audio : ouvre un flux de paquets, ou renvoie la raison de l'échec.
pub trait AudioSource {
    /// Ouvre le micro ; les paquets arrivent sur `out`.
    fn start(&mut self, out: Sender<MicMessage>) -> Result<(), VoiceProblem>;
    /// Coupe le micro immédiatement.
    fn stop(&mut self);
}

/// Micro réel : `parecord`, sur le périphérique par défaut ou un périphérique nommé.
#[derive(Default)]
pub struct ParecordSource {
    device: Option<String>,
    mic: Option<Microphone>,
}

impl ParecordSource {
    /// Source sur un périphérique donné (`None` : micro par défaut du système).
    pub fn new(device: Option<String>) -> Self {
        Self { device, mic: None }
    }
}

impl AudioSource for ParecordSource {
    fn start(&mut self, out: Sender<MicMessage>) -> Result<(), VoiceProblem> {
        self.mic = Some(Microphone::start(self.device.as_deref(), out)?);
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(mic) = self.mic.take() {
            mic.stop();
        }
    }
}

/// Étape de la dictée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Idle,
    Listening,
    Stopping,
}

/// Une dictée à la fois : micro, réserve audio, fil de reconnaissance.
pub struct VoiceEngine {
    pub(super) store: ModelStore,
    pub(super) source: Box<dyn AudioSource>,
    pub(super) mic_tx: Sender<MicMessage>,
    pub(super) mic_rx: Receiver<MicMessage>,
    pub(super) msg_tx: Sender<WorkerMessage>,
    pub(super) msg_rx: Receiver<WorkerMessage>,
    pub(super) worker: Option<RecognizerWorker>,
    pub(super) loaded: Option<VoiceLanguage>,
    pub(super) phase: Phase,
    pub(super) ready: bool,
    pub(super) finish_sent: bool,
    pub(super) backlog: Backlog,
    retiring: Vec<JoinHandle<()>>,
}

impl VoiceEngine {
    /// Moteur sur un magasin de modèles et une source audio.
    pub fn new(store: ModelStore, source: Box<dyn AudioSource>) -> Self {
        let (mic_tx, mic_rx) = channel();
        let (msg_tx, msg_rx) = channel();
        Self {
            store,
            source,
            mic_tx,
            mic_rx,
            msg_tx,
            msg_rx,
            worker: None,
            loaded: None,
            phase: Phase::Idle,
            ready: false,
            finish_sent: false,
            backlog: Backlog::new(BACKLOG_SAMPLES),
            retiring: Vec::new(),
        }
    }

    /// Démarre une dictée : micro ouvert tout de suite, modèle chargé en parallèle (ou déjà chaud).
    pub fn start(&mut self, language: VoiceLanguage) -> Result<(), VoiceProblem> {
        if self.phase != Phase::Idle {
            return Err(VoiceProblem::Busy);
        }
        if !self.store.is_installed(language) {
            return Err(VoiceProblem::ModelMissing);
        }
        self.source.start(self.mic_tx.clone())?;
        self.prepare_worker(language);
        self.backlog.drain();
        self.finish_sent = false;
        self.phase = Phase::Listening;
        Ok(())
    }

    /// Réutilise le modèle chargé pour cette langue, sinon en charge un nouveau dans le fil.
    fn prepare_worker(&mut self, language: VoiceLanguage) {
        let warm = self.loaded == Some(language) && self.worker.is_some();
        if warm {
            self.ready = true;
            if let Some(worker) = &self.worker {
                worker.send(WorkerCommand::Begin);
            }
            return;
        }
        self.release_model();
        let dir = self.store.model_dir(language);
        self.worker = Some(RecognizerWorker::spawn(dir, self.msg_tx.clone()));
        self.loaded = Some(language);
        self.ready = false;
    }

    /// Arrête le micro tout de suite ; le texte restant est validé au prochain `poll` puis `Stopped` est annoncé.
    pub fn stop(&mut self) {
        if self.phase == Phase::Listening {
            self.source.stop();
            self.phase = Phase::Stopping;
        }
    }

    /// Annule : micro coupé, rien n'est écrit, le modèle est libéré.
    pub fn cancel(&mut self) {
        if self.phase != Phase::Idle {
            self.source.stop();
        }
        self.phase = Phase::Idle;
        self.release_model();
        while self.mic_rx.try_recv().is_ok() {}
    }

    /// Fermeture de l'application : annule tout et attend la libération des modèles.
    pub fn shutdown(&mut self) {
        self.cancel();
        for handle in self.retiring.drain(..) {
            let _ = handle.join();
        }
    }

    /// Libère le modèle chargé (garde expirée, annulation, changement de langue).
    pub fn release_model(&mut self) {
        self.retiring.retain(|handle| !handle.is_finished());
        if let Some(worker) = self.worker.take() {
            self.retiring.extend(worker.release());
        }
        let (msg_tx, msg_rx) = channel();
        self.msg_tx = msg_tx;
        self.msg_rx = msg_rx;
        self.loaded = None;
        self.ready = false;
    }

    /// Dit si aucune dictée n'est en cours.
    #[allow(dead_code)]
    pub fn is_idle(&self) -> bool {
        self.phase == Phase::Idle
    }

    /// Dit si un modèle est chargé (ou en cours de chargement).
    #[allow(dead_code)]
    pub fn model_loaded(&self) -> bool {
        self.worker.is_some()
    }

    /// Dit si le reconnaisseur a fini de charger son modèle.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Remplace la source audio (tests, changement de périphérique).
    #[allow(dead_code)]
    pub fn replace_source(&mut self, source: Box<dyn AudioSource>) {
        self.source = source;
    }
}

impl Drop for VoiceEngine {
    /// À la destruction, attend la libération des modèles pour ne jamais quitter pendant qu'un fil les utilise.
    fn drop(&mut self) {
        self.release_model();
        for handle in self.retiring.drain(..) {
            let _ = handle.join();
        }
    }
}
