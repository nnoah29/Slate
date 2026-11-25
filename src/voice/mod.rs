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
** Dictée vocale locale : micro, reconnaissance en flux, modèles et insertion dans la note.
*/

pub mod audio_level;
pub mod backlog;
pub mod buffer_target;
pub mod controller;
pub mod engine;
mod engine_poll;
pub mod indicator;
pub mod indicator_glyph;
mod indicator_position;
pub mod inserter;
pub mod install_error;
pub mod microphone;
pub mod model_archive;
pub mod model_catalog;
pub mod model_dialog;
pub mod model_download;
pub mod model_store;
pub mod pcm;
pub mod push_to_talk;
pub mod recognizer_config;
pub mod recognizer_session;
pub mod recognizer_worker;
pub mod text_target;
pub mod transcript;
pub mod types;
