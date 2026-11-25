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
** Tests de la conversion PCM 16 bits du micro et du niveau sonore (T2).
*/

use slate::voice::audio_level::audio_level;
use slate::voice::pcm::Pcm16Decoder;

fn bytes(samples: &[i16]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

#[test]
fn test_decoder_converts_to_unit_range() {
    let mut decoder = Pcm16Decoder::default();
    let out = decoder.add(&bytes(&[0, 16384, -16384, i16::MIN]));
    assert_eq!(out, vec![0.0, 0.5, -0.5, -1.0]);
}

#[test]
fn test_decoder_keeps_orphan_byte_for_next_chunk() {
    let mut decoder = Pcm16Decoder::default();
    let all = bytes(&[1000, -2000, 3000]);
    let mut out = decoder.add(&all[..3]);
    out.extend(decoder.add(&all[3..]));
    let expected: Vec<f32> = [1000i16, -2000, 3000]
        .iter()
        .map(|s| f32::from(*s) / 32768.0)
        .collect();
    assert_eq!(out, expected);
}

#[test]
fn test_decoder_handles_empty_and_single_byte_chunks() {
    let mut decoder = Pcm16Decoder::default();
    assert!(decoder.add(&[]).is_empty());
    assert!(decoder.add(&[0x10]).is_empty());
    assert_eq!(decoder.add(&[0x00]), vec![16.0 / 32768.0]);
}

#[test]
fn test_audio_level_silence_is_zero() {
    assert_eq!(audio_level(&[]), 0.0);
    assert_eq!(audio_level(&[0.0; 1600]), 0.0);
}

#[test]
fn test_audio_level_loud_voice_saturates_and_is_monotonic() {
    assert_eq!(audio_level(&[0.9; 1600]), 1.0);
    let quiet = audio_level(&[0.01; 1600]);
    let medium = audio_level(&[0.05; 1600]);
    assert!(quiet > 0.0 && quiet < medium && medium < 1.0);
}
