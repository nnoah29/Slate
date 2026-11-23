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
** Cible de texte abstraite (positions en caractères) et son implémentation simple sur une chaîne.
*/

/// Champ de texte dans lequel on insère : lecture, sélection et remplacement d'une plage.
pub trait TextTarget {
    /// Texte complet.
    fn text(&self) -> String;
    /// Sélection (début, fin) en caractères ; égaux si elle est vide, la position étant alors le curseur.
    fn selection(&self) -> (usize, usize);
    /// Remplace `len` caractères à partir de `start` par `text`, puis place le curseur.
    fn replace(&mut self, start: usize, len: usize, text: &str, cursor: usize);
}

/// Cible en mémoire : sert aux tests et à tout champ réduit à une chaîne.
#[derive(Debug, Clone)]
pub struct StringTarget {
    text: String,
    selection: (usize, usize),
}

impl StringTarget {
    /// Texte initial et curseur (fin du texte par défaut).
    pub fn new(text: &str, cursor: Option<usize>) -> Self {
        let at = cursor.unwrap_or_else(|| text.chars().count());
        Self {
            text: text.to_string(),
            selection: (at, at),
        }
    }

    /// Texte courant.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Position du curseur (fin de la sélection).
    pub fn cursor(&self) -> usize {
        self.selection.1
    }

    /// Simule une sélection faite par l'utilisateur.
    pub fn set_selection(&mut self, start: usize, end: usize) {
        self.selection = (start, end);
    }

    /// Simule une frappe : remplace tout le texte et place le curseur.
    pub fn set_value(&mut self, text: &str, cursor: usize) {
        self.text = text.to_string();
        self.selection = (cursor, cursor);
    }
}

impl TextTarget for StringTarget {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn selection(&self) -> (usize, usize) {
        self.selection
    }

    fn replace(&mut self, start: usize, len: usize, text: &str, cursor: usize) {
        let chars: Vec<char> = self.text.chars().collect();
        let end = (start + len).min(chars.len());
        let before: String = chars[..start.min(chars.len())].iter().collect();
        let after: String = chars[end..].iter().collect();
        self.text = format!("{before}{text}{after}");
        self.selection = (cursor, cursor);
    }
}
