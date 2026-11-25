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
** Palette de commandes : recherche floue d'une action et exécution au clavier.
*/

use super::matches_query;
use crate::shortcuts::{Action, ShortcutManager, all_actions};
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::rc::Rc;

/// Actions proposées : toutes sauf la palette elle-même et l'indentation (Tab).
fn palette_actions() -> Vec<Action> {
    let hidden = [Action::CommandPalette, Action::Indent, Action::Unindent];
    all_actions()
        .into_iter()
        .filter(|a| !hidden.contains(a))
        .collect()
}

/// Affiche le raccourci d'une action sous une forme lisible (« Ctrl+S »).
fn accelerator_label(mgr: &ShortcutManager, action: Action) -> String {
    let name = mgr.accelerator_for(action).unwrap_or_default();
    match gtk4::accelerator_parse(&name) {
        Some((key, mods)) => gtk4::accelerator_get_label(key, mods).to_string(),
        None => String::new(),
    }
}

/// Construit la ligne d'une action : libellé à gauche, raccourci grisé à droite.
fn build_row(mgr: &ShortcutManager, action: Action) -> gtk4::ListBoxRow {
    let line = gtk4::Box::builder()
        .spacing(12)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(12)
        .margin_end(12)
        .build();
    let label = gtk4::Label::builder()
        .label(action.label())
        .xalign(0.0)
        .hexpand(true)
        .build();
    let accel = gtk4::Label::builder()
        .label(accelerator_label(mgr, action))
        .css_classes(["dim-label"])
        .build();
    line.append(&label);
    line.append(&accel);
    gtk4::ListBoxRow::builder().child(&line).build()
}

/// Déplace la sélection vers la ligne visible suivante (+1) ou précédente (-1).
fn move_selection(list: &gtk4::ListBox, delta: i32) {
    let current = list.selected_row().map_or(-1, |r| r.index());
    let mut index = current + delta;
    while let Some(row) = list.row_at_index(index) {
        if row.is_child_visible() {
            list.select_row(Some(&row));
            return;
        }
        index += delta;
    }
}

/// Ouvre la palette ; l'action choisie est transmise à `on_run` après fermeture.
pub fn present_command_palette(
    parent: &impl IsA<gtk4::Widget>,
    mgr: &ShortcutManager,
    on_run: Rc<dyn Fn(Action)>,
) {
    let actions = Rc::new(palette_actions());
    let entry = gtk4::SearchEntry::builder()
        .placeholder_text("Rechercher une commande…")
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .build();
    let list = gtk4::ListBox::builder()
        .selection_mode(gtk4::SelectionMode::Single)
        .css_classes(["navigation-sidebar"])
        .build();
    actions
        .iter()
        .for_each(|a| list.append(&build_row(mgr, *a)));
    let dialog = libadwaita::Dialog::builder()
        .title("Palette de commandes")
        .content_width(440)
        .content_height(420)
        .build();
    wire_palette(&dialog, &entry, &list, &actions, on_run);
    let scroller = gtk4::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .margin_top(8)
        .build();
    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
    content.append(&entry);
    content.append(&scroller);
    dialog.set_child(Some(&content));
    dialog.set_focus(Some(&entry));
    dialog.present(Some(parent));
}

/// Branche filtrage, navigation clavier et exécution sur les widgets de la palette.
fn wire_palette(
    dialog: &libadwaita::Dialog,
    entry: &gtk4::SearchEntry,
    list: &gtk4::ListBox,
    actions: &Rc<Vec<Action>>,
    on_run: Rc<dyn Fn(Action)>,
) {
    let query = Rc::new(std::cell::RefCell::new(String::new()));
    let (acts, q) = (actions.clone(), query.clone());
    list.set_filter_func(move |row| {
        let label = acts.get(row.index() as usize).map_or("", |a| a.label());
        matches_query(label, &q.borrow())
    });
    let list_f = list.clone();
    entry.connect_search_changed(move |e| {
        *query.borrow_mut() = e.text().to_string();
        list_f.invalidate_filter();
        list_f.select_row(None::<&gtk4::ListBoxRow>);
        move_selection(&list_f, 1);
    });
    wire_activation(dialog, entry, list, actions, on_run);
}

/// Gère les flèches, Entrée et le clic : ferme la palette puis lance l'action.
fn wire_activation(
    dialog: &libadwaita::Dialog,
    entry: &gtk4::SearchEntry,
    list: &gtk4::ListBox,
    actions: &Rc<Vec<Action>>,
    on_run: Rc<dyn Fn(Action)>,
) {
    let run = {
        let (dialog, actions) = (dialog.clone(), actions.clone());
        Rc::new(move |row: &gtk4::ListBoxRow| {
            let Some(action) = actions.get(row.index() as usize).copied() else {
                return;
            };
            dialog.close();
            let on_run = on_run.clone();
            glib::idle_add_local_once(move || on_run(action));
        })
    };
    let r = run.clone();
    list.connect_row_activated(move |_, row| r(row));
    let (list_a, r) = (list.clone(), run);
    entry.connect_activate(move |_| {
        if let Some(row) = list_a.selected_row() {
            r(&row);
        }
    });
    let keys = gtk4::EventControllerKey::new();
    let list_k = list.clone();
    keys.connect_key_pressed(move |_, key, _, _| match key {
        gtk4::gdk::Key::Down => {
            move_selection(&list_k, 1);
            glib::Propagation::Stop
        }
        gtk4::gdk::Key::Up => {
            move_selection(&list_k, -1);
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    });
    entry.add_controller(keys);
}
