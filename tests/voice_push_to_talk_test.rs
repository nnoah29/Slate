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
** Tests de la machine à états d'Espace maintenu : appui court, maintien, autre touche, répétition (T9).
*/

use slate::voice::push_to_talk::{PttAction, PushToTalk};

#[test]
fn test_short_press_types_a_space_on_release() {
    let mut ptt = PushToTalk::default();
    assert_eq!(ptt.key_down(true, false), PttAction::Swallow);
    assert_eq!(ptt.key_up(), PttAction::InsertSpace);
    assert_eq!(ptt.key_up(), PttAction::PassThrough);
}

#[test]
fn test_hold_starts_dictation_and_release_stops_it() {
    let mut ptt = PushToTalk::default();
    ptt.key_down(true, false);
    assert_eq!(ptt.hold_elapsed(), PttAction::StartDictation);
    assert_eq!(ptt.key_down(true, true), PttAction::Swallow);
    assert_eq!(ptt.key_up(), PttAction::StopDictation);
    assert!(!ptt.is_active());
}

#[test]
fn test_hold_timer_after_early_release_does_nothing() {
    let mut ptt = PushToTalk::default();
    ptt.key_down(true, false);
    ptt.key_up();
    assert_eq!(ptt.hold_elapsed(), PttAction::PassThrough);
}

#[test]
fn test_another_key_flushes_the_pending_space_first() {
    let mut ptt = PushToTalk::default();
    ptt.key_down(true, false);
    assert_eq!(ptt.other_key(), PttAction::InsertSpace);
    assert_eq!(ptt.hold_elapsed(), PttAction::PassThrough);
    assert_eq!(ptt.key_down(true, true), PttAction::Swallow);
    assert_eq!(ptt.key_up(), PttAction::Swallow);
    assert!(!ptt.is_active());
}

#[test]
fn test_other_key_without_pending_space_passes_through() {
    let mut ptt = PushToTalk::default();
    assert_eq!(ptt.other_key(), PttAction::PassThrough);
}

#[test]
fn test_other_key_while_dictating_does_not_stop_it() {
    let mut ptt = PushToTalk::default();
    ptt.key_down(true, false);
    ptt.hold_elapsed();
    assert_eq!(ptt.other_key(), PttAction::PassThrough);
    assert_eq!(ptt.key_up(), PttAction::StopDictation);
}

#[test]
fn test_ineligible_space_behaves_like_a_normal_key() {
    let mut ptt = PushToTalk::default();
    assert_eq!(ptt.key_down(false, false), PttAction::PassThrough);
    assert_eq!(ptt.key_down(false, true), PttAction::PassThrough);
    assert_eq!(ptt.key_up(), PttAction::PassThrough);
    assert!(!ptt.is_active());
}

#[test]
fn test_cancel_resets_everything() {
    let mut ptt = PushToTalk::default();
    ptt.key_down(true, false);
    ptt.hold_elapsed();
    ptt.cancel();
    assert!(!ptt.is_active());
    assert_eq!(ptt.key_up(), PttAction::PassThrough);
}

#[test]
fn test_hold_delay_defaults_to_400_ms_and_can_be_changed() {
    assert_eq!(PushToTalk::default().hold_delay().as_millis(), 400);
    assert_eq!(PushToTalk::new(600).hold_delay().as_millis(), 600);
}
