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
** In-document search and replace floating bar with real-time highlighting.
*/

use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use sourceview5::prelude::*;
use sourceview5::{SearchContext, SearchSettings, View};
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct SearchReplaceBar {
    container: gtk4::Revealer,
    search_entry: gtk4::SearchEntry,
    replace_entry: gtk4::Entry,
    replace_row: gtk4::Box,
    count_label: gtk4::Label,
    search_context: Rc<RefCell<Option<SearchContext>>>,
    search_settings: SearchSettings,
}

impl SearchReplaceBar {
    pub fn new(view: &View) -> Self {
        let container = gtk4::Revealer::builder()
            .transition_type(gtk4::RevealerTransitionType::SlideDown)
            .reveal_child(false)
            .build();

        let root_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .margin_start(12)
            .margin_end(12)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        root_box.add_css_class("card");
        root_box.add_css_class("search-bar-card");

        let top_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_start(8)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        let search_entry = gtk4::SearchEntry::builder()
            .placeholder_text("Rechercher...")
            .hexpand(true)
            .build();

        let count_label = gtk4::Label::builder()
            .label("")
            .css_classes(["dim-label"])
            .build();

        let prev_btn = gtk4::Button::builder()
            .icon_name("go-up-symbolic")
            .tooltip_text("Précédent (Shift+Entrée)")
            .build();

        let next_btn = gtk4::Button::builder()
            .icon_name("go-down-symbolic")
            .tooltip_text("Suivant (Entrée)")
            .build();

        let close_btn = gtk4::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text("Fermer (Échap)")
            .build();
        close_btn.add_css_class("flat");

        top_row.append(&search_entry);
        top_row.append(&count_label);
        top_row.append(&prev_btn);
        top_row.append(&next_btn);
        top_row.append(&close_btn);

        let replace_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(6)
            .visible(false)
            .build();

        let replace_entry = gtk4::Entry::builder()
            .placeholder_text("Remplacer par...")
            .hexpand(true)
            .build();

        let replace_btn = gtk4::Button::builder().label("Remplacer").build();

        let replace_all_btn = gtk4::Button::builder().label("Tout remplacer").build();

        replace_row.append(&replace_entry);
        replace_row.append(&replace_btn);
        replace_row.append(&replace_all_btn);

        root_box.append(&top_row);
        root_box.append(&replace_row);
        container.set_child(Some(&root_box));

        let search_settings = SearchSettings::new();
        search_settings.set_wrap_around(true);

        let search_context: Rc<RefCell<Option<SearchContext>>> = Rc::new(RefCell::new(None));

        if let Some(buf) = view.buffer().downcast_ref::<sourceview5::Buffer>() {
            let ctx = SearchContext::new(buf, Some(&search_settings));
            ctx.set_highlight(true);
            *search_context.borrow_mut() = Some(ctx);
        }

        {
            let settings = search_settings.clone();
            let count_lbl = count_label.clone();
            let ctx_cell = search_context.clone();
            let view_clone = view.clone();
            search_entry.connect_search_changed(move |entry| {
                let text = entry.text();
                if text.is_empty() {
                    settings.set_search_text(None);
                    count_lbl.set_label("");
                } else {
                    settings.set_search_text(Some(&text));
                    if let Some(ctx) = ctx_cell.borrow().as_ref() {
                        let count = ctx.occurrences_count();
                        if count == -1 {
                            count_lbl.set_label("Recherche...");
                        } else if count == 0 {
                            count_lbl.set_label("Aucun résultat");
                        } else {
                            count_lbl.set_label(&format!("{count} trouvés"));
                        }
                    }
                    Self::navigate_next(&view_clone, &ctx_cell);
                }
            });
        }

        {
            let ctx_cell = search_context.clone();
            let view_clone = view.clone();
            next_btn.connect_clicked(move |_| {
                Self::navigate_next(&view_clone, &ctx_cell);
            });
        }
        {
            let ctx_cell = search_context.clone();
            let view_clone = view.clone();
            search_entry.connect_activate(move |_| {
                Self::navigate_next(&view_clone, &ctx_cell);
            });
        }

        {
            let ctx_cell = search_context.clone();
            let view_clone = view.clone();
            prev_btn.connect_clicked(move |_| {
                Self::navigate_prev(&view_clone, &ctx_cell);
            });
        }

        {
            let revealer = container.clone();
            let view_clone = view.clone();
            close_btn.connect_clicked(move |_| {
                revealer.set_reveal_child(false);
                view_clone.grab_focus();
            });
        }

        {
            let rep_entry = replace_entry.clone();
            let ctx_cell = search_context.clone();
            let view_clone = view.clone();
            replace_btn.connect_clicked(move |_| {
                Self::replace_one(&view_clone, &ctx_cell, &rep_entry.text());
            });
        }

        {
            let rep_entry = replace_entry.clone();
            let ctx_cell = search_context.clone();
            let count_lbl = count_label.clone();
            replace_all_btn.connect_clicked(move |_| {
                Self::replace_all(&ctx_cell, &rep_entry.text(), &count_lbl);
            });
        }

        let key_controller = gtk4::EventControllerKey::new();
        {
            let revealer = container.clone();
            let view_clone = view.clone();
            key_controller.connect_key_pressed(move |_, keyval, _, _| {
                if keyval == gdk::Key::Escape {
                    revealer.set_reveal_child(false);
                    view_clone.grab_focus();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            });
        }
        root_box.add_controller(key_controller);

        Self {
            container,
            search_entry,
            replace_entry,
            replace_row,
            count_label,
            search_context,
            search_settings,
        }
    }

    pub fn widget(&self) -> &gtk4::Revealer {
        &self.container
    }

    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        self.container.reveals_child()
    }

