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
** Tests de l'insertion du texte dicté : partiel remplacé, saisie respectée (T8, cas repris de Mira).
*/

use slate::voice::inserter::Inserter;
use slate::voice::text_target::StringTarget;

#[test]
fn test_partial_is_replaced_then_validated() {
    let mut t = StringTarget::new("Bonjour ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "je");
    assert_eq!(t.text(), "Bonjour je");
    ins.show_partial(&mut t, "je vous");
    assert_eq!(t.text(), "Bonjour je vous");
    ins.commit(&mut t, "je vous confirme.");
    assert_eq!(t.text(), "Bonjour je vous confirme. ");
    assert_eq!(t.cursor(), t.text().chars().count());
}

#[test]
fn test_blank_added_only_when_previous_char_is_not_blank() {
    let mut t = StringTarget::new("Bonjour", None);
    Inserter::default().show_partial(&mut t, "je");
    assert_eq!(t.text(), "Bonjour je");
    let mut empty = StringTarget::new("", None);
    Inserter::default().show_partial(&mut empty, "je");
    assert_eq!(empty.text(), "je");
}

#[test]
fn test_insertion_in_the_middle_keeps_surroundings() {
    let mut t = StringTarget::new("Salut  fin", Some(6));
    Inserter::default().show_partial(&mut t, "toi");
    assert_eq!(t.text(), "Salut toi fin");
    assert_eq!(t.cursor(), "Salut toi".chars().count());
}

#[test]
fn test_non_empty_selection_is_replaced_once() {
    let mut t = StringTarget::new("un mot ici", None);
    t.set_selection(3, 6);
    Inserter::default().show_partial(&mut t, "nom");
    assert_eq!(t.text(), "un nom ici");
}

#[test]
fn test_typing_during_a_partial_is_kept() {
    let mut t = StringTarget::new("A ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "bc");
    assert_eq!(t.text(), "A bc");
    t.set_value("A bcX", 5);
    ins.show_partial(&mut t, "de");
    assert_eq!(t.text(), "A bcX de");
}

#[test]
fn test_cumulative_partial_is_not_rewritten_after_typing() {
    let mut t = StringTarget::new("A ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "bc");
    t.set_value("A bcX", 5);
    ins.show_partial(&mut t, "bc de");
    assert_eq!(t.text(), "A bcX de");
    ins.commit(&mut t, "bc de.");
    assert_eq!(t.text(), "A bcX de. ");
}

#[test]
fn test_moving_the_cursor_reanchors_the_rest() {
    let mut t = StringTarget::new("Un deux ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "trois");
    t.set_selection(0, 0);
    ins.show_partial(&mut t, "quatre");
    assert!(t.text().starts_with("quatre"));
    assert!(t.text().contains("Un deux trois"));
}

#[test]
fn test_release_validates_partial_and_stops_touching_the_field() {
    let mut t = StringTarget::new("A ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "bc");
    ins.release();
    ins.show_partial(&mut t, "zzz");
    ins.commit(&mut t, "zzz");
    assert_eq!(t.text(), "A bc");
}

#[test]
fn test_empty_commit_removes_partial_without_leaving_a_blank() {
    let mut t = StringTarget::new("A", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "bc");
    ins.commit(&mut t, "");
    assert_eq!(t.text(), "A");
}

#[test]
fn test_selection_made_during_dictation_is_never_replaced() {
    let mut t = StringTarget::new("Texte important. ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "je");
    t.set_selection(0, 15);
    ins.show_partial(&mut t, "vous");
    assert!(t.text().contains("Texte important"));
    assert!(t.text().contains("vous"));
}

#[test]
fn test_next_sentence_is_whole_after_typing_then_sentence_end() {
    let mut t = StringTarget::new("A ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "Merci");
    t.set_value("A MerciX", 8);
    ins.commit(&mut t, "Merci");
    ins.show_partial(&mut t, "Merci pour votre retour");
    assert!(t.text().contains("Merci pour votre retour"));
}

#[test]
fn test_accented_text_uses_character_offsets() {
    let mut t = StringTarget::new("Été ", None);
    let mut ins = Inserter::default();
    ins.show_partial(&mut t, "à la");
    ins.commit(&mut t, "à la plage.");
    assert_eq!(t.text(), "Été à la plage. ");
}
