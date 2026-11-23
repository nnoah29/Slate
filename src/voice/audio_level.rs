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
** Niveau sonore d'un paquet d'échantillons, de 0 (silence) à 1 (voix forte), pour l'animation du micro.
*/

const FLOOR_DB: f32 = -50.0;
const RANGE_DB: f32 = 40.0;
const SILENCE_RMS: f32 = 1e-5;

/// Niveau perçu d'un paquet : valeur efficace en décibels ramenée sur l'échelle de 0 à 1.
pub fn audio_level(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let energy: f32 = samples.iter().map(|s| s * s).sum();
    let rms = (energy / samples.len() as f32).sqrt();
    if rms < SILENCE_RMS {
        return 0.0;
    }
    ((20.0 * rms.log10() - FLOOR_DB) / RANGE_DB).clamp(0.0, 1.0)
}
