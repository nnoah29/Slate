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
** Application life-cycle, DBus IPC multi-instance, and global CSS theme integration.
*/

use crate::config::{AppConfig, ThemeMode};
use crate::window::SlateWindow;
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub const APP_ID: &str = "com.nnoah29.slate";

const CUSTOM_CSS_BASE: &str = "
scrolledwindow, scrolledwindow viewport, viewport, stack, .view {
    background-color: transparent;
    background: transparent;
}

textview text selection {
    background-color: #45475a;
    color: #cdd6f4;
}

.search-bar-card {
    background-color: #1e1e2e;
    color: #cdd6f4;
    border-radius: 10px;
    border: 1px solid alpha(#b4befe, 0.2);
    box-shadow: 0 3px 8px alpha(black, 0.25);
}

.blockquote {
    border-left: 3px solid #89b4fa;
    padding-left: 14px;
    font-style: italic;
    opacity: 0.9;
}

.code-block {
    background-color: #1e1e2e;
    border-radius: 8px;
    border: 1px solid #313244;
}

.table-view {
    margin: 4px 0;
}

.table-cell {
    padding: 6px 12px;
    border: 1px solid #313244;
}

.dim-label {
    color: #a6adc8;
}
";

pub fn generate_custom_css(
    opacity: f64,
    blur: f64,
    font_size: f64,
    font_family: Option<&str>,
) -> String {
    let backdrop = if blur > 0.0 {
        format!("    backdrop-filter: blur({blur:.1}px);\n")
    } else {
        String::new()
    };
    let family = font_family.unwrap_or("Iosevka Nerd Font");
    format!(
        "{CUSTOM_CSS_BASE}
window, window.background, .background {{
    background-color: rgba(39, 38, 38, {opacity:.2});
    background: rgba(39, 38, 38, {opacity:.2});
    color: #cdd6f4;
    border-radius: 12px;
    border: none;
    box-shadow: none;
    outline: none;
{backdrop}}}

textview, textview text {{
    font-family: '{family}', monospace;
    font-size: {font_size:.1}pt;
    line-height: 1.35;
    background-color: transparent;
    background: transparent;
    color: #cdd6f4;
    caret-color: #f5e0dc;
}}
"
    )
}

pub struct SlateApp {
    app: libadwaita::Application,
    active_window: Rc<RefCell<Option<Rc<SlateWindow>>>>,
    initial_file: Rc<RefCell<Option<PathBuf>>>,
}

impl SlateApp {
    pub fn new(initial_file: Option<PathBuf>) -> Self {
        let app = libadwaita::Application::builder()
            .application_id(APP_ID)
            .flags(
                gio::ApplicationFlags::HANDLES_COMMAND_LINE | gio::ApplicationFlags::HANDLES_OPEN,
            )
            .build();

        let active_window = Rc::new(RefCell::new(None));
        let initial_file = Rc::new(RefCell::new(initial_file));

        Self {
            app,
            active_window,
            initial_file,
        }
    }

    pub fn run(&self) -> glib::ExitCode {
        self.setup_signals();
        self.app.run()
    }

    fn setup_signals(&self) {
        let active_win = self.active_window.clone();
        let init_file = self.initial_file.clone();

        self.app.connect_startup(|_| {
            let config = AppConfig::load();

            let style_mgr = libadwaita::StyleManager::default();
            match config.theme {
                ThemeMode::System => style_mgr.set_color_scheme(libadwaita::ColorScheme::Default),
                ThemeMode::Light => style_mgr.set_color_scheme(libadwaita::ColorScheme::ForceLight),
                ThemeMode::Dark => style_mgr.set_color_scheme(libadwaita::ColorScheme::ForceDark),
            }

            if let Some(display) = gdk::Display::default() {
                let provider = gtk4::CssProvider::new();
                let css = generate_custom_css(
                    config.opacity,
                    config.blur,
                    config.font_size,
                    config.font_family.as_deref(),
                );
                provider.load_from_string(&css);
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        });

        {
            let active_win = active_win.clone();
            let init_file = init_file.clone();
            self.app.connect_activate(move |app| {
                if let Some(win) = active_win.borrow().as_ref() {
                    win.window().present();
                    return;
                }

                let path = init_file.borrow_mut().take();
                let config = AppConfig::load();
                let slate_win = SlateWindow::new(app, path, config);
                slate_win.window().present();
                *active_win.borrow_mut() = Some(slate_win);
            });
        }

        {
            let active_win = active_win.clone();
            self.app.connect_open(move |app, files, _| {
                let first_file = files.first().and_then(|f| f.path());

                if let Some(win) = active_win.borrow().as_ref() {
                    if let Some(path) = first_file {
                        win.load_file(&path);
                    }
                    win.window().present();
                } else {
                    let config = AppConfig::load();
                    let slate_win = SlateWindow::new(app, first_file, config);
                    slate_win.window().present();
                    *active_win.borrow_mut() = Some(slate_win);
                }
            });
        }

        {
            let active_win = active_win.clone();
            self.app.connect_command_line(move |app, cmd_line| {
                let args = cmd_line.arguments();
                let mut target_file: Option<PathBuf> = None;

                for arg in args.iter().skip(1) {
                    let s = arg.to_str().unwrap_or("");
                    if !s.starts_with('-') {
                        let path = PathBuf::from(s);
                        target_file = Some(path);
                        break;
                    }
                }

                if let Some(win) = active_win.borrow().as_ref() {
                    if let Some(path) = target_file {
                        win.load_file(&path);
                    }
                    win.window().present();
                } else {
                    let config = AppConfig::load();
                    let slate_win = SlateWindow::new(app, target_file, config);
                    slate_win.window().present();
                    *active_win.borrow_mut() = Some(slate_win);
                }

                glib::ExitCode::SUCCESS
            });
        }
    }
}
