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
** Tests des raccourcis mnémoniques, du catalogue d'actions et de la personnalisation.
*/

use gtk4::gdk::{Key, ModifierType};
use slate::shortcuts::{Action, ShortcutManager, all_actions};
use std::collections::HashMap;

const CTRL_SHIFT: ModifierType = ModifierType::CONTROL_MASK.union(ModifierType::SHIFT_MASK);
const CTRL_ALT: ModifierType = ModifierType::CONTROL_MASK.union(ModifierType::ALT_MASK);

#[test]
fn test_mnemonic_list_and_markup_shortcuts() {
    let mgr = ShortcutManager::new();
    assert_eq!(
        mgr.match_action(Key::L, CTRL_SHIFT),
        Some(Action::FormatBulletList)
    );
    assert_eq!(
        mgr.match_action(Key::N, CTRL_SHIFT),
        Some(Action::FormatNumberedList)
    );
    assert_eq!(
        mgr.match_action(Key::M, CTRL_SHIFT),
        Some(Action::ToggleConceal)
    );
}

#[test]
fn test_legacy_shortcuts_remain_as_aliases() {
    let mgr = ShortcutManager::new();
    assert_eq!(
        mgr.match_action(Key::l, ModifierType::CONTROL_MASK),
        Some(Action::ToggleConceal)
    );
    assert_eq!(
        mgr.match_action(Key::_8, CTRL_SHIFT),
        Some(Action::FormatBulletList)
    );
    assert_eq!(
        mgr.match_action(Key::H, CTRL_SHIFT),
        Some(Action::FormatHeading)
    );
}

#[test]
fn test_help_and_palette_shortcuts() {
    let mgr = ShortcutManager::new();
    assert_eq!(
        mgr.match_action(Key::F1, ModifierType::empty()),
        Some(Action::ShowShortcuts)
    );
    assert_eq!(
        mgr.match_action(Key::p, ModifierType::CONTROL_MASK),
        Some(Action::CommandPalette)
    );
}

#[test]
fn test_heading_levels_match_physical_keys_whatever_the_layout() {
    let mgr = ShortcutManager::new();
    for (code, level) in (10u32..=15).zip(1u8..=6) {
        assert_eq!(
            mgr.match_action_with_keycode(Key::ampersand, code, CTRL_ALT),
            Some(Action::Heading(level))
        );
    }
    assert_eq!(
        mgr.match_action_with_keycode(Key::minus, 15, ModifierType::CONTROL_MASK),
        Some(Action::ZoomOut)
    );
}

#[test]
fn test_defaults_have_no_conflicts() {
    assert!(ShortcutManager::new().check_conflicts().is_empty());
}

#[test]
fn test_accelerator_for_returns_gtk_name() {
    let mgr = ShortcutManager::new();
    assert_eq!(
        mgr.accelerator_for(Action::Save).as_deref(),
        Some("<Control>s")
    );
    assert_eq!(
        mgr.accelerator_for(Action::FormatBulletList).as_deref(),
        Some("<Control><Shift>l")
    );
    assert_eq!(
        mgr.accelerator_for(Action::Heading(2)).as_deref(),
        Some("<Control><Alt>2")
    );
}

#[test]
fn test_every_action_has_id_label_and_a_binding() {
    let mgr = ShortcutManager::new();
    for action in all_actions() {
        assert!(!action.label().is_empty(), "{action:?}");
        assert_eq!(Action::from_id(action.id()), Some(action), "{action:?}");
        assert!(
            mgr.accelerator_for(action).is_some(),
            "{action:?} sans raccourci"
        );
    }
}

#[test]
fn test_custom_shortcut_replaces_default() {
    let mut mgr = ShortcutManager::new();
    let custom = HashMap::from([("save".to_string(), "<Control><Shift>k".to_string())]);
    mgr.apply_custom(&custom);
    assert_eq!(mgr.match_action(Key::K, CTRL_SHIFT), Some(Action::Save));
    assert_eq!(mgr.match_action(Key::s, ModifierType::CONTROL_MASK), None);
}

#[test]
fn test_invalid_custom_shortcuts_are_ignored() {
    let mut mgr = ShortcutManager::new();
    let custom = HashMap::from([
        ("inconnue".to_string(), "<Control>j".to_string()),
        ("save".to_string(), "n'importe quoi".to_string()),
    ]);
    mgr.apply_custom(&custom);
    assert_eq!(
        mgr.match_action(Key::s, ModifierType::CONTROL_MASK),
        Some(Action::Save)
    );
}

#[test]
fn test_dictation_toggle_shortcut_and_catalog_entry() {
    let mgr = ShortcutManager::new();
    assert_eq!(
        mgr.match_action(Key::d, ModifierType::CONTROL_MASK),
        Some(Action::ToggleDictation)
    );
    assert_eq!(
        Action::from_id("toggle_dictation"),
        Some(Action::ToggleDictation)
    );
    assert_eq!(Action::ToggleDictation.label(), "Dictée vocale");
}
