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
** Tests de l'invite d'enregistrement : pas d'invite pour une note sans nom restée vide.
*/

use slate::document::save_prompt::needs_save_prompt;

#[test]
fn untitled_empty_note_does_not_prompt() {
    assert!(!needs_save_prompt(true, false, ""));
    assert!(!needs_save_prompt(true, false, "  \n\t "));
}

#[test]
fn untitled_note_with_text_prompts() {
    assert!(needs_save_prompt(true, false, "bonjour"));
}

#[test]
fn saved_file_emptied_still_prompts() {
    assert!(needs_save_prompt(true, true, ""));
}

#[test]
fn clean_document_never_prompts() {
    assert!(!needs_save_prompt(false, true, "texte"));
    assert!(!needs_save_prompt(false, false, "texte"));
}
