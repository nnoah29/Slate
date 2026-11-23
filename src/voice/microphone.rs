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
** Micro par `parecord` (PulseAudio/PipeWire-pulse) : processus enfant, flux PCM 16 kHz lu et converti.
*/

use super::pcm::Pcm16Decoder;
use super::types::VoiceProblem;
use std::io::{ErrorKind, Read};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

const PARECORD_ARGS: [&str; 5] = [
    "--raw",
    "--format=s16le",
    "--rate=16000",
    "--channels=1",
    "--latency-msec=60",
];

/// Messages du micro : paquets d'échantillons dans [-1, 1], puis la fin du flux.
#[derive(Debug, PartialEq)]
pub enum MicMessage {
    Samples(Vec<f32>),
    Ended,
}

/// Micro ouvert : le processus de capture et le fil qui lit sa sortie.
pub struct Microphone {
    child: Child,
    reader: Option<JoinHandle<()>>,
}

impl Microphone {
    /// Ouvre le micro par défaut avec `parecord`.
    pub fn start(out: Sender<MicMessage>) -> Result<Self, VoiceProblem> {
        Self::start_with("parecord", &PARECORD_ARGS, out)
    }

    /// Lance un programme de capture quelconque qui écrit du PCM 16 bits mono 16 kHz sur sa sortie.
    pub fn start_with(
        program: &str,
        args: &[&str],
        out: Sender<MicMessage>,
    ) -> Result<Self, VoiceProblem> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(spawn_problem)?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| VoiceProblem::MicrophoneUnavailable("sortie absente".into()))?;
        let reader = thread::spawn(move || read_loop(stdout, &out));
        Ok(Self {
            child,
            reader: Some(reader),
        })
    }

    /// Coupe le micro tout de suite : tue le processus puis attend la fin de la lecture.
    pub fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

/// Traduit l'échec de lancement : programme absent ou autre cause.
fn spawn_problem(err: std::io::Error) -> VoiceProblem {
    if err.kind() == ErrorKind::NotFound {
        return VoiceProblem::MicrophoneMissing;
    }
    VoiceProblem::MicrophoneUnavailable(err.to_string())
}

/// Lit le flux, convertit en flottants et transmet ; signale la fin à la sortie ou si plus personne n'écoute.
fn read_loop(mut stdout: ChildStdout, out: &Sender<MicMessage>) {
    let mut decoder = Pcm16Decoder::default();
    let mut buffer = [0u8; 3200];
    while let Ok(read) = stdout.read(&mut buffer) {
        if read == 0 {
            break;
        }
        let samples = decoder.add(&buffer[..read]);
        if !samples.is_empty() && out.send(MicMessage::Samples(samples)).is_err() {
            return;
        }
    }
    let _ = out.send(MicMessage::Ended);
}
