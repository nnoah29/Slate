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
** Tests du micro avec de faux programmes de capture (aucun micro réel) (T6).
*/

use slate::voice::microphone::{MicMessage, Microphone};
use slate::voice::types::VoiceProblem;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

const WAIT: Duration = Duration::from_secs(5);

#[test]
fn test_samples_are_decoded_then_stream_ends() {
    let (tx, rx) = channel();
    let mic =
        Microphone::start_with("sh", &["-c", "printf '\\000\\100\\000\\300'"], tx).expect("start");
    assert_eq!(
        rx.recv_timeout(WAIT),
        Ok(MicMessage::Samples(vec![0.5, -0.5]))
    );
    assert_eq!(rx.recv_timeout(WAIT), Ok(MicMessage::Ended));
    mic.stop();
}

#[test]
fn test_missing_capture_program_is_reported() {
    let (tx, _rx) = channel();
    let result = Microphone::start_with("slate-programme-inexistant", &[], tx);
    assert_eq!(result.err(), Some(VoiceProblem::MicrophoneMissing));
}

#[test]
fn test_stop_kills_the_capture_process_immediately() {
    let (tx, rx) = channel();
    let mic = Microphone::start_with("sleep", &["30"], tx).expect("start");
    let began = Instant::now();
    mic.stop();
    assert!(began.elapsed() < Duration::from_secs(3));
    assert_eq!(rx.recv_timeout(WAIT), Ok(MicMessage::Ended));
}
