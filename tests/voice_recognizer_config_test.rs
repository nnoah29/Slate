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
** Test du réglage du reconnaisseur sherpa-onnx pour un dossier de modèle Kroko (T5).
*/

use slate::voice::recognizer_config::recognizer_config_for;
use std::path::Path;

#[test]
fn test_config_points_to_the_four_model_files() {
    let config = recognizer_config_for(Path::new("/m/kroko"));
    let transducer = &config.model_config.transducer;
    assert_eq!(transducer.encoder.as_deref(), Some("/m/kroko/encoder.onnx"));
    assert_eq!(transducer.decoder.as_deref(), Some("/m/kroko/decoder.onnx"));
    assert_eq!(transducer.joiner.as_deref(), Some("/m/kroko/joiner.onnx"));
    assert_eq!(
        config.model_config.tokens.as_deref(),
        Some("/m/kroko/tokens.txt")
    );
}

#[test]
fn test_config_declares_architecture_threads_and_endpointing() {
    let config = recognizer_config_for(Path::new("/m/kroko"));
    assert_eq!(
        config.model_config.model_type.as_deref(),
        Some("zipformer2")
    );
    assert_eq!(config.model_config.num_threads, 2);
    assert!(config.enable_endpoint);
    assert_eq!(config.decoding_method.as_deref(), Some("greedy_search"));
}
