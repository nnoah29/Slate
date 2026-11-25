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
** Adaptateur : le `TextBuffer` GTK de la note vu comme cible de l'insertion dictée.
*/

use super::text_target::TextTarget;
use gtk4::prelude::*;

/// Buffer de la note ; les positions sont des offsets en caractères, balises masquées comprises.
pub struct BufferTarget<'a> {
    buffer: &'a gtk4::TextBuffer,
}

impl<'a> BufferTarget<'a> {
    /// Cible sur un buffer existant.
    pub fn new(buffer: &'a gtk4::TextBuffer) -> Self {
        Self { buffer }
    }

    fn offset(&self, value: usize) -> gtk4::TextIter {
        self.buffer
            .iter_at_offset(i32::try_from(value).unwrap_or(i32::MAX))
    }
}

impl TextTarget for BufferTarget<'_> {
    fn text(&self) -> String {
        let (start, end) = self.buffer.bounds();
        self.buffer.text(&start, &end, true).to_string()
    }

    fn selection(&self) -> (usize, usize) {
        match self.buffer.selection_bounds() {
            Some((start, end)) => (start.offset() as usize, end.offset() as usize),
            None => {
                let cursor = self.buffer.cursor_position() as usize;
                (cursor, cursor)
            }
        }
    }

    fn replace(&mut self, start: usize, len: usize, text: &str, cursor: usize) {
        let (mut from, mut to) = (self.offset(start), self.offset(start + len));
        self.buffer.delete(&mut from, &mut to);
        let mut at = self.offset(start);
        self.buffer.insert(&mut at, text);
        self.buffer.place_cursor(&self.offset(cursor));
    }
}
