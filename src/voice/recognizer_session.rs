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
** Session de reconnaissance en flux : un modèle chargé, un flux audio, le dernier texte annoncé.
*/

use super::recognizer_config::recognizer_config_for;
use super::transcript::{finish_events, next_events};
use super::types::{RecognizerEvent, VoiceProblem};
use sherpa_onnx::{OnlineRecognizer, OnlineStream};
use std::path::Path;

const MODEL_FILES: [&str; 4] = ["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"];
const SAMPLE_RATE: i32 = 16_000;
const TAIL_SILENCE: usize = 6_400;

/// Un reconnaisseur en flux et le dernier texte provisoire déjà annoncé.
pub struct Session {
    recognizer: OnlineRecognizer,
    stream: OnlineStream,
    last: String,
}

impl Session {
    /// Charge le modèle du dossier donné (encoder, decoder, joiner, tokens).
    pub fn open(model_dir: &Path) -> Result<Self, VoiceProblem> {
        let complete = MODEL_FILES.iter().all(|f| model_dir.join(f).is_file());
        if !complete {
            return Err(VoiceProblem::ModelMissing);
        }
        let config = recognizer_config_for(model_dir);
        let recognizer = OnlineRecognizer::create(&config)
            .ok_or_else(|| VoiceProblem::RecognizerFailed("modèle illisible".into()))?;
        let stream = recognizer.create_stream();
        Ok(Self {
            recognizer,
            stream,
            last: String::new(),
        })
    }

    /// Décode tout ce que le modèle peut décoder avec l'audio reçu.
    fn decode_available(&self) {
        while self.recognizer.is_ready(&self.stream) {
            self.recognizer.decode(&self.stream);
        }
    }

    /// Texte courant de l'hypothèse du reconnaisseur.
    fn current_text(&self) -> String {
        self.recognizer
            .get_result(&self.stream)
            .map(|r| r.text)
            .unwrap_or_default()
    }

    /// Décode un paquet d'échantillons ; annonce le texte s'il a changé, le valide en fin de phrase.
    pub fn accept(&mut self, samples: &[f32], send: &mut dyn FnMut(RecognizerEvent)) {
        self.stream.accept_waveform(SAMPLE_RATE, samples);
        self.decode_available();
        let endpoint = self.recognizer.is_endpoint(&self.stream);
        let (events, last) = next_events(&self.last, &self.current_text(), endpoint);
        events.into_iter().for_each(&mut *send);
        if endpoint {
            self.recognizer.reset(&self.stream);
        }
        self.last = last;
    }

    /// Fin de la parole : vide le modèle avec un court silence, valide le reste, repart d'un flux neuf.
    pub fn finish(&mut self, send: &mut dyn FnMut(RecognizerEvent)) {
        self.stream
            .accept_waveform(SAMPLE_RATE, &[0.0; TAIL_SILENCE]);
        self.stream.input_finished();
        self.decode_available();
        finish_events(&self.current_text())
            .into_iter()
            .for_each(&mut *send);
        self.stream = self.recognizer.create_stream();
        self.last.clear();
    }

    /// Nouvelle dictée sur le modèle déjà chargé.
    pub fn restart(&mut self) {
        self.stream = self.recognizer.create_stream();
        self.last.clear();
    }
}
