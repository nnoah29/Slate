mod app;
mod config;
mod document;
mod editor;
mod preview;
mod search;
mod shortcuts;
mod storage;
mod window;

use app::SlateApp;
use clap::Parser;
use gtk4::glib;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "slate", author, version, about = "Minimalist native Linux Markdown notes application")]
struct Cli {
    /// Markdown file to open
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Open a new empty note
    #[arg(short, long)]
    new: bool,

    /// Enable verbose debug logging
    #[arg(long)]
    debug: bool,
}

fn main() -> glib::ExitCode {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.debug {
        EnvFilter::new("debug,slate=trace")
    } else {
        EnvFilter::new("warn,slate=info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    tracing::debug!("Starting Slate application...");

    libadwaita::init().expect("Failed to initialize libadwaita");

    let initial_file = if cli.new {
        None
    } else {
        cli.file
    };

    let app = SlateApp::new(initial_file);
    app.run()
}
