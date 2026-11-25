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
** Écrit le texte dicté dans un champ : partiel remplacé en direct, définitif validé, saisie respectée.
*/

use super::text_target::TextTarget;

/// Insère au curseur le texte reconnu, sans jamais écraser ce que l'utilisateur tape pendant ce temps.
#[derive(Debug, Default)]
pub struct Inserter {
    start: Option<usize>,
    written: String,
    expected_text: String,
    expected_cursor: usize,
    last_raw: String,
    skip: String,
    released: bool,
    wrote_once: bool,
}

impl Inserter {
    /// Remplace le texte provisoire (ou l'insère au curseur la première fois).
    pub fn show_partial(&mut self, target: &mut dyn TextTarget, text: &str) {
        if self.released {
            return;
        }
        self.follow_user(target);
        self.last_raw = text.to_string();
        let body = self.without_committed(text);
        self.write(target, &body, false);
    }

    /// Valide la phrase : le texte reste, suivi d'une espace, et la suite s'ancre au nouveau curseur.
    pub fn commit(&mut self, target: &mut dyn TextTarget, text: &str) {
        if self.released {
            return;
        }
        self.follow_user(target);
        let body = self.without_committed(text);
        self.write(target, &body, true);
    }

    /// Laisse le provisoire tel quel et ne touche plus jamais au champ.
    pub fn release(&mut self) {
        self.released = true;
    }

    /// Retire de la phrase cumulative ce que l'utilisateur a déjà vu validé dans le champ.
    fn without_committed(&self, text: &str) -> String {
        match text.strip_prefix(self.skip.as_str()) {
            Some(rest) if !self.skip.is_empty() => rest.trim().to_string(),
            _ => text.to_string(),
        }
    }

    /// Si l'utilisateur a tapé ou déplacé le curseur, le provisoire écrit devient du texte ordinaire.
    fn follow_user(&mut self, target: &dyn TextTarget) {
        let (start, end) = target.selection();
        let touched =
            target.text() != self.expected_text || start != end || end != self.expected_cursor;
        if self.start.is_some() && touched {
            self.skip = self.last_raw.clone();
            self.start = None;
            self.written.clear();
        }
    }

    /// Pose l'ancrage : seule la première écriture remplace une sélection, ensuite on écrit après elle.
    fn anchor(&mut self, target: &dyn TextTarget) {
        let (sel_start, sel_end) = target.selection();
        let start = if self.wrote_once { sel_end } else { sel_start };
        self.wrote_once = true;
        self.start = Some(start);
        let chars: Vec<char> = target.text().chars().collect();
        self.written = chars[start.min(chars.len())..sel_end.min(chars.len())]
            .iter()
            .collect();
    }

    /// Écrit `body` à la place du texte précédemment écrit par l'inserter.
    fn write(&mut self, target: &mut dyn TextTarget, body: &str, finalize: bool) {
        if self.start.is_none() && body.is_empty() {
            if finalize {
                self.reset();
            }
            return;
        }
        if self.start.is_none() {
            self.anchor(target);
        }
        self.apply(target, body, finalize);
        if finalize {
            self.reset();
        }
    }

    /// Remplace la plage écrite par la nouvelle version et mémorise l'état attendu.
    fn apply(&mut self, target: &mut dyn TextTarget, body: &str, finalize: bool) {
        let chars: Vec<char> = target.text().chars().collect();
        let start = self.start.unwrap_or(0).min(chars.len());
        let old_len = self.written.chars().count();
        let before: String = chars[..start].iter().collect();
        let after: String = chars[(start + old_len).min(chars.len())..].iter().collect();
        let lead = if !body.is_empty() && needs_blank(&before) {
            " "
        } else {
            ""
        };
        let tail = blank_after(body, &after, finalize);
        let written = if body.is_empty() {
            String::new()
        } else {
            format!("{lead}{body}{tail}")
        };
        let tail_len = if finalize { tail.chars().count() } else { 0 };
        let cursor = start + lead.len() + body.chars().count() + tail_len;
        target.replace(start, old_len, &written, cursor);
        self.expected_text = format!("{before}{written}{after}");
        self.expected_cursor = cursor;
        self.written = written;
    }

    /// Fin de phrase : l'ancrage et la mémoire du cumul repartent de zéro.
    fn reset(&mut self) {
        self.start = None;
        self.written.clear();
        self.last_raw.clear();
        self.skip.clear();
    }
}

/// Vrai si le texte qui précède se termine par un caractère qui n'est pas un blanc.
fn needs_blank(before: &str) -> bool {
    before.chars().last().is_some_and(|c| !c.is_whitespace())
}

/// Espace à placer après le texte écrit, selon ce qui suit dans le champ.
fn blank_after(body: &str, after: &str, finalize: bool) -> &'static str {
    match (body.is_empty(), after.chars().next()) {
        (true, _) => "",
        (false, None) => {
            if finalize {
                " "
            } else {
                ""
            }
        }
        (false, Some(c)) => {
            if c.is_whitespace() {
                ""
            } else {
                " "
            }
        }
    }
}
