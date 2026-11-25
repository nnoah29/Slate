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
** Tests du filtrage de la palette de commandes (insensible à la casse et aux accents).
*/

use slate::discovery::matches_query;

#[test]
fn test_query_ignores_case_and_accents() {
    assert!(matches_query("Aperçu / Édition", "apercu"));
    assert!(matches_query("Aperçu / Édition", "EDITION"));
    assert!(matches_query("Titre (niveau suivant)", "niv"));
}

#[test]
fn test_empty_query_matches_everything() {
    assert!(matches_query("Gras", ""));
    assert!(matches_query("Gras", "   "));
}

#[test]
fn test_query_words_may_appear_in_any_order() {
    assert!(matches_query("Rechercher et remplacer", "remplacer rech"));
}

#[test]
fn test_non_matching_query_is_rejected() {
    assert!(!matches_query("Gras", "italique"));
}