    pub fn open_search(&self) {
        self.replace_row.set_visible(false);
        self.container.set_reveal_child(true);
        self.search_entry.grab_focus();
    }

    pub fn open_replace(&self) {
        self.replace_row.set_visible(true);
        self.container.set_reveal_child(true);
        self.search_entry.grab_focus();
    }

    #[allow(dead_code)]
    pub fn close(&self, view: &View) {
        self.container.set_reveal_child(false);
        self.search_settings.set_search_text(None);
        self.count_label.set_label("");
        view.grab_focus();
    }

    fn navigate_next(view: &View, ctx_cell: &Rc<RefCell<Option<SearchContext>>>) {
        if let Some(ctx) = ctx_cell.borrow().as_ref() {
            let buffer = view.buffer();
            let cursor = buffer.iter_at_offset(buffer.cursor_position());
            if let Some((start, end, _has_wrapped)) = ctx.forward(&cursor) {
                buffer.select_range(&start, &end);
                view.scroll_to_iter(&mut start.clone(), 0.1, false, 0.0, 0.5);
            }
        }
    }

    fn navigate_prev(view: &View, ctx_cell: &Rc<RefCell<Option<SearchContext>>>) {
        if let Some(ctx) = ctx_cell.borrow().as_ref() {
            let buffer = view.buffer();
            let cursor = if let Some((start, _)) = buffer.selection_bounds() {
                start
            } else {
                buffer.iter_at_offset(buffer.cursor_position())
            };
            if let Some((start, end, _has_wrapped)) = ctx.backward(&cursor) {
                buffer.select_range(&start, &end);
                view.scroll_to_iter(&mut start.clone(), 0.1, false, 0.0, 0.5);
            }
        }
    }

    fn replace_one(view: &View, ctx_cell: &Rc<RefCell<Option<SearchContext>>>, replace_text: &str) {
        if let Some(ctx) = ctx_cell.borrow().as_ref() {
            let buffer = view.buffer();
            if let Some((mut start, mut end)) = buffer.selection_bounds() {
                let _ = ctx.replace(&mut start, &mut end, replace_text);
            }
            Self::navigate_next(view, ctx_cell);
        }
    }

    fn replace_all(
        ctx_cell: &Rc<RefCell<Option<SearchContext>>>,
        replace_text: &str,
        count_lbl: &gtk4::Label,
    ) {
        if let Some(ctx) = ctx_cell.borrow().as_ref() {
            match ctx.replace_all(replace_text) {
                Ok(()) => {
                    count_lbl.set_label("Remplacements terminés");
                }
                Err(_) => {
                    count_lbl.set_label("Erreur de remplacement");
                }
            }
        }
    }
}
