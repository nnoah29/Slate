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
** Pompe du moteur : route l'audio vers le reconnaisseur ou la réserve et traduit ses messages.
*/

use super::audio_level::audio_level;
use super::engine::{EngineEvent, Phase, VoiceEngine};
use super::microphone::MicMessage;
use super::recognizer_worker::{WorkerCommand, WorkerMessage};
use super::types::VoiceProblem;

impl VoiceEngine {
    /// À appeler souvent depuis la boucle de l'interface : renvoie les événements survenus depuis le dernier appel.
    pub fn poll(&mut self) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        self.drain_microphone(&mut events);
        self.drain_worker(&mut events);
        self.finish_if_due(&mut events);
        events
    }

    /// Envoie un paquet au reconnaisseur s'il est prêt, sinon le met en réserve.
    fn route_audio(&mut self, samples: Vec<f32>) {
        match (&self.worker, self.ready) {
            (Some(worker), true) => {
                worker.send(WorkerCommand::Audio(samples));
            }
            _ => self.backlog.push(samples),
        }
    }

    /// Lit les paquets du micro ; une fin de flux inattendue arrête la dictée avec une erreur.
    fn drain_microphone(&mut self, events: &mut Vec<EngineEvent>) {
        while let Ok(message) = self.mic_rx.try_recv() {
            match message {
                MicMessage::Samples(samples) => {
                    if self.phase == Phase::Listening {
                        events.push(EngineEvent::Level(audio_level(&samples)));
                    }
                    self.route_audio(samples);
                }
                MicMessage::Ended if self.phase == Phase::Listening => {
                    let problem = VoiceProblem::MicrophoneUnavailable("flux interrompu".into());
                    self.fail(problem, events);
                }
                MicMessage::Ended => {}
            }
        }
    }

    /// Traduit les messages du fil de reconnaissance en événements pour l'interface.
    fn drain_worker(&mut self, events: &mut Vec<EngineEvent>) {
        while let Ok(message) = self.msg_rx.try_recv() {
            match message {
                WorkerMessage::Ready => self.on_ready(),
                WorkerMessage::Event(event) => events.push(EngineEvent::Text(event)),
                WorkerMessage::Finished => {
                    self.phase = Phase::Idle;
                    events.push(EngineEvent::Stopped);
                }
                WorkerMessage::Failed(problem) => self.fail(problem, events),
                WorkerMessage::Released => {}
            }
        }
    }

    /// Modèle chargé : rejoue la réserve dans l'ordre.
    fn on_ready(&mut self) {
        self.ready = true;
        for chunk in self.backlog.drain() {
            if let Some(worker) = &self.worker {
                worker.send(WorkerCommand::Audio(chunk));
            }
        }
    }

    /// Une fois le micro coupé et le modèle prêt, demande au reconnaisseur de valider la fin de la parole.
    fn finish_if_due(&mut self, events: &mut Vec<EngineEvent>) {
        if self.phase != Phase::Stopping || !self.ready || self.finish_sent {
            return;
        }
        let sent = self
            .worker
            .as_ref()
            .is_some_and(|w| w.send(WorkerCommand::Finish));
        self.finish_sent = sent;
        if !sent {
            self.phase = Phase::Idle;
            events.push(EngineEvent::Stopped);
        }
    }

    /// Échec : micro coupé, modèle libéré, l'interface est prévenue.
    fn fail(&mut self, problem: VoiceProblem, events: &mut Vec<EngineEvent>) {
        self.source.stop();
        self.release_model();
        self.phase = Phase::Idle;
        events.push(EngineEvent::Failed(problem));
    }
}
