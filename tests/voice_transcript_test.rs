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
** Tests de la normalisation du texte reconnu et de la dérivation des événements (T5).
*/

use slate::voice::transcript::{finish_events, next_events, normalize_transcript};
use slate::voice::types::RecognizerEvent::{Final, Partial};

#[test]
fn test_normalize_straightens_apostrophes_and_blanks() {
    assert_eq!(normalize_transcript("  c’est   l’été \n"), "c'est l'été");
    assert_eq!(normalize_transcript(""), "");
}

#[test]
fn test_changed_text_gives_a_partial() {
    let (events, last) = next_events("bonjour", "bonjour marie", false);
    assert_eq!(events, vec![Partial("bonjour marie".into())]);
    assert_eq!(last, "bonjour marie");
}

#[test]
fn test_unchanged_text_gives_nothing() {
    let (events, last) = next_events("bonjour", "bonjour", false);
    assert!(events.is_empty());
    assert_eq!(last, "bonjour");
}

#[test]
fn test_endpoint_validates_the_sentence_and_resets() {
    let (events, last) = next_events("bonjour", "bonjour marie.", true);
    assert_eq!(
        events,
        vec![
            Partial("bonjour marie.".into()),
            Final("bonjour marie.".into())
        ]
    );
    assert_eq!(last, "");
}

#[test]
fn test_endpoint_on_silence_gives_nothing() {
    let (events, last) = next_events("", "   ", true);
    assert!(events.is_empty());
    assert_eq!(last, "");
}

#[test]
fn test_finish_validates_remaining_text_only() {
    assert_eq!(finish_events("à demain"), vec![Final("à demain".into())]);
    assert!(finish_events("  ").is_empty());
}
