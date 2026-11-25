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
** Réglage du reconnaisseur sherpa-onnx : fichiers du modèle, architecture, fils, fin de phrase.
*/

use sherpa_onnx::{OnlineModelConfig, OnlineRecognizerConfig, OnlineTransducerModelConfig};
use std::path::Path;

const ARCHITECTURE: &str = "zipformer2";

/// Chemin d'un fichier du modèle sous forme de chaîne.
fn file(dir: &Path, name: &str) -> Option<String> {
    Some(dir.join(name).display().to_string())
}

/// Réglage pour un dossier de modèle Kroko ; le type donné évite un premier chargement complet pour le deviner.
pub fn recognizer_config_for(model_dir: &Path) -> OnlineRecognizerConfig {
    let model = OnlineModelConfig {
        transducer: OnlineTransducerModelConfig {
            encoder: file(model_dir, "encoder.onnx"),
            decoder: file(model_dir, "decoder.onnx"),
            joiner: file(model_dir, "joiner.onnx"),
        },
        tokens: file(model_dir, "tokens.txt"),
        num_threads: 2,
        model_type: Some(ARCHITECTURE.to_string()),
        ..OnlineModelConfig::default()
    };
    OnlineRecognizerConfig {
        model_config: model,
        decoding_method: Some("greedy_search".to_string()),
        enable_endpoint: true,
        rule1_min_trailing_silence: 2.4,
        rule2_min_trailing_silence: 1.2,
        rule3_min_utterance_length: 20.0,
        ..OnlineRecognizerConfig::default()
    }
}
