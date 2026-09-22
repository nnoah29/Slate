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
** Main window presentation, editor/preview stacking, and document persistence workflows.
*/

use crate::config::AppConfig;
use crate::document::Document;
use crate::editor::EditorView;
use crate::preview::PreviewView;
use crate::search::SearchReplaceBar;
use crate::shortcuts::{Action, ShortcutManager};
use crate::storage::{check_external_modification, generate_diff, read_file, write_file_atomic};

use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

pub struct WindowState {
    pub document: Document,
    pub config: AppConfig,
    pub font_size: f64,
    pub font_family: Option<String>,
    pub pending_action: Option<PendingAction>,
    pub is_reloading: bool,
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    NewDocument,
    OpenPath(PathBuf),
    OpenDialog,
    CloseDocument,
    QuitApp,
}

pub struct SlateWindow {
    window: libadwaita::ApplicationWindow,
    state: Rc<RefCell<WindowState>>,
    editor_view: Rc<EditorView>,
    preview_view: Rc<PreviewView>,
    search_bar: Rc<SearchReplaceBar>,
    stack: gtk4::Stack,
    toast_overlay: libadwaita::ToastOverlay,
    shortcut_mgr: ShortcutManager,
}

impl SlateWindow {
    pub fn new(
        app: &libadwaita::Application,
        initial_path: Option<PathBuf>,
        config: AppConfig,
    ) -> Rc<Self> {
        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("Slate")
            .default_width(config.window_width)
            .default_height(config.window_height)
            .decorated(false)
            .build();

        if config.is_maximized {
            window.maximize();
        }

        let font_size = config.font_size;
        let font_family = config.font_family.clone();

        let state = Rc::new(RefCell::new(WindowState {
            document: Document::new_untitled(),
            config,
            font_size,
            font_family,
            pending_action: None,
            is_reloading: false,
        }));

        let editor_view = Rc::new(EditorView::new());
        let preview_view = Rc::new(PreviewView::new());
        let search_bar = Rc::new(SearchReplaceBar::new(editor_view.view()));

        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::Crossfade)
            .transition_duration(150)
            .hexpand(true)
            .vexpand(true)
            .build();

        stack.add_named(editor_view.widget(), Some("editor"));
        stack.add_named(preview_view.widget(), Some("preview"));
        stack.set_visible_child_name("editor");

        let main_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .build();

        main_box.append(search_bar.widget());
        main_box.append(&stack);

        let toast_overlay = libadwaita::ToastOverlay::new();
        toast_overlay.set_child(Some(&main_box));

        let window_handle = gtk4::WindowHandle::builder().child(&toast_overlay).build();

        window.set_content(Some(&window_handle));

        let slate_win = Rc::new(Self {
            window,
            state,
            editor_view,
            preview_view,
            search_bar,
            stack,
            toast_overlay,
            shortcut_mgr: ShortcutManager::new(),
        });

