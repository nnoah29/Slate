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
** Catalogue des actions : identifiant stable, libellé français et catégorie.
*/

use super::Action;

const ENTRIES: &[(Action, &str, &str, &str)] = &[
    (Action::Save, "save", "Enregistrer", "Fichier"),
    (Action::SaveAs, "save_as", "Enregistrer sous…", "Fichier"),
    (Action::New, "new", "Nouvelle note", "Fichier"),
    (Action::Open, "open", "Ouvrir une note", "Fichier"),
    (Action::Close, "close", "Fermer la note", "Fichier"),
    (Action::Quit, "quit", "Quitter", "Fichier"),
    (Action::Undo, "undo", "Annuler", "Édition"),
    (Action::Redo, "redo", "Rétablir", "Édition"),
    (
        Action::ToggleDictation,
        "toggle_dictation",
        "Dictée vocale",
        "Édition",
    ),
    (Action::Search, "search", "Rechercher", "Édition"),
    (
        Action::Replace,
        "replace",
        "Rechercher et remplacer",
        "Édition",
    ),
    (Action::FormatBold, "bold", "Gras", "Formatage"),
    (Action::FormatItalic, "italic", "Italique", "Formatage"),
    (Action::FormatLink, "link", "Lien", "Formatage"),
    (
        Action::FormatStrikethrough,
        "strikethrough",
        "Barré",
        "Formatage",
    ),
    (
        Action::FormatHeading,
        "heading_cycle",
        "Titre (niveau suivant)",
        "Formatage",
    ),
    (Action::Heading(1), "heading_1", "Titre 1", "Formatage"),
    (Action::Heading(2), "heading_2", "Titre 2", "Formatage"),
    (Action::Heading(3), "heading_3", "Titre 3", "Formatage"),
    (Action::Heading(4), "heading_4", "Titre 4", "Formatage"),
    (Action::Heading(5), "heading_5", "Titre 5", "Formatage"),
    (Action::Heading(6), "heading_6", "Titre 6", "Formatage"),
    (
        Action::FormatBulletList,
        "bullet_list",
        "Liste à puces",
        "Formatage",
    ),
    (
        Action::FormatNumberedList,
        "numbered_list",
        "Liste numérotée",
        "Formatage",
    ),
    (Action::FormatCode, "code", "Code", "Formatage"),
    (
        Action::FormatBlockquote,
        "blockquote",
        "Citation",
        "Formatage",
    ),
    (Action::Indent, "indent", "Indenter", "Formatage"),
    (Action::Unindent, "unindent", "Désindenter", "Formatage"),
    (
        Action::TogglePreview,
        "toggle_preview",
        "Aperçu / Édition",
        "Affichage",
    ),
    (
        Action::ToggleConceal,
        "toggle_conceal",
        "Masquer / afficher les balises",
        "Affichage",
    ),
    (
        Action::ToggleWrapMode,
        "toggle_wrap",
        "Retour à la ligne",
        "Affichage",
    ),
    (Action::ZoomIn, "zoom_in", "Zoom avant", "Affichage"),
    (Action::ZoomOut, "zoom_out", "Zoom arrière", "Affichage"),
    (
        Action::ZoomReset,
        "zoom_reset",
        "Zoom par défaut",
        "Affichage",
    ),
    (
        Action::ShowShortcuts,
        "shortcuts_help",
        "Aide : raccourcis clavier",
        "Aide",
    ),
    (
        Action::CommandPalette,
        "command_palette",
        "Palette de commandes",
        "Aide",
    ),
];

/// Liste toutes les actions dans l'ordre d'affichage du catalogue.
pub fn all_actions() -> Vec<Action> {
    ENTRIES.iter().map(|(action, ..)| *action).collect()
}

fn entry(action: Action) -> Option<&'static (Action, &'static str, &'static str, &'static str)> {
    ENTRIES.iter().find(|(a, ..)| *a == action)
}

impl Action {
    /// Identifiant stable utilisé dans `custom_shortcuts`.
    #[allow(dead_code)]
    pub fn id(&self) -> &'static str {
        entry(*self).map_or("", |e| e.1)
    }

    /// Retrouve une action à partir de son identifiant.
    pub fn from_id(id: &str) -> Option<Action> {
        ENTRIES.iter().find(|e| e.1 == id).map(|e| e.0)
    }

    /// Libellé français affiché dans l'aide et la palette.
    pub fn label(&self) -> &'static str {
        entry(*self).map_or("", |e| e.2)
    }

    /// Catégorie française de l'action.
    pub fn category(&self) -> &'static str {
        entry(*self).map_or("", |e| e.3)
    }
}
