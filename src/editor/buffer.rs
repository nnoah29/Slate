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
** Editor buffer encapsulation around GtkSourceView5 with Markdown highlighting.
*/

use sourceview5::prelude::*;
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager};

pub struct EditorBuffer {
    buffer: Buffer,
}

impl EditorBuffer {
    pub fn new() -> Self {
        let buffer = Buffer::new(None);

        let lang_manager = LanguageManager::default();
        if let Some(markdown_lang) = lang_manager.language("markdown") {
            buffer.set_language(Some(&markdown_lang));
        }

        let scheme_manager = StyleSchemeManager::default();
        if let Some(scheme) = scheme_manager.scheme("Adwaita-dark") {
            buffer.set_style_scheme(Some(&scheme));
        } else if let Some(scheme) = scheme_manager.scheme("Adwaita") {
            buffer.set_style_scheme(Some(&scheme));
        } else if let Some(scheme) = scheme_manager.scheme("classic") {
            buffer.set_style_scheme(Some(&scheme));
        }

        buffer.set_highlight_syntax(true);
        buffer.set_highlight_matching_brackets(true);

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn text(&self) -> String {
        let (start, end) = self.buffer.bounds();
        self.buffer.text(&start, &end, true).to_string()
    }

    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
        self.buffer.set_modified(false);
    }

    pub fn clear(&self) {
        self.set_text("");
    }

    #[allow(dead_code)]
    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    pub fn set_modified(&self, modified: bool) {
        self.buffer.set_modified(modified);
    }

    #[allow(dead_code)]
    pub fn update_theme_scheme(&self, dark: bool) {
        let scheme_manager = StyleSchemeManager::default();
        let scheme_name = if dark { "Adwaita-dark" } else { "Adwaita" };
        if let Some(scheme) = scheme_manager.scheme(scheme_name) {
            self.buffer.set_style_scheme(Some(&scheme));
        }
    }
}

impl Default for EditorBuffer {
    fn default() -> Self {
        Self::new()
    }
}
