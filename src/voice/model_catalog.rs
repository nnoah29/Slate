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
** Modèles de reconnaissance pinnés (Kroko, CC-BY-SA) : source, taille et empreinte par langue.
*/

use super::types::VoiceLanguage;

/// Un modèle téléchargeable : archive à récupérer et fichiers à en garder.
#[derive(Debug, Clone, Copy)]
pub struct ModelEntry {
    #[allow(dead_code)]
    pub language: VoiceLanguage,
    pub url: &'static str,
    pub archive_bytes: u64,
    pub sha256: &'static str,
    pub folder: &'static str,
    pub files: [&'static str; 4],
}

const FILES: [&str; 4] = ["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"];

const FRENCH: ModelEntry = ModelEntry {
    language: VoiceLanguage::Fr,
    url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-fr-kroko-2025-08-06.tar.bz2",
    archive_bytes: 57_220_361,
    sha256: "e6ffd3dc43725cd6c8137b05c739f15607d0df946b9b90eb141e10059efca024",
    folder: "sherpa-onnx-streaming-zipformer-fr-kroko-2025-08-06",
    files: FILES,
};

const ENGLISH: ModelEntry = ModelEntry {
    language: VoiceLanguage::En,
    url: "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06.tar.bz2",
    archive_bytes: 57_267_600,
    sha256: "c8676e5ff9ac2a85296e53ee0fd4d5fb1db6770e7a7647166eeafe349ade6834",
    folder: "sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06",
    files: FILES,
};

/// Modèle pinné associé à une langue.
pub fn entry_for(language: VoiceLanguage) -> &'static ModelEntry {
    match language {
        VoiceLanguage::Fr => &FRENCH,
        VoiceLanguage::En => &ENGLISH,
    }
}
