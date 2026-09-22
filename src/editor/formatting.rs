use super::markdown;
use gtk4::prelude::*;

pub fn apply_toggle_wrap(buffer: &gtk4::TextBuffer, prefix: &str, suffix: &str) {
    buffer.begin_user_action();
    if let Some((mut start, mut end)) = buffer.selection_bounds() {
        let text = buffer.text(&start, &end, false);
        let (new_text, sel_start, sel_end) = markdown::toggle_wrap(&text, prefix, suffix);
        let start_offset = start.offset();
        buffer.delete(&mut start, &mut end);
        buffer.insert(&mut start, &new_text);

        let iter_start = buffer.iter_at_offset(start_offset + sel_start as i32);
        let iter_end = buffer.iter_at_offset(start_offset + sel_end as i32);
        buffer.select_range(&iter_start, &iter_end);
    } else {
        let mut cursor = buffer.iter_at_offset(buffer.cursor_position());
        let (template, sel_start, sel_end) = markdown::toggle_wrap("", prefix, suffix);
        let offset = cursor.offset();
        buffer.insert(&mut cursor, &template);

        let iter_start = buffer.iter_at_offset(offset + sel_start as i32);
        let iter_end = buffer.iter_at_offset(offset + sel_end as i32);
        buffer.select_range(&iter_start, &iter_end);
    }
    buffer.end_user_action();
}

pub fn apply_link(buffer: &gtk4::TextBuffer) {
    buffer.begin_user_action();
    if let Some((mut start, mut end)) = buffer.selection_bounds() {
        let text = buffer.text(&start, &end, false);
        let (new_text, sel_start, sel_end) = markdown::wrap_link(&text);
        let start_offset = start.offset();
        buffer.delete(&mut start, &mut end);
        buffer.insert(&mut start, &new_text);

        let iter_start = buffer.iter_at_offset(start_offset + sel_start as i32);
        let iter_end = buffer.iter_at_offset(start_offset + sel_end as i32);
        buffer.select_range(&iter_start, &iter_end);
    } else {
        let mut cursor = buffer.iter_at_offset(buffer.cursor_position());
        let (template, sel_start, sel_end) = markdown::wrap_link("");
        let offset = cursor.offset();
        buffer.insert(&mut cursor, &template);

        let iter_start = buffer.iter_at_offset(offset + sel_start as i32);
        let iter_end = buffer.iter_at_offset(offset + sel_end as i32);
        buffer.select_range(&iter_start, &iter_end);
    }
    buffer.end_user_action();
}

pub fn apply_code(buffer: &gtk4::TextBuffer) {
    buffer.begin_user_action();
    if let Some((mut start, mut end)) = buffer.selection_bounds() {
        let text = buffer.text(&start, &end, false);
        let (new_text, sel_start, sel_end) = markdown::wrap_code(&text);
        let start_offset = start.offset();
        buffer.delete(&mut start, &mut end);
        buffer.insert(&mut start, &new_text);

        let iter_start = buffer.iter_at_offset(start_offset + sel_start as i32);
        let iter_end = buffer.iter_at_offset(start_offset + sel_end as i32);
        buffer.select_range(&iter_start, &iter_end);
    } else {
        let mut cursor = buffer.iter_at_offset(buffer.cursor_position());
        let (template, sel_start, sel_end) = markdown::wrap_code("");
        let offset = cursor.offset();
        buffer.insert(&mut cursor, &template);

        let iter_start = buffer.iter_at_offset(offset + sel_start as i32);
        let iter_end = buffer.iter_at_offset(offset + sel_end as i32);
        buffer.select_range(&iter_start, &iter_end);
    }
    buffer.end_user_action();
}

