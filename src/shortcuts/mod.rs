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
** Centralized keyboard shortcut registry and event matching.
*/

use gtk4::gdk::{self, Key};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Save,
    SaveAs,
    New,
    Open,
    Close,
    Quit,
    Undo,
    Redo,
    TogglePreview,
    Search,
    Replace,
    FormatBold,
    FormatItalic,
    FormatLink,
    FormatStrikethrough,
    FormatHeading,
    FormatBulletList,
    FormatNumberedList,
    FormatCode,
    FormatBlockquote,
    Indent,
    Unindent,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleConceal,
    ToggleWrapMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: Key,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBinding {
    pub fn new(key: Key, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key,
            ctrl,
            shift,
            alt,
        }
    }

    pub fn matches(&self, keyval: Key, modifier_state: gdk::ModifierType) -> bool {
        let has_ctrl = modifier_state.contains(gdk::ModifierType::CONTROL_MASK);
        let has_shift = modifier_state.contains(gdk::ModifierType::SHIFT_MASK);
        let has_alt = modifier_state.contains(gdk::ModifierType::ALT_MASK);

        let key_match = self.key == keyval || self.key.to_lower() == keyval.to_lower();
        key_match && self.ctrl == has_ctrl && self.shift == has_shift && self.alt == has_alt
    }
}

pub struct ShortcutManager {
    bindings: HashMap<KeyBinding, Action>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            bindings: HashMap::new(),
        };
        mgr.register_defaults();
        mgr
    }

    pub fn register_defaults(&mut self) {
        self.bind(Key::s, true, false, false, Action::Save);
        self.bind(Key::S, true, true, false, Action::SaveAs);
        self.bind(Key::n, true, false, false, Action::New);
        self.bind(Key::o, true, false, false, Action::Open);
        self.bind(Key::w, true, false, false, Action::Close);
        self.bind(Key::q, true, false, false, Action::Quit);
        self.bind(Key::z, true, false, false, Action::Undo);
        self.bind(Key::Z, true, true, false, Action::Redo);

        self.bind(Key::e, true, false, false, Action::TogglePreview);
        self.bind(Key::l, true, false, false, Action::ToggleConceal);
        self.bind(Key::z, false, false, true, Action::ToggleWrapMode);

        self.bind(Key::f, true, false, false, Action::Search);
        self.bind(Key::h, true, false, false, Action::Replace);

        self.bind(Key::b, true, false, false, Action::FormatBold);
        self.bind(Key::i, true, false, false, Action::FormatItalic);
        self.bind(Key::k, true, false, false, Action::FormatLink);
        self.bind(Key::X, true, true, false, Action::FormatStrikethrough);
        self.bind(Key::x, true, true, false, Action::FormatStrikethrough);
        self.bind(Key::H, true, true, false, Action::FormatHeading);
        self.bind(Key::C, true, true, false, Action::FormatCode);
        self.bind(Key::c, true, true, false, Action::FormatCode);
        self.bind(Key::Q, true, true, false, Action::FormatBlockquote);
        self.bind(Key::q, true, true, false, Action::FormatBlockquote);

        self.bind(Key::_8, true, true, false, Action::FormatBulletList);
        self.bind(Key::asterisk, true, true, false, Action::FormatBulletList);
        self.bind(Key::_7, true, true, false, Action::FormatNumberedList);
        self.bind(
            Key::ampersand,
            true,
            true,
            false,
            Action::FormatNumberedList,
        );

        self.bind(Key::Tab, false, false, false, Action::Indent);
        self.bind(Key::ISO_Left_Tab, false, true, false, Action::Unindent);
        self.bind(Key::Tab, false, true, false, Action::Unindent);

        self.bind(Key::plus, true, false, false, Action::ZoomIn);
        self.bind(Key::equal, true, false, false, Action::ZoomIn);
        self.bind(Key::minus, true, false, false, Action::ZoomOut);
        self.bind(Key::_0, true, false, false, Action::ZoomReset);
    }

    pub fn bind(&mut self, key: Key, ctrl: bool, shift: bool, alt: bool, action: Action) {
        let binding = KeyBinding::new(key, ctrl, shift, alt);
        self.bindings.insert(binding, action);
    }

    pub fn match_action(&self, keyval: Key, modifier_state: gdk::ModifierType) -> Option<Action> {
        for (binding, action) in &self.bindings {
            if binding.matches(keyval, modifier_state) {
                return Some(*action);
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn check_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        let mut seen = HashMap::new();
        for (binding, action) in &self.bindings {
            if let Some(prev) = seen.insert(binding, action) {
                conflicts.push(format!("Conflit entre {:?} et {:?}", prev, action));
            }
        }
        conflicts
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}
