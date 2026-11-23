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
** Texte reconnu : normalisation et dérivation des événements provisoires et définitifs.
*/

use super::types::RecognizerEvent;

/// Normalise une phrase reconnue : apostrophes droites, blancs multiples réduits, bords retirés.
pub fn normalize_transcript(raw: &str) -> String {
    let straight = raw.replace('’', "'");
    straight.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Événements à annoncer après un paquet d'audio, et nouvelle mémoire du dernier texte.
pub fn next_events(last: &str, current: &str, endpoint: bool) -> (Vec<RecognizerEvent>, String) {
    let mut events = Vec::new();
    let text = normalize_transcript(current);
    if current != last {
        events.push(RecognizerEvent::Partial(text.clone()));
    }
    if !endpoint {
        return (events, current.to_string());
    }
    if !text.is_empty() {
        events.push(RecognizerEvent::Final(text));
    }
    (events, String::new())
}

/// Événement final de fin de parole : le texte restant, s'il y en a.
pub fn finish_events(current: &str) -> Vec<RecognizerEvent> {
    let text = normalize_transcript(current);
    if text.is_empty() {
        return Vec::new();
    }
    vec![RecognizerEvent::Final(text)]
}