fn apply_line_transform<F>(buffer: &gtk4::TextBuffer, transform: F)
where
    F: Fn(&str) -> String,
{
    buffer.begin_user_action();
    let (start_line, end_line) = if let Some((start, end)) = buffer.selection_bounds() {
        (start.line(), end.line())
    } else {
        let cursor = buffer.iter_at_offset(buffer.cursor_position());
        (cursor.line(), cursor.line())
    };

    let mut line_start = buffer.iter_at_line(start_line).unwrap_or_else(|| buffer.start_iter());
    let mut line_end = buffer.iter_at_line(end_line).unwrap_or_else(|| buffer.end_iter());
    if !line_end.ends_line() {
        line_end.forward_to_line_end();
    }

    let original_text = buffer.text(&line_start, &line_end, false);
    let lines: Vec<&str> = original_text.split('\n').collect();
    let transformed: Vec<String> = lines.into_iter().map(transform).collect();
    let new_text = transformed.join("\n");

    buffer.delete(&mut line_start, &mut line_end);
    buffer.insert(&mut line_start, &new_text);
    buffer.end_user_action();
}

pub fn apply_cycle_heading(buffer: &gtk4::TextBuffer) {
    apply_line_transform(buffer, markdown::cycle_heading_prefix);
}

pub fn apply_bullet_list(buffer: &gtk4::TextBuffer) {
    apply_line_transform(buffer, markdown::toggle_bullet_list);
}

pub fn apply_blockquote(buffer: &gtk4::TextBuffer) {
    apply_line_transform(buffer, markdown::toggle_blockquote);
}

pub fn apply_numbered_list(buffer: &gtk4::TextBuffer) {
    buffer.begin_user_action();
    let (start_line, end_line) = if let Some((start, end)) = buffer.selection_bounds() {
        (start.line(), end.line())
    } else {
        let cursor = buffer.iter_at_offset(buffer.cursor_position());
        (cursor.line(), cursor.line())
    };

    let mut line_start = buffer.iter_at_line(start_line).unwrap_or_else(|| buffer.start_iter());
    let mut line_end = buffer.iter_at_line(end_line).unwrap_or_else(|| buffer.end_iter());
    if !line_end.ends_line() {
        line_end.forward_to_line_end();
    }

    let original_text = buffer.text(&line_start, &line_end, false);
    let lines: Vec<&str> = original_text.split('\n').collect();
    let transformed = markdown::toggle_numbered_list(&lines);
    let new_text = transformed.join("\n");

    buffer.delete(&mut line_start, &mut line_end);
    buffer.insert(&mut line_start, &new_text);
    buffer.end_user_action();
}

pub fn apply_indent(buffer: &gtk4::TextBuffer) {
    buffer.begin_user_action();
    if let Some((start, end)) = buffer.selection_bounds() {
        let start_line = start.line();
        let end_line = end.line();
        let mut line_start = buffer.iter_at_line(start_line).unwrap_or_else(|| buffer.start_iter());
        let mut line_end = buffer.iter_at_line(end_line).unwrap_or_else(|| buffer.end_iter());
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }

        let original_text = buffer.text(&line_start, &line_end, false);
        let lines: Vec<&str> = original_text.split('\n').collect();
        let transformed = markdown::indent_text(&lines);
        let new_text = transformed.join("\n");

        buffer.delete(&mut line_start, &mut line_end);
        buffer.insert(&mut line_start, &new_text);
    } else {
        let mut cursor = buffer.iter_at_offset(buffer.cursor_position());
        buffer.insert(&mut cursor, "    ");
    }
    buffer.end_user_action();
}

pub fn apply_unindent(buffer: &gtk4::TextBuffer) {
    buffer.begin_user_action();
    let (start_line, end_line) = if let Some((start, end)) = buffer.selection_bounds() {
        (start.line(), end.line())
    } else {
        let cursor = buffer.iter_at_offset(buffer.cursor_position());
        (cursor.line(), cursor.line())
    };

    let mut line_start = buffer.iter_at_line(start_line).unwrap_or_else(|| buffer.start_iter());
    let mut line_end = buffer.iter_at_line(end_line).unwrap_or_else(|| buffer.end_iter());
    if !line_end.ends_line() {
        line_end.forward_to_line_end();
    }

    let original_text = buffer.text(&line_start, &line_end, false);
    let lines: Vec<&str> = original_text.split('\n').collect();
    let transformed = markdown::unindent_text(&lines);
    let new_text = transformed.join("\n");

    buffer.delete(&mut line_start, &mut line_end);
    buffer.insert(&mut line_start, &new_text);
    buffer.end_user_action();
}
