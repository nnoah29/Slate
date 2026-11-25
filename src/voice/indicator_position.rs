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
** Position du curseur d'écriture dans la surimpression, pour poser l'indicateur à côté.
*/

use gtk4::prelude::*;

/// Coin haut-gauche du curseur dans le repère de la surimpression, ou `None` s'il est hors de la zone visible.
pub fn caret_in_overlay(view: &gtk4::TextView, overlay: &gtk4::Overlay) -> Option<(i32, i32)> {
    let buffer = view.buffer();
    let iter = buffer.iter_at_mark(&buffer.get_insert());
    let rect = view.iter_location(&iter);
    let (x, y) = view.buffer_to_window_coords(gtk4::TextWindowType::Widget, rect.x(), rect.y());
    if x < 0 || y < 0 || y + rect.height() > view.height() {
        return None;
    }
    let point = gtk4::graphene::Point::new(x as f32, y as f32);
    let moved = view.compute_point(overlay, &point)?;
    Some((moved.x() as i32, moved.y() as i32))
}