        slate_win.init(initial_path);
        slate_win
    }

    pub fn window(&self) -> &libadwaita::ApplicationWindow {
        &self.window
    }

    fn init(self: &Rc<Self>, initial_path: Option<PathBuf>) {
        {
            let s = self.state.borrow();
            self.editor_view
                .set_font(s.font_size, s.font_family.as_deref());
        }

        self.setup_buffer_tracking();

        self.setup_shortcuts();

        self.setup_drag_and_drop();

        self.setup_close_handler();

        self.setup_autosave();

        self.setup_focus_watcher();

        if let Some(path) = initial_path {
            self.load_file(&path);
        } else {
            self.update_title();
        }

        self.editor_view.grab_focus();
    }

    fn setup_buffer_tracking(self: &Rc<Self>) {
        let this = Rc::downgrade(self);
        let buffer = self.editor_view.buffer().buffer().clone();

        buffer.connect_modified_changed(move |buf| {
            if let Some(this) = this.upgrade() {
                if this.state.borrow().is_reloading {
                    return;
                }
                if buf.is_modified() {
                    this.state.borrow_mut().document.mark_dirty();
                }
                this.update_title();
            }
        });
    }

    fn setup_shortcuts(self: &Rc<Self>) {
        let key_controller = gtk4::EventControllerKey::new();
        let this = Rc::downgrade(self);

        key_controller.connect_key_pressed(move |_, keyval, _, state| {
            let Some(this) = this.upgrade() else {
                return glib::Propagation::Proceed;
            };
            if let Some(action) = this.shortcut_mgr.match_action(keyval, state) {
                this.handle_action(action);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });

        self.window.add_controller(key_controller);
    }

    fn handle_action(self: &Rc<Self>, action: Action) {
        let buffer = self.editor_view.buffer().buffer();

        match action {
            Action::Save => self.save_document(false),
            Action::SaveAs => self.save_as_dialog(),
            Action::New => self.prompt_unsaved_if_dirty(PendingAction::NewDocument),
            Action::Open => self.prompt_unsaved_if_dirty(PendingAction::OpenDialog),
            Action::Close => self.prompt_unsaved_if_dirty(PendingAction::CloseDocument),
            Action::Quit => self.prompt_unsaved_if_dirty(PendingAction::QuitApp),
            Action::Undo => {
                if buffer.can_undo() {
                    buffer.undo();
                }
            }
            Action::Redo => {
                if buffer.can_redo() {
                    buffer.redo();
                }
            }
            Action::TogglePreview => self.toggle_preview(),
            Action::Search => self.search_bar.open_search(),
            Action::Replace => self.search_bar.open_replace(),
            Action::FormatBold => {
                crate::editor::formatting::apply_toggle_wrap(buffer.upcast_ref(), "**", "**");
            }
            Action::FormatItalic => {
                crate::editor::formatting::apply_toggle_wrap(buffer.upcast_ref(), "*", "*");
            }
            Action::FormatLink => {
                crate::editor::formatting::apply_link(buffer.upcast_ref());
            }
            Action::FormatStrikethrough => {
                crate::editor::formatting::apply_toggle_wrap(buffer.upcast_ref(), "~~", "~~");
            }
            Action::FormatHeading => {
                crate::editor::formatting::apply_cycle_heading(buffer.upcast_ref());
            }
            Action::FormatBulletList => {
                crate::editor::formatting::apply_bullet_list(buffer.upcast_ref());
            }
            Action::FormatNumberedList => {
                crate::editor::formatting::apply_numbered_list(buffer.upcast_ref());
            }
            Action::FormatCode => {
                crate::editor::formatting::apply_code(buffer.upcast_ref());
            }
            Action::FormatBlockquote => {
                crate::editor::formatting::apply_blockquote(buffer.upcast_ref());
            }
            Action::Indent => {
                crate::editor::formatting::apply_indent(buffer.upcast_ref());
            }
            Action::Unindent => {
                crate::editor::formatting::apply_unindent(buffer.upcast_ref());
            }
            Action::ZoomIn => self.change_zoom(1.5),
            Action::ZoomOut => self.change_zoom(-1.5),
            Action::ZoomReset => self.reset_zoom(),
            Action::ToggleConceal => self.toggle_conceal(),
        }
    }

    fn toggle_conceal(&self) {
        let conceal = self.editor_view.conceal();
        conceal.set_enabled(!conceal.is_enabled());
    }

    fn change_zoom(&self, delta: f64) {
        let mut s = self.state.borrow_mut();
        s.font_size =
            (s.font_size + delta).clamp(crate::config::MIN_FONT_SIZE, crate::config::MAX_FONT_SIZE);
        let size = s.font_size;
        let family = s.font_family.clone();
        s.config.font_size = size;
        let _ = s.config.save();
        drop(s);
        self.editor_view.set_font(size, family.as_deref());
    }

    fn reset_zoom(&self) {
        let mut s = self.state.borrow_mut();
        s.font_size = crate::config::DEFAULT_FONT_SIZE;
        let size = s.font_size;
        let family = s.font_family.clone();
        s.config.font_size = size;
        let _ = s.config.save();
        drop(s);
        self.editor_view.set_font(size, family.as_deref());
    }

    pub fn toggle_preview(&self) {
        let current = self.stack.visible_child_name();
        if current.as_deref() == Some("editor") {
            let text = self.editor_view.buffer().text();
            self.preview_view.render_markdown(&text);
            self.stack.set_visible_child_name("preview");
        } else {
            self.stack.set_visible_child_name("editor");
            self.editor_view.grab_focus();
        }
    }

    fn setup_drag_and_drop(self: &Rc<Self>) {
        let target = gtk4::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        let this = Rc::downgrade(self);

        target.connect_drop(move |_, value, _, _| {
            let Ok(file_list) = value.get::<gdk::FileList>() else {
                return false;
            };
            let files = file_list.files();
            let Some(file) = files.first() else {
                return false;
            };
            let Some(path) = file.path() else {
                return false;
            };
            let Some(this) = this.upgrade() else {
                return false;
            };
            this.prompt_unsaved_if_dirty(PendingAction::OpenPath(path));
            true
        });

        self.window.add_controller(target);
    }

    fn setup_close_handler(self: &Rc<Self>) {
        let this = Rc::downgrade(self);
        self.window.connect_close_request(move |_| {
            if let Some(this) = this.upgrade() {
                let is_dirty = this.state.borrow().document.is_dirty();
                if is_dirty {
                    this.prompt_unsaved_if_dirty(PendingAction::QuitApp);
                    return glib::Propagation::Stop;
                }
                this.save_window_geometry();
            }
            glib::Propagation::Proceed
        });
    }

    fn save_window_geometry(&self) {
        let mut s = self.state.borrow_mut();
        s.config.window_width = self.window.default_width();
        s.config.window_height = self.window.default_height();
        s.config.is_maximized = self.window.is_maximized();
        let _ = s.config.save();
    }

    fn setup_autosave(self: &Rc<Self>) {
        let this = Rc::downgrade(self);

        glib::timeout_add_seconds_local(5, move || {
            if let Some(this) = this.upgrade() {
                let interval_secs = this.state.borrow().config.auto_save_interval.as_seconds();
                if let Some(_secs) = interval_secs {
                    let should_save = {
                        let s = this.state.borrow();
                        s.document.is_dirty() && s.document.path().is_some()
                    };
                    if should_save {
                        this.save_document(true);
                    }
                }
                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    fn setup_focus_watcher(self: &Rc<Self>) {
        let this = Rc::downgrade(self);
        self.window.connect_is_active_notify(move |win| {
            if !win.is_active() {
                return;
            }
            if let Some(this) = this.upgrade() {
                this.check_external_changes();
            }
        });
    }

    fn check_external_changes(self: &Rc<Self>) {
        let (path, last_mtime) = {
            let s = self.state.borrow();
            match (s.document.path(), s.document.last_mtime()) {
                (Some(p), Some(m)) => (p.to_path_buf(), m),
                _ => return,
            }
        };

        if let Ok(true) = check_external_modification(&path, last_mtime) {
            self.show_external_conflict_dialog(&path);
        }
    }

    fn show_external_conflict_dialog(self: &Rc<Self>, path: &Path) {
        let dialog = libadwaita::AlertDialog::builder()
            .heading("Le fichier a été modifié ailleurs.")
            .body("Le document sur le disque est plus récent que la version en cours d'édition.")
            .build();

        dialog.add_response("reload", "Recharger");
        dialog.add_response("keep", "Conserver ma version");
        dialog.add_response("diff", "Comparer");
        dialog.set_response_appearance("reload", libadwaita::ResponseAppearance::Destructive);
        dialog.set_response_appearance("keep", libadwaita::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("keep"));

        let this = Rc::downgrade(self);
        let path_buf = path.to_path_buf();

        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |response| {
                if let Some(this) = this.upgrade() {
                    match response.as_str() {
                        "reload" => {
                            this.load_file(&path_buf);
                            this.show_toast("Document rechargé depuis le disque.");
                        }
                        "keep" => {
                            if let Ok(mtime) =
                                std::fs::metadata(&path_buf).and_then(|m| m.modified())
                            {
                                this.state.borrow_mut().document.set_last_mtime(Some(mtime));
                            }
                        }
                        "diff" => {
                            this.show_diff_dialog(&path_buf);
                        }
                        _ => {}
                    }
                }
            },
        );
    }

    fn show_diff_dialog(self: &Rc<Self>, path: &Path) {
        let (disk_content, _) = match read_file(path) {
            Ok(res) => res,
            Err(e) => {
                self.show_toast(&format!("Erreur lors de la lecture du fichier : {e}"));
                return;
            }
        };

        let mem_content = self.editor_view.buffer().text();
        let diff_text = generate_diff(&disk_content, &mem_content);

        let dialog = libadwaita::AlertDialog::builder()
            .heading("Différences avec le disque")
            .body(&diff_text)
            .build();

        dialog.add_response("close", "Fermer");
        dialog.choose(Some(&self.window), None::<&gio::Cancellable>, |_| {});
    }

    pub fn save_document(self: &Rc<Self>, is_auto: bool) {
        let path = {
            let s = self.state.borrow();
            s.document.path().map(|p| p.to_path_buf())
        };

        match path {
            Some(p) => self.perform_save_to_path(&p, is_auto),
            None => {
                if !is_auto {
                    self.save_as_dialog();
                }
            }
        }
    }

    fn perform_save_to_path(self: &Rc<Self>, path: &Path, is_auto: bool) {
        self.state.borrow_mut().document.mark_saving();
        let content = self.editor_view.buffer().text();

        match write_file_atomic(path, &content) {
            Ok(mtime) => {
                {
                    let mut s = self.state.borrow_mut();
                    s.document.set_path(path.to_path_buf());
                    s.document.mark_clean(mtime);
                    s.config.add_recent_file(path.to_path_buf());
                    let _ = s.config.save();
                }
                self.editor_view.buffer().set_modified(false);
                self.update_title();

                if !is_auto {
                    self.show_toast("Document enregistré.");
                }
            }
            Err(err) => {
                let err_msg = err.to_string();
                self.state.borrow_mut().document.mark_error(&err_msg);
                self.update_title();
                self.show_toast(&format!("Erreur d'enregistrement : {err_msg}"));
            }
        }
    }

    fn set_dialog_folder(&self, dialog: &gtk4::FileDialog) {
        let Some(save_dir) = self.state.borrow().config.resolved_save_directory() else {
            return;
        };
        let file = gio::File::for_path(&save_dir);
        dialog.set_initial_folder(Some(&file));
    }

    pub fn save_as_dialog(self: &Rc<Self>) {
        let file_dialog = gtk4::FileDialog::builder()
            .title("Enregistrer sous...")
            .initial_name("Sans titre.md")
            .build();
        self.set_dialog_folder(&file_dialog);

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Fichiers Markdown (*.md)"));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");

        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        file_dialog.set_filters(Some(&filters));

        let this = Rc::downgrade(self);
        file_dialog.save(Some(&self.window), None::<&gio::Cancellable>, move |res| {
            let Ok(file) = res else {
                return;
            };
            let Some(path) = file.path() else {
                return;
            };
            let Some(this) = this.upgrade() else {
                return;
            };
            this.perform_save_to_path(&path, false);
            let pending = this.state.borrow_mut().pending_action.take();
            if let Some(pending) = pending {
                this.execute_pending_action(pending);
            }
        });
    }

    pub fn open_dialog(self: &Rc<Self>) {
        let file_dialog = gtk4::FileDialog::builder()
            .title("Ouvrir un document Markdown")
            .build();
        self.set_dialog_folder(&file_dialog);

        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Fichiers Markdown (*.md)"));
        filter.add_pattern("*.md");
        filter.add_pattern("*.markdown");

        let all_filter = gtk4::FileFilter::new();
        all_filter.set_name(Some("Tous les fichiers"));
        all_filter.add_pattern("*");

        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        filters.append(&all_filter);
        file_dialog.set_filters(Some(&filters));

        let this = Rc::downgrade(self);
        file_dialog.open(Some(&self.window), None::<&gio::Cancellable>, move |res| {
            let Ok(file) = res else {
                return;
            };
            let Some(path) = file.path() else {
                return;
            };
            let Some(this) = this.upgrade() else {
                return;
            };
            this.load_file(&path);
        });
    }

    pub fn load_file(self: &Rc<Self>, path: &Path) {
        match read_file(path) {
            Ok((content, mtime)) => {
                {
                    let mut s = self.state.borrow_mut();
                    s.is_reloading = true;
                    s.document = Document::from_path(path.to_path_buf(), mtime);
                    s.config.add_recent_file(path.to_path_buf());
                    let _ = s.config.save();
                }
                self.editor_view.buffer().set_text(&content);
                self.state.borrow_mut().is_reloading = false;
                self.update_title();
                self.editor_view.grab_focus();
            }
            Err(e) => {
                self.show_toast(&format!("Impossible d'ouvrir le fichier : {e}"));
            }
        }
    }

    pub fn new_document(self: &Rc<Self>) {
        {
            let mut s = self.state.borrow_mut();
            s.is_reloading = true;
            s.document = Document::new_untitled();
        }
        self.editor_view.buffer().clear();
        self.state.borrow_mut().is_reloading = false;
        self.update_title();
        self.editor_view.grab_focus();
    }

    fn prompt_unsaved_if_dirty(self: &Rc<Self>, pending: PendingAction) {
        if !self.state.borrow().document.is_dirty() {
            self.execute_pending_action(pending);
            return;
        }

        self.state.borrow_mut().pending_action = Some(pending);

        let doc_name = self.state.borrow().document.display_name();
        let dialog = libadwaita::AlertDialog::builder()
            .heading("Enregistrer les modifications ?")
            .body(format!(
                "Le document \"{doc_name}\" contient des modifications non enregistrées."
            ))
            .build();

        dialog.add_response("cancel", "Annuler");
        dialog.add_response("discard", "Ne pas sauvegarder");
        dialog.add_response("save", "Sauvegarder");
        dialog.set_response_appearance("discard", libadwaita::ResponseAppearance::Destructive);
        dialog.set_response_appearance("save", libadwaita::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("save"));
        dialog.set_close_response("cancel");

        let this = Rc::downgrade(self);
        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |response| {
                if let Some(this) = this.upgrade() {
                    match response.as_str() {
                        "save" => {
                            let has_path = this.state.borrow().document.path().is_some();
                            if has_path {
                                this.save_document(false);
                                let act = this.state.borrow_mut().pending_action.take();
                                if let Some(act) = act {
                                    this.execute_pending_action(act);
                                }
                            } else {
                                this.save_as_dialog();
                            }
                        }
                        "discard" => {
                            this.state
                                .borrow_mut()
                                .document
                                .mark_clean(SystemTime::now());
                            let act = this.state.borrow_mut().pending_action.take();
                            if let Some(act) = act {
                                this.execute_pending_action(act);
                            }
                        }
                        _ => {
                            this.state.borrow_mut().pending_action = None;
                        }
                    }
                }
            },
        );
    }

    fn execute_pending_action(self: &Rc<Self>, action: PendingAction) {
        match action {
            PendingAction::NewDocument => self.new_document(),
            PendingAction::OpenPath(path) => self.load_file(&path),
            PendingAction::OpenDialog => self.open_dialog(),
            PendingAction::CloseDocument => {
                self.new_document();
            }
            PendingAction::QuitApp => {
                self.save_window_geometry();
                self.window.close();
            }
        }
    }

    pub fn update_title(&self) {
        let s = self.state.borrow();
        let title = s.document.window_title();
        self.window.set_title(Some(&title));
    }

    pub fn show_toast(&self, text: &str) {
        let toast = libadwaita::Toast::builder().title(text).timeout(3).build();
        self.toast_overlay.add_toast(toast);
    }
}
