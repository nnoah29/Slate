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
** Boîtes de dialogue du modèle : confirmation explicite du téléchargement, puis progression.
*/

use super::install_error::ModelInstallError;
use super::model_catalog::entry_for;
use super::model_download::install_model;
use super::model_store::ModelStore;
use super::types::VoiceLanguage;
use gtk4::{gio, glib};
use libadwaita::prelude::*;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

/// Résultat du téléchargement, rappelé sur la boucle de l'interface.
pub type InstallCallback = Rc<dyn Fn(Result<(), ModelInstallError>)>;

enum Progress {
    Bytes(u64, u64),
    Done(Result<(), ModelInstallError>),
}

/// Nom de la langue pour l'affichage.
fn language_name(language: VoiceLanguage) -> &'static str {
    match language {
        VoiceLanguage::Fr => "Français",
        VoiceLanguage::En => "English",
    }
}

/// Demande confirmation (taille et source affichées) puis télécharge le modèle ; rien ne part sans accord.
pub fn offer_download(
    parent: &impl IsA<gtk4::Widget>,
    store: ModelStore,
    language: VoiceLanguage,
    on_done: InstallCallback,
) {
    let entry = entry_for(language);
    let body = format!(
        "Langue : {}\nTaille : {} Mo\nSource : github.com/k2-fsa/sherpa-onnx\n\nVotre voix reste sur votre machine : seul ce téléchargement utilise le réseau.",
        language_name(language),
        entry.archive_bytes / 1_000_000
    );
    let dialog =
        libadwaita::AlertDialog::new(Some("Télécharger le modèle de dictée ?"), Some(&body));
    dialog.add_response("cancel", "Annuler");
    dialog.add_response("download", "Télécharger");
    dialog.set_response_appearance("download", libadwaita::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("download"));
    dialog.set_close_response("cancel");
    let anchor = parent.clone().upcast::<gtk4::Widget>();
    dialog.choose(Some(parent), None::<&gio::Cancellable>, move |response| {
        if response == "download" {
            start_download(&anchor, store, language, on_done);
        }
    });
}

/// Lance le téléchargement dans un fil et affiche une barre de progression.
fn start_download(
    parent: &gtk4::Widget,
    store: ModelStore,
    language: VoiceLanguage,
    on_done: InstallCallback,
) {
    let bar = gtk4::ProgressBar::builder().show_text(true).build();
    let dialog = libadwaita::AlertDialog::new(Some("Téléchargement du modèle…"), None);
    dialog.set_extra_child(Some(&bar));
    dialog.add_response("hide", "Masquer");
    dialog.present(Some(parent));
    let (sender, receiver) = channel();
    std::thread::spawn(move || {
        let progress = sender.clone();
        let result = install_model(&store, entry_for(language), &mut |done, total| {
            let _ = progress.send(Progress::Bytes(done, total));
        });
        let _ = sender.send(Progress::Done(result));
    });
    watch_download(receiver, bar, dialog, on_done);
}

/// Suit la progression sur la boucle de l'interface ; ferme la boîte et rappelle à la fin.
fn watch_download(
    receiver: Receiver<Progress>,
    bar: gtk4::ProgressBar,
    dialog: libadwaita::AlertDialog,
    on_done: InstallCallback,
) {
    glib::timeout_add_local(Duration::from_millis(100), move || {
        while let Ok(message) = receiver.try_recv() {
            match message {
                Progress::Bytes(done, total) => {
                    bar.set_fraction(done as f64 / total.max(1) as f64);
                    bar.set_text(Some(&format!(
                        "{} / {} Mo",
                        done / 1_000_000,
                        total / 1_000_000
                    )));
                }
                Progress::Done(result) => {
                    dialog.force_close();
                    on_done(result);
                    return glib::ControlFlow::Break;
                }
            }
        }
        glib::ControlFlow::Continue
    });
}
