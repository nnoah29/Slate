/*
**  _                                              _      ___    ___
** | |                                            | |    |__ \  / _ \
** | |_Created _       _ __   _ __    ___    __ _ | |__     ) || (_) |
** | '_ \ | | | |     | '_ \ | '_ \  / _ \  / _` || '_ \   / /  \__, |
** | |_) || |_| |     | | | || | | || (_) || (_| || | | | / /_    / /
** |_.__/  \__, |     |_| |_||_| |_| \___/  \__,_||_| |_||____|  /_/
**          __/ |     on 2026-09-22.
**         |___/
**
** Unit tests for shortcut key matching, defaults, and conflict verification.
*/

use gtk4::gdk::{Key, ModifierType};
use slate::shortcuts::{Action, KeyBinding, ShortcutManager};

#[test]
fn test_shortcut_binding_matches() {
    let binding = KeyBinding::new(Key::s, true, false, false);
    assert!(binding.matches(Key::s, ModifierType::CONTROL_MASK));
    assert!(binding.matches(Key::S, ModifierType::CONTROL_MASK));
    assert!(!binding.matches(Key::s, ModifierType::SHIFT_MASK));
    assert!(!binding.matches(Key::a, ModifierType::CONTROL_MASK));
}

#[test]
fn test_shortcut_manager_defaults() {
    let mgr = ShortcutManager::new();

    assert_eq!(
        mgr.match_action(Key::s, ModifierType::CONTROL_MASK),
        Some(Action::Save)
    );

    assert_eq!(
        mgr.match_action(Key::b, ModifierType::CONTROL_MASK),
        Some(Action::FormatBold)
    );

    assert_eq!(
        mgr.match_action(Key::e, ModifierType::CONTROL_MASK),
        Some(Action::TogglePreview)
    );

    assert_eq!(
        mgr.match_action(Key::f, ModifierType::CONTROL_MASK),
        Some(Action::Search)
    );
}

#[test]
fn test_shortcut_no_conflicts_in_defaults() {
    let mgr = ShortcutManager::new();
    let conflicts = mgr.check_conflicts();
    assert!(
        conflicts.is_empty(),
        "Unexpected conflicts in defaults: {:?}",
        conflicts
    );
}
