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
** Text editor view configuration and layout wrapper.
*/

pub mod buffer;
pub mod conceal;
pub mod formatting;
pub mod markdown;

pub use buffer::EditorBuffer;
pub use conceal::ConcealController;

use gtk4::prelude::*;
use sourceview5::View;
use sourceview5::prelude::*;

pub struct EditorView {
    container: gtk4::ScrolledWindow,
    view: View,
    buffer: EditorBuffer,
    css_provider: gtk4::CssProvider,
    conceal: ConcealController,
}

impl EditorView {
    pub fn new() -> Self {
        let buffer = EditorBuffer::new();
        let view = View::with_buffer(buffer.buffer());

        view.set_show_line_numbers(false);
        view.set_show_right_margin(false);
        view.set_auto_indent(true);
        view.set_indent_width(4);
        view.set_tab_width(4);
        view.set_insert_spaces_instead_of_tabs(true);
        view.set_wrap_mode(gtk4::WrapMode::None);
        view.set_left_margin(10);
        view.set_right_margin(10);
        view.set_top_margin(10);
        view.set_bottom_margin(10);
        view.set_monospace(true);
        view.set_hexpand(true);
        view.set_vexpand(true);
        view.set_accepts_tab(false);

        let css_provider = gtk4::CssProvider::new();
        #[allow(deprecated)]
        view.style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let container = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&view)
            .hexpand(true)
            .vexpand(true)
            .build();

        let conceal = ConcealController::new(buffer.buffer().upcast_ref());

        Self {
            container,
            view,
            buffer,
            css_provider,
            conceal,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.container
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn buffer(&self) -> &EditorBuffer {
        &self.buffer
    }

    pub fn conceal(&self) -> &ConcealController {
        &self.conceal
    }

    pub fn set_font(&self, font_size: f64, font_family: Option<&str>) {
        let family_css = match font_family {
            Some(family) if !family.is_empty() => format!("font-family: '{family}', monospace;"),
            _ => "font-family: 'Iosevka Nerd Font', monospace;".to_string(),
        };

        let css = format!(
            "textview, textview text {{ font-size: {:.1}pt; line-height: 1.35; background-color: transparent; background: transparent; color: #cdd6f4; caret-color: #f5e0dc; {family_css} }}",
            font_size
        );
        self.css_provider.load_from_string(&css);
    }

    pub fn grab_focus(&self) {
        self.view.grab_focus();
    }

    pub fn toggle_wrap_mode(&self) {
        let current = self.view.wrap_mode();
        if current == gtk4::WrapMode::None {
            self.view.set_wrap_mode(gtk4::WrapMode::WordChar);
        } else {
            self.view.set_wrap_mode(gtk4::WrapMode::None);
        }
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}
