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
** Caractère-bloc de l'indicateur de dictée : un seul glyphe dont la hauteur suit le niveau sonore.
*/

const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Bloc correspondant à un niveau sonore de 0 à 1 (hors plage ramené dans les bornes).
pub fn glyph_for_level(level: f32) -> char {
    let clamped = if level.is_nan() {
        0.0
    } else {
        level.clamp(0.0, 1.0)
    };
    let index = (clamped * (BLOCKS.len() - 1) as f32).round() as usize;
    BLOCKS[index]
}
