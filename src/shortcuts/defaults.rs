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
** Raccourcis par défaut ; le premier lié à une action est celui qui s'affiche.
*/

use super::{Action, KeycodeBinding, ShortcutManager};
use gtk4::gdk::Key;

const FIRST_DIGIT_KEYCODE: u32 = 10;

impl ShortcutManager {
    /// Enregistre tous les raccourcis par défaut.
    pub fn register_defaults(&mut self) {
        self.register_files_and_edit();
        self.register_formatting();
        self.register_view_and_help();
    }

    fn register_files_and_edit(&mut self) {
        self.bind(Key::s, true, false, false, Action::Save);
        self.bind(Key::S, true, true, false, Action::SaveAs);
        self.bind(Key::n, true, false, false, Action::New);
        self.bind(Key::o, true, false, false, Action::Open);
        self.bind(Key::w, true, false, false, Action::Close);
        self.bind(Key::q, true, false, false, Action::Quit);
        self.bind(Key::z, true, false, false, Action::Undo);
        self.bind(Key::Z, true, true, false, Action::Redo);
        self.bind(Key::f, true, false, false, Action::Search);
        self.bind(Key::h, true, false, false, Action::Replace);
    }

    fn register_formatting(&mut self) {
        self.bind(Key::b, true, false, false, Action::FormatBold);
        self.bind(Key::i, true, false, false, Action::FormatItalic);
        self.bind(Key::k, true, false, false, Action::FormatLink);
        self.bind(Key::X, true, true, false, Action::FormatStrikethrough);
        self.bind(Key::H, true, true, false, Action::FormatHeading);
        self.bind(Key::C, true, true, false, Action::FormatCode);
        self.bind(Key::Q, true, true, false, Action::FormatBlockquote);
        self.bind(Key::L, true, true, false, Action::FormatBulletList);
        self.bind(Key::_8, true, true, false, Action::FormatBulletList);
        self.bind(Key::asterisk, true, true, false, Action::FormatBulletList);
        self.bind(Key::N, true, true, false, Action::FormatNumberedList);
        self.bind(Key::_7, true, true, false, Action::FormatNumberedList);
        self.bind(
            Key::ampersand,
            true,
            true,
            false,
            Action::FormatNumberedList,
        );
        self.bind(Key::Tab, false, false, false, Action::Indent);
        self.bind(Key::Tab, false, true, false, Action::Unindent);
        self.bind(Key::ISO_Left_Tab, false, true, false, Action::Unindent);
        self.register_heading_levels();
    }

    fn register_heading_levels(&mut self) {
        for level in 1u8..=6 {
            let binding = KeycodeBinding {
                code: FIRST_DIGIT_KEYCODE + u32::from(level) - 1,
                ctrl: true,
                shift: false,
                alt: true,
            };
            self.keycode_bindings
                .push((binding, Action::Heading(level)));
        }
    }

    fn register_view_and_help(&mut self) {
        self.bind(Key::e, true, false, false, Action::TogglePreview);
        self.bind(Key::M, true, true, false, Action::ToggleConceal);
        self.bind(Key::l, true, false, false, Action::ToggleConceal);
        self.bind(Key::z, false, false, true, Action::ToggleWrapMode);
        self.bind(Key::plus, true, false, false, Action::ZoomIn);
        self.bind(Key::equal, true, false, false, Action::ZoomIn);
        self.bind(Key::minus, true, false, false, Action::ZoomOut);
        self.bind(Key::_0, true, false, false, Action::ZoomReset);
        self.bind(Key::F1, false, false, false, Action::ShowShortcuts);
        self.bind(Key::question, true, true, false, Action::ShowShortcuts);
        self.bind(Key::p, true, false, false, Action::CommandPalette);
    }
}
