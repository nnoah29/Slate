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
** Tests du titre à niveau direct (Ctrl+Alt+1 à 6).
*/

use slate::editor::markdown::set_heading_level;

#[test]
fn test_heading_level_adds_prefix_to_plain_line() {
    assert_eq!(set_heading_level("Bonjour", 2), "## Bonjour");
}

#[test]
fn test_heading_level_replaces_other_level() {
    assert_eq!(set_heading_level("### Bonjour", 1), "# Bonjour");
}

#[test]
fn test_heading_level_same_level_removes_heading() {
    assert_eq!(set_heading_level("## Bonjour", 2), "Bonjour");
}

#[test]
fn test_heading_level_keeps_indentation() {
    assert_eq!(set_heading_level("  texte", 3), "  ### texte");
}

#[test]
fn test_heading_level_out_of_range_is_ignored() {
    assert_eq!(set_heading_level("texte", 0), "texte");
    assert_eq!(set_heading_level("texte", 7), "texte");
}
