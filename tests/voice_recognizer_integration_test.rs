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
** Test d'intégration optionnel du reconnaisseur réel (T5) : exige SLATE_VOICE_MODELS et SLATE_VOICE_WAV_FR (enregistrement français du modèle Kroko).
*/

use sherpa_onnx::Wave;
use slate::voice::model_catalog::entry_for;
use slate::voice::recognizer_worker::{RecognizerWorker, WorkerCommand, WorkerMessage};
use slate::voice::types::{RecognizerEvent, VoiceLanguage};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

fn models_root() -> Option<PathBuf> {
    std::env::var_os("SLATE_VOICE_MODELS").map(PathBuf::from)
}

/// Charge l'enregistrement français de test (16 kHz) indiqué par SLATE_VOICE_WAV_FR.
fn french_recording() -> Option<Vec<f32>> {
    let path = std::env::var("SLATE_VOICE_WAV_FR").ok()?;
    let wave = Wave::read(&path).filter(|w| w.sample_rate() == 16_000)?;
    Some(wave.samples().to_vec())
}

fn collect_until_finished(messages: &Receiver<WorkerMessage>) -> Vec<RecognizerEvent> {
    let mut events = Vec::new();
    while let Ok(message) = messages.recv_timeout(Duration::from_secs(60)) {
        match message {
            WorkerMessage::Event(event) => events.push(event),
            WorkerMessage::Finished | WorkerMessage::Failed(_) => break,
            _ => {}
        }
    }
    events
}

fn dictate(language: VoiceLanguage, samples: &[f32]) -> Option<Vec<RecognizerEvent>> {
    let model_dir = models_root()?.join(entry_for(language).folder);
    let (out, messages) = channel();
    let worker = RecognizerWorker::spawn(model_dir, out);
    for chunk in samples.chunks(1600) {
        worker.send(WorkerCommand::Audio(chunk.to_vec()));
    }
    worker.send(WorkerCommand::Finish);
    let events = collect_until_finished(&messages);
    worker.shutdown();
    Some(events)
}

fn finals(events: &[RecognizerEvent]) -> String {
    let texts = events.iter().filter_map(|e| match e {
        RecognizerEvent::Final(text) => Some(text.as_str()),
        RecognizerEvent::Partial(_) => None,
    });
    texts.collect::<Vec<_>>().join(" ").to_lowercase()
}

#[test]
fn test_french_sentence_is_recognized_live_then_validated() {
    let Some(samples) = french_recording() else {
        eprintln!("ignoré : SLATE_VOICE_WAV_FR absent");
        return;
    };
    let Some(events) = dictate(VoiceLanguage::Fr, &samples) else {
        eprintln!("ignoré : SLATE_VOICE_MODELS absent");
        return;
    };
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RecognizerEvent::Partial(_)))
    );
    let text = finals(&events);
    assert!(text.contains("demandez"), "texte reconnu : {text}");
    assert!(text.contains("pays"), "texte reconnu : {text}");
    assert!(text.trim_end().ends_with("faire"), "texte reconnu : {text}");
}

#[test]
fn test_silence_gives_no_text_and_no_panic() {
    let Some(model_root) = models_root() else {
        eprintln!("ignoré : SLATE_VOICE_MODELS absent");
        return;
    };
    let (out, messages) = channel();
    let worker = RecognizerWorker::spawn(model_root.join(entry_for(VoiceLanguage::Fr).folder), out);
    for _ in 0..30 {
        worker.send(WorkerCommand::Audio(vec![0.0; 1600]));
    }
    worker.send(WorkerCommand::Finish);
    let events = collect_until_finished(&messages);
    worker.shutdown();
    assert!(finals(&events).trim().is_empty());
}

#[test]
fn test_missing_model_reports_failure_instead_of_panicking() {
    let (out, messages) = channel();
    let worker = RecognizerWorker::spawn(PathBuf::from("/nonexistent/model"), out);
    let first = messages
        .recv_timeout(Duration::from_secs(30))
        .expect("message");
    assert_eq!(
        first,
        WorkerMessage::Failed(slate::voice::types::VoiceProblem::ModelMissing)
    );
    worker.shutdown();
}
