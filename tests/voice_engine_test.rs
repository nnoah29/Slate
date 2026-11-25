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
** Tests du moteur de dictée avec un faux micro ; le test complet exige les modèles réels (T7).
*/

use sherpa_onnx::Wave;
use slate::voice::engine::{AudioSource, EngineEvent, VoiceEngine};
use slate::voice::microphone::MicMessage;
use slate::voice::model_store::ModelStore;
use slate::voice::types::{RecognizerEvent, VoiceLanguage, VoiceProblem};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// Faux micro : déverse des paquets prédéfinis dès l'ouverture, sans attendre le modèle.
struct FakeSource {
    chunks: Vec<Vec<f32>>,
    fail_with: Option<VoiceProblem>,
    stopped: Rc<Cell<u32>>,
}

impl AudioSource for FakeSource {
    fn start(&mut self, out: Sender<MicMessage>) -> Result<(), VoiceProblem> {
        if let Some(problem) = self.fail_with.clone() {
            return Err(problem);
        }
        self.chunks
            .drain(..)
            .for_each(|c| drop(out.send(MicMessage::Samples(c))));
        Ok(())
    }

    fn stop(&mut self) {
        self.stopped.set(self.stopped.get() + 1);
    }
}

fn source(chunks: Vec<Vec<f32>>, fail_with: Option<VoiceProblem>) -> (FakeSource, Rc<Cell<u32>>) {
    let stopped = Rc::new(Cell::new(0));
    (
        FakeSource {
            chunks,
            fail_with,
            stopped: stopped.clone(),
        },
        stopped,
    )
}

fn real_store() -> Option<ModelStore> {
    std::env::var_os("SLATE_VOICE_MODELS").map(|p| ModelStore::new(PathBuf::from(p)))
}

fn french_chunks() -> Option<Vec<Vec<f32>>> {
    let path = std::env::var("SLATE_VOICE_WAV_FR").ok()?;
    let wave = Wave::read(&path).filter(|w| w.sample_rate() == 16_000)?;
    Some(wave.samples().chunks(1600).map(<[f32]>::to_vec).collect())
}

fn poll_until_stopped(engine: &mut VoiceEngine) -> Vec<EngineEvent> {
    let (mut all, deadline) = (Vec::new(), Instant::now() + Duration::from_secs(60));
    while Instant::now() < deadline {
        let batch = engine.poll();
        let done = batch
            .iter()
            .any(|e| matches!(e, EngineEvent::Stopped | EngineEvent::Failed(_)));
        all.extend(batch);
        if done {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    all
}

#[test]
fn test_start_without_model_fails_before_opening_the_microphone() {
    let tmp = tempfile::tempdir().expect("tmp");
    let (fake, stopped) = source(vec![], None);
    let mut engine = VoiceEngine::new(ModelStore::new(tmp.path().to_path_buf()), Box::new(fake));
    assert_eq!(
        engine.start(VoiceLanguage::Fr),
        Err(VoiceProblem::ModelMissing)
    );
    assert!(engine.is_idle());
    assert_eq!(stopped.get(), 0);
}

#[test]
fn test_second_start_while_listening_is_refused() {
    let Some(store) = real_store() else {
        eprintln!("ignoré : SLATE_VOICE_MODELS absent");
        return;
    };
    let (fake, _) = source(vec![], None);
    let mut engine = VoiceEngine::new(store, Box::new(fake));
    engine.start(VoiceLanguage::Fr).expect("start");
    assert_eq!(engine.start(VoiceLanguage::Fr), Err(VoiceProblem::Busy));
    engine.cancel();
}

#[test]
fn test_microphone_failure_is_reported_and_leaves_engine_idle() {
    let Some(store) = real_store() else {
        eprintln!("ignoré : SLATE_VOICE_MODELS absent");
        return;
    };
    let (fake, _) = source(vec![], Some(VoiceProblem::MicrophoneMissing));
    let mut engine = VoiceEngine::new(store, Box::new(fake));
    assert_eq!(
        engine.start(VoiceLanguage::Fr),
        Err(VoiceProblem::MicrophoneMissing)
    );
    assert!(engine.is_idle());
    assert!(!engine.model_loaded());
}

#[test]
fn test_audio_spoken_before_model_is_ready_is_kept_and_stop_goes_to_the_end() {
    let (Some(store), Some(chunks)) = (real_store(), french_chunks()) else {
        eprintln!("ignoré : SLATE_VOICE_MODELS ou SLATE_VOICE_WAV_FR absents");
        return;
    };
    let (fake, stopped) = source(chunks, None);
    let mut engine = VoiceEngine::new(store, Box::new(fake));
    engine.start(VoiceLanguage::Fr).expect("start");
    engine.stop();
    assert_eq!(stopped.get(), 1);
    let events = poll_until_stopped(&mut engine);
    let text: String = events
        .iter()
        .filter_map(|e| match e {
            EngineEvent::Text(RecognizerEvent::Final(t)) => Some(t.to_lowercase()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains("demandez") && text.contains("pays"),
        "texte : {text}"
    );
    assert!(events.contains(&EngineEvent::Stopped));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::Level(l) if *l > 0.0))
    );
    assert!(engine.is_idle() && engine.model_loaded());
}

#[test]
fn test_second_dictation_reuses_the_loaded_model() {
    let (Some(store), Some(chunks)) = (real_store(), french_chunks()) else {
        eprintln!("ignoré : SLATE_VOICE_MODELS ou SLATE_VOICE_WAV_FR absents");
        return;
    };
    let (fake, _) = source(chunks.clone(), None);
    let mut engine = VoiceEngine::new(store, Box::new(fake));
    engine.start(VoiceLanguage::Fr).expect("start");
    engine.stop();
    poll_until_stopped(&mut engine);
    assert!(engine.model_loaded());
    let (fake, _) = source(vec![], None);
    engine.replace_source(Box::new(fake));
    assert_eq!(engine.start(VoiceLanguage::Fr), Ok(()));
    assert!(
        engine.is_ready(),
        "le modèle déjà chargé doit servir tout de suite"
    );
    engine.cancel();
    assert!(engine.is_idle());
    assert!(!engine.model_loaded());
}

#[test]
fn test_cancel_writes_nothing_and_stops_the_microphone() {
    let (Some(store), Some(chunks)) = (real_store(), french_chunks()) else {
        eprintln!("ignoré : SLATE_VOICE_MODELS ou SLATE_VOICE_WAV_FR absents");
        return;
    };
    let (fake, stopped) = source(chunks, None);
    let mut engine = VoiceEngine::new(store, Box::new(fake));
    engine.start(VoiceLanguage::Fr).expect("start");
    engine.cancel();
    assert_eq!(stopped.get(), 1);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        engine
            .poll()
            .iter()
            .all(|e| !matches!(e, EngineEvent::Text(_)))
    );
    assert!(engine.is_idle());
}
