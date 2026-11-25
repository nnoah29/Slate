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
** Centralized keyboard shortcut registry and event matching.
*/

mod accelerator;
mod catalog;
mod custom;
mod defaults;

pub use catalog::all_actions;

use gtk4::gdk::{self, Key};

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
    Heading(u8),
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
    ToggleDictation,
    ShowShortcuts,
    CommandPalette,
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

    /// Nom GTK du raccourci (`<Control><Shift>l`), utilisé pour l'affichage.
    pub fn accelerator_name(&self) -> String {
        let name = self.key.to_lower().name().map(|n| n.to_string());
        let key_name = name.unwrap_or_default();
        accelerator::format_accelerator(self.ctrl, self.shift, self.alt, &key_name)
    }
}

/// Raccourci lié à une touche physique, donc indépendant de la disposition du clavier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeycodeBinding {
    pub code: u32,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

pub struct ShortcutManager {
    pub(crate) bindings: Vec<(KeyBinding, Action)>,
    pub(crate) keycode_bindings: Vec<(KeycodeBinding, Action)>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            bindings: Vec::new(),
            keycode_bindings: Vec::new(),
        };
        mgr.register_defaults();
        mgr
    }

    pub fn bind(&mut self, key: Key, ctrl: bool, shift: bool, alt: bool, action: Action) {
        let binding = KeyBinding::new(key, ctrl, shift, alt);
        match self.bindings.iter_mut().find(|(b, _)| *b == binding) {
            Some(existing) => existing.1 = action,
            None => self.bindings.push((binding, action)),
        }
    }

    pub fn match_action(&self, keyval: Key, modifier_state: gdk::ModifierType) -> Option<Action> {
        self.bindings
            .iter()
            .find(|(binding, _)| binding.matches(keyval, modifier_state))
            .map(|(_, action)| *action)
    }

    /// Cherche d'abord un raccourci par touche physique, puis par caractère produit.
    pub fn match_action_with_keycode(
        &self,
        keyval: Key,
        keycode: u32,
        modifier_state: gdk::ModifierType,
    ) -> Option<Action> {
        let ctrl = modifier_state.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = modifier_state.contains(gdk::ModifierType::SHIFT_MASK);
        let alt = modifier_state.contains(gdk::ModifierType::ALT_MASK);
        let by_code = self
            .keycode_bindings
            .iter()
            .find(|(b, _)| b.code == keycode && b.ctrl == ctrl && b.shift == shift && b.alt == alt);
        by_code
            .map(|(_, action)| *action)
            .or_else(|| self.match_action(keyval, modifier_state))
    }

    /// Premier raccourci de l'action, au format GTK, pour l'afficher à l'utilisateur.
    pub fn accelerator_for(&self, action: Action) -> Option<String> {
        let by_key = self.bindings.iter().find(|(_, a)| *a == action);
        if let Some((binding, _)) = by_key {
            return Some(binding.accelerator_name());
        }
        let (binding, _) = self.keycode_bindings.iter().find(|(_, a)| *a == action)?;
        Some(binding.accelerator_name())
    }

    pub(crate) fn remove_action(&mut self, action: Action) {
        self.bindings.retain(|(_, a)| *a != action);
        self.keycode_bindings.retain(|(_, a)| *a != action);
    }

    #[allow(dead_code)]
    pub fn check_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        for (i, (binding, action)) in self.bindings.iter().enumerate() {
            let duplicate = self.bindings[i + 1..].iter().find(|(b, _)| b == binding);
            if let Some((_, other)) = duplicate {
                conflicts.push(format!("Conflit entre {action:?} et {other:?}"));
            }
        }
        conflicts
    }
}

impl KeycodeBinding {
    /// Nom d'affichage de type GTK, la touche étant le chiffre de la rangée haute.
    pub(crate) fn accelerator_name(&self) -> String {
        let digit = (self.code.saturating_sub(9) % 10).to_string();
        accelerator::format_accelerator(self.ctrl, self.shift, self.alt, &digit)
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}
