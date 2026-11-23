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
** Tests des langues et des problèmes de dictée (T2).
*/

use slate::voice::types::{VoiceLanguage, VoiceProblem};

#[test]
fn test_language_codes_round_trip() {
    for language in [VoiceLanguage::Fr, VoiceLanguage::En] {
        assert_eq!(VoiceLanguage::from_code(language.code()), Some(language));
    }
    assert_eq!(VoiceLanguage::from_code("de"), None);
}

#[test]
fn test_language_from_locale_defaults_to_french() {
    assert_eq!(VoiceLanguage::from_locale("en_US.UTF-8"), VoiceLanguage::En);
    assert_eq!(VoiceLanguage::from_locale("fr_FR"), VoiceLanguage::Fr);
    assert_eq!(VoiceLanguage::from_locale("pt_BR"), VoiceLanguage::Fr);
}

#[test]
fn test_language_serializes_as_lowercase_code() {
    let json = serde_json::to_string(&VoiceLanguage::En).expect("json");
    assert_eq!(json, "\"en\"");
    let back: VoiceLanguage = serde_json::from_str("\"fr\"").expect("parse");
    assert_eq!(back, VoiceLanguage::Fr);
}

#[test]
fn test_problems_have_french_messages() {
    assert!(VoiceProblem::ModelMissing.to_string().contains("modèle"));
    assert!(
        VoiceProblem::MicrophoneMissing
            .to_string()
            .contains("Micro introuvable")
    );
    assert!(
        VoiceProblem::RecognizerFailed("x".into())
            .to_string()
            .contains('x')
    );
}
