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
** Tests du caractère-bloc de l'indicateur : silence, saturation, bornes et monotonie.
*/

use slate::voice::indicator_glyph::glyph_for_level;

#[test]
fn silence_gives_lowest_block() {
    assert_eq!(glyph_for_level(0.0), '▁');
}

#[test]
fn saturation_gives_full_block() {
    assert_eq!(glyph_for_level(1.0), '█');
}

#[test]
fn out_of_range_and_nan_are_clamped() {
    assert_eq!(glyph_for_level(-3.0), '▁');
    assert_eq!(glyph_for_level(7.0), '█');
    assert_eq!(glyph_for_level(f32::NAN), '▁');
}

#[test]
fn glyph_never_decreases_with_level() {
    let mut previous = glyph_for_level(0.0);
    for step in 1..=100 {
        let current = glyph_for_level(step as f32 / 100.0);
        assert!(current >= previous);
        previous = current;
    }
}
