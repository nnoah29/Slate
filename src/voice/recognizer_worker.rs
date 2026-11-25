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
** Fil dédié à la reconnaissance : commandes en entrée, messages en sortie, l'interface ne fige jamais.
*/

use super::recognizer_session::Session;
use super::types::{RecognizerEvent, VoiceProblem};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

/// Commandes envoyées au fil de reconnaissance.
#[derive(Debug)]
pub enum WorkerCommand {
    Audio(Vec<f32>),
    Begin,
    Finish,
    Release,
}

/// Messages renvoyés par le fil de reconnaissance.
#[derive(Debug, PartialEq)]
pub enum WorkerMessage {
    Ready,
    Event(RecognizerEvent),
    Finished,
    Released,
    Failed(VoiceProblem),
}

/// Poignée sur le fil de reconnaissance ; le modèle se charge dans le fil, pas chez l'appelant.
pub struct RecognizerWorker {
    commands: Sender<WorkerCommand>,
    handle: Option<JoinHandle<()>>,
}

impl RecognizerWorker {
    /// Démarre le fil et lance le chargement du modèle ; `Ready` ou `Failed` arrive sur `messages`.
    pub fn spawn(model_dir: PathBuf, messages: Sender<WorkerMessage>) -> Self {
        let (commands, inbox) = channel();
        let handle = thread::spawn(move || worker_main(&model_dir, &inbox, &messages));
        Self {
            commands,
            handle: Some(handle),
        }
    }

    /// Envoie une commande ; faux si le fil est déjà terminé.
    pub fn send(&self, command: WorkerCommand) -> bool {
        self.commands.send(command).is_ok()
    }

    /// Demande la libération du modèle sans attendre ; rend le fil pour qu'on puisse le joindre plus tard.
    pub fn release(mut self) -> Option<JoinHandle<()>> {
        let _ = self.commands.send(WorkerCommand::Release);
        self.handle.take()
    }

    /// Libère le modèle puis attend la fin du fil.
    #[allow(dead_code)]
    pub fn shutdown(mut self) {
        let _ = self.commands.send(WorkerCommand::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Boucle du fil : charge le modèle puis traite les commandes jusqu'à `Release` ou la fermeture du canal.
fn worker_main(
    model_dir: &std::path::Path,
    inbox: &Receiver<WorkerCommand>,
    out: &Sender<WorkerMessage>,
) {
    let mut session = match Session::open(model_dir) {
        Ok(session) => session,
        Err(problem) => {
            let _ = out.send(WorkerMessage::Failed(problem));
            return;
        }
    };
    let _ = out.send(WorkerMessage::Ready);
    let mut emit = |event| {
        let _ = out.send(WorkerMessage::Event(event));
    };
    for command in inbox {
        match command {
            WorkerCommand::Audio(samples) => session.accept(&samples, &mut emit),
            WorkerCommand::Begin => {
                session.restart();
                let _ = out.send(WorkerMessage::Ready);
            }
            WorkerCommand::Finish => {
                session.finish(&mut emit);
                let _ = out.send(WorkerMessage::Finished);
            }
            WorkerCommand::Release => break,
        }
    }
    let _ = out.send(WorkerMessage::Released);
}
