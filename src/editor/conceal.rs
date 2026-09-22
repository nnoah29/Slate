use gtk4::pango;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub const CONCEAL_TAG: &str = "slate_conceal";
pub const BOLD_TAG: &str = "slate_bold";
pub const ITALIC_TAG: &str = "slate_italic";
pub const STRIKE_TAG: &str = "slate_strike";
pub const CODE_TAG: &str = "slate_code";
pub const LINK_TAG: &str = "slate_link";
pub const QUOTE_TAG: &str = "slate_quote";
pub const H1_TAG: &str = "slate_h1";
pub const H2_TAG: &str = "slate_h2";
pub const H3_TAG: &str = "slate_h3";
pub const H4_TAG: &str = "slate_h4";
pub const H5_TAG: &str = "slate_h5";
pub const H6_TAG: &str = "slate_h6";

pub const ALL_SLATE_TAGS: &[&str] = &[
    CONCEAL_TAG,
    BOLD_TAG,
    ITALIC_TAG,
    STRIKE_TAG,
    CODE_TAG,
    LINK_TAG,
    QUOTE_TAG,
    H1_TAG,
    H2_TAG,
    H3_TAG,
    H4_TAG,
    H5_TAG,
    H6_TAG,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanType {
    #[allow(dead_code)]
    Conceal,
    Bold,
    Italic,
    Strike,
    Code,
    Link,
    Quote,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl SpanType {
    pub fn tag_name(&self) -> &'static str {
        match self {
            SpanType::Conceal => CONCEAL_TAG,
            SpanType::Bold => BOLD_TAG,
            SpanType::Italic => ITALIC_TAG,
            SpanType::Strike => STRIKE_TAG,
            SpanType::Code => CODE_TAG,
            SpanType::Link => LINK_TAG,
            SpanType::Quote => QUOTE_TAG,
            SpanType::H1 => H1_TAG,
            SpanType::H2 => H2_TAG,
            SpanType::H3 => H3_TAG,
            SpanType::H4 => H4_TAG,
            SpanType::H5 => H5_TAG,
            SpanType::H6 => H6_TAG,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct SpanToken {
    /// Start offset in Unicode characters (0-indexed) within the line
    pub start: usize,
    /// End offset in Unicode characters (exclusive) within the line
    pub end: usize,
    pub span_type: SpanType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownElement {
    pub start: usize,
    pub end: usize,
    pub is_line_block: bool,
    pub delims: Vec<(usize, usize)>,
    pub styles: Vec<(usize, usize, SpanType)>,
}

/// Parses a line of Markdown into discrete MarkdownElement structures.
pub fn parse_markdown_elements(line: &str) -> Vec<MarkdownElement> {
    let mut elements = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    if n == 0 {
        return elements;
    }

    let mut indent = 0;
    while indent < n && chars[indent] == ' ' {
        indent += 1;
    }

    // 1. Headings: ^( *#{1,6} )(.*)
    let mut hashes = 0;
    let mut i = indent;
    while i < n && chars[i] == '#' {
        hashes += 1;
        i += 1;
    }
    if hashes >= 1 && hashes <= 6 && i < n && chars[i] == ' ' {
        let delim_end = i + 1;
        let heading_type = match hashes {
            1 => SpanType::H1,
            2 => SpanType::H2,
            3 => SpanType::H3,
            4 => SpanType::H4,
            5 => SpanType::H5,
            _ => SpanType::H6,
        };

        elements.push(MarkdownElement {
            start: indent,
            end: n,
            is_line_block: true,
            delims: vec![(indent, delim_end)],
            styles: vec![(delim_end, n, heading_type)],
        });

        parse_inline_elements(&chars, delim_end, n, &mut elements);
        return elements;
    }

    // 2. Blockquotes: ^( *> )(.*)
    if indent + 1 < n && chars[indent] == '>' && chars[indent + 1] == ' ' {
        elements.push(MarkdownElement {
            start: indent,
            end: n,
            is_line_block: true,
            delims: vec![(indent, indent + 2)],
            styles: vec![(indent + 2, n, SpanType::Quote)],
        });
        parse_inline_elements(&chars, indent + 2, n, &mut elements);
        return elements;
    }

    // 3. Checkboxes: ^( *[-*+] \[[ xX]\] )(.*)
    if indent + 5 < n
        && (chars[indent] == '-' || chars[indent] == '*' || chars[indent] == '+')
        && chars[indent + 1] == ' '
        && chars[indent + 2] == '['
        && (chars[indent + 3] == ' ' || chars[indent + 3] == 'x' || chars[indent + 3] == 'X')
        && chars[indent + 4] == ']'
        && chars[indent + 5] == ' '
    {
        let is_checked = chars[indent + 3] == 'x' || chars[indent + 3] == 'X';
        if is_checked {
            elements.push(MarkdownElement {
                start: indent + 6,
                end: n,
                is_line_block: true,
                delims: Vec::new(),
                styles: vec![(indent + 6, n, SpanType::Strike)],
            });
        }
        parse_inline_elements(&chars, indent + 6, n, &mut elements);
        return elements;
    }

    // 4. Code block fence line: ^( *```.*)
    if indent + 2 < n && chars[indent] == '`' && chars[indent + 1] == '`' && chars[indent + 2] == '`' {
        elements.push(MarkdownElement {
            start: indent,
            end: n,
            is_line_block: true,
            delims: Vec::new(),
            styles: vec![(indent, n, SpanType::Code)],
        });
        return elements;
    }

    // 5. Standard line inline elements
    parse_inline_elements(&chars, 0, n, &mut elements);
    elements
}

fn parse_inline_elements(chars: &[char], start: usize, end: usize, elements: &mut Vec<MarkdownElement>) {
    let mut i = start;
    while i < end {
        // Inline code: `...`
        if chars[i] == '`' {
            if let Some(close) = find_closing_char(chars, i + 1, end, '`') {
                if close > i + 1 {
                    elements.push(MarkdownElement {
                        start: i,
                        end: close + 1,
                        is_line_block: false,
                        delims: vec![(i, i + 1), (close, close + 1)],
                        styles: vec![(i + 1, close, SpanType::Code)],
                    });
                    i = close + 1;
                    continue;
                }
            }
        }

        // Markdown link: [text](url)
        if chars[i] == '[' {
            if let Some(close_bracket) = find_closing_char(chars, i + 1, end, ']') {
                if close_bracket + 1 < end && chars[close_bracket + 1] == '(' {
                    if let Some(close_paren) = find_closing_char(chars, close_bracket + 2, end, ')') {
                        elements.push(MarkdownElement {
                            start: i,
                            end: close_paren + 1,
                            is_line_block: false,
                            delims: vec![(i, i + 1), (close_bracket, close_paren + 1)],
                            styles: vec![(i + 1, close_bracket, SpanType::Link)],
                        });
                        i = close_paren + 1;
                        continue;
                    }
                }
            }
        }

        // Bold + Italic: ***text***
        if i + 2 < end && chars[i] == '*' && chars[i + 1] == '*' && chars[i + 2] == '*' {
            if let Some(close) = find_closing_triple(chars, i + 3, end, '*') {
                if close > i + 3 {
                    elements.push(MarkdownElement {
                        start: i,
                        end: close + 3,
                        is_line_block: false,
                        delims: vec![(i, i + 3), (close, close + 3)],
                        styles: vec![
                            (i + 3, close, SpanType::Bold),
                            (i + 3, close, SpanType::Italic),
                        ],
                    });
                    i = close + 3;
                    continue;
                }
            }
        }

        // Bold: **text**
        if i + 1 < end && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(close) = find_closing_double(chars, i + 2, end, '*') {
                if close > i + 2 {
                    elements.push(MarkdownElement {
                        start: i,
                        end: close + 2,
                        is_line_block: false,
                        delims: vec![(i, i + 2), (close, close + 2)],
                        styles: vec![(i + 2, close, SpanType::Bold)],
                    });
                    i = close + 2;
                    continue;
                }
            }
        }

        // Bold: __text__
        if i + 1 < end && chars[i] == '_' && chars[i + 1] == '_' {
            if let Some(close) = find_closing_double(chars, i + 2, end, '_') {
                if close > i + 2 {
                    elements.push(MarkdownElement {
                        start: i,
                        end: close + 2,
                        is_line_block: false,
                        delims: vec![(i, i + 2), (close, close + 2)],
                        styles: vec![(i + 2, close, SpanType::Bold)],
                    });
                    i = close + 2;
                    continue;
                }
            }
        }

        // Strikethrough: ~~text~~
        if i + 1 < end && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(close) = find_closing_double(chars, i + 2, end, '~') {
                if close > i + 2 {
                    elements.push(MarkdownElement {
                        start: i,
                        end: close + 2,
                        is_line_block: false,
                        delims: vec![(i, i + 2), (close, close + 2)],
                        styles: vec![(i + 2, close, SpanType::Strike)],
                    });
                    i = close + 2;
                    continue;
                }
            }
        }

        // Italic: *text*
        if chars[i] == '*' && (i + 1 < end && chars[i + 1] != '*' && chars[i + 1] != ' ') {
            if let Some(close) = find_closing_single_asterisk(chars, i + 1, end) {
                if close > i + 1 {
                    elements.push(MarkdownElement {
                        start: i,
                        end: close + 1,
                        is_line_block: false,
                        delims: vec![(i, i + 1), (close, close + 1)],
                        styles: vec![(i + 1, close, SpanType::Italic)],
                    });
                    i = close + 1;
                    continue;
                }
            }
        }

        // Italic: _text_
        if chars[i] == '_' && (i + 1 < end && chars[i + 1] != '_' && chars[i + 1] != ' ') {
            let is_boundary_before = i == 0 || !chars[i - 1].is_alphanumeric();
            if is_boundary_before {
                if let Some(close) = find_closing_single_underscore(chars, i + 1, end) {
                    let is_boundary_after = close + 1 >= end || !chars[close + 1].is_alphanumeric();
                    if is_boundary_after && close > i + 1 {
                        elements.push(MarkdownElement {
                            start: i,
                            end: close + 1,
                            is_line_block: false,
                            delims: vec![(i, i + 1), (close, close + 1)],
                            styles: vec![(i + 1, close, SpanType::Italic)],
                        });
                        i = close + 1;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }
}

fn find_closing_char(chars: &[char], start: usize, end: usize, target: char) -> Option<usize> {
    for j in start..end {
        if chars[j] == target && (j == start || chars[j - 1] != '\\') {
            return Some(j);
        }
    }
    None
}

fn find_closing_double(chars: &[char], start: usize, end: usize, target: char) -> Option<usize> {
    let mut j = start;
    while j + 1 < end {
        if chars[j] == target && chars[j + 1] == target && (j == start || chars[j - 1] != '\\') {
            if j > start && chars[j - 1] != ' ' {
                return Some(j);
            }
        }
        j += 1;
    }
    None
}

fn find_closing_triple(chars: &[char], start: usize, end: usize, target: char) -> Option<usize> {
    let mut j = start;
    while j + 2 < end {
        if chars[j] == target
            && chars[j + 1] == target
            && chars[j + 2] == target
            && (j == start || chars[j - 1] != '\\')
        {
            if j > start && chars[j - 1] != ' ' {
                return Some(j);
            }
        }
        j += 1;
    }
    None
}

fn find_closing_single_asterisk(chars: &[char], start: usize, end: usize) -> Option<usize> {
    let mut j = start;
    while j < end {
        if chars[j] == '*' && (j == start || chars[j - 1] != '\\') {
            let not_prev_star = j == 0 || chars[j - 1] != '*';
            let not_next_star = j + 1 >= end || chars[j + 1] != '*';
            let not_preceded_by_space = j > start && chars[j - 1] != ' ';
            if not_prev_star && not_next_star && not_preceded_by_space {
                return Some(j);
            }
        }
        j += 1;
    }
    None
}

fn find_closing_single_underscore(chars: &[char], start: usize, end: usize) -> Option<usize> {
    let mut j = start;
    while j < end {
        if chars[j] == '_' && (j == start || chars[j - 1] != '\\') {
            let not_prev = j == 0 || chars[j - 1] != '_';
            let not_next = j + 1 >= end || chars[j + 1] != '_';
            let not_preceded_by_space = j > start && chars[j - 1] != ' ';
            if not_prev && not_next && not_preceded_by_space {
                return Some(j);
            }
        }
        j += 1;
    }
    None
}

/// Backwards compatibility helper returning individual SpanTokens
#[allow(dead_code)]
pub fn parse_markdown_spans(line: &str) -> Vec<SpanToken> {
    let elements = parse_markdown_elements(line);
    let mut spans = Vec::new();
    for elem in elements {
        for (d_start, d_end) in elem.delims {
            spans.push(SpanToken {
                start: d_start,
                end: d_end,
                span_type: SpanType::Conceal,
            });
        }
        for (s_start, s_end, span_type) in elem.styles {
            spans.push(SpanToken {
                start: s_start,
                end: s_end,
                span_type,
            });
        }
    }
    spans.sort_by_key(|s| s.start);
    spans
}

pub fn register_tags(tag_table: &gtk4::TextTagTable) {
    if tag_table.lookup(BOLD_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(BOLD_TAG)
            .weight(700)
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(ITALIC_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(ITALIC_TAG)
            .style(pango::Style::Italic)
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(STRIKE_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(STRIKE_TAG)
            .strikethrough(true)
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(CODE_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(CODE_TAG)
            .family("monospace")
            .background("#313244")
            .foreground("#f5c2e7")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(LINK_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(LINK_TAG)
            .foreground("#89b4fa")
            .underline(pango::Underline::Single)
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(QUOTE_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(QUOTE_TAG)
            .style(pango::Style::Italic)
            .foreground("#bac2de")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(H1_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(H1_TAG)
            .weight(700)
            .scale(1.4)
            .foreground("#89b4fa")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(H2_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(H2_TAG)
            .weight(700)
            .scale(1.25)
            .foreground("#a6e3a1")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(H3_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(H3_TAG)
            .weight(700)
            .scale(1.15)
            .foreground("#f9e2af")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(H4_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(H4_TAG)
            .weight(700)
            .scale(1.05)
            .foreground("#fab387")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(H5_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(H5_TAG)
            .weight(700)
            .foreground("#cdd6f4")
            .build();
        tag_table.add(&tag);
    }
    if tag_table.lookup(H6_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(H6_TAG)
            .weight(700)
            .foreground("#a6adc8")
            .build();
        tag_table.add(&tag);
    }
    // Add conceal tag last for highest priority
    if tag_table.lookup(CONCEAL_TAG).is_none() {
        let tag = gtk4::TextTag::builder()
            .name(CONCEAL_TAG)
            .invisible(true)
            .build();
        tag_table.add(&tag);
    }
}

fn update_line(buffer: &gtk4::TextBuffer, line_idx: i32, cursor_info: Option<(usize, usize)>) {
    if let Some(line_start) = buffer.iter_at_line(line_idx) {
        let mut line_end = line_start;
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }

        // 1. Clear existing slate tags on this line
        let tag_table = buffer.tag_table();
        for &tag_name in ALL_SLATE_TAGS {
            if let Some(tag) = tag_table.lookup(tag_name) {
                buffer.remove_tag(&tag, &line_start, &line_end);
            }
        }

        // 2. Parse elements on this line
        let line_text = buffer.text(&line_start, &line_end, false);
        let elements = parse_markdown_elements(&line_text);

        for elem in &elements {
            let is_editing = match cursor_info {
                Some((c_start, c_end)) => {
                    if elem.is_line_block {
                        // Heading / blockquote: revealed when cursor is on this line
                        true
                    } else if c_start != c_end {
                        // Text selection on this line: revealed if selection overlaps element
                        c_start < elem.end && c_end > elem.start
                    } else {
                        // Single cursor: revealed ONLY when cursor is strictly inside the element
                        c_start > elem.start && c_start < elem.end
                    }
                }
                None => false,
            };

            // Conceal delimiters if not actively editing this element
            if !is_editing {
                if let Some(conceal_tag) = tag_table.lookup(CONCEAL_TAG) {
                    for &(d_start, d_end) in &elem.delims {
                        let mut span_start = line_start;
                        span_start.forward_chars(d_start as i32);
                        let mut span_end = line_start;
                        span_end.forward_chars(d_end as i32);
                        buffer.apply_tag(&conceal_tag, &span_start, &span_end);
                    }
                }
            }

            // Always apply formatting styles (bold, italic, code, link, strike, heading, etc.)
            for &(s_start, s_end, span_type) in &elem.styles {
                if let Some(style_tag) = tag_table.lookup(span_type.tag_name()) {
                    let mut span_start = line_start;
                    span_start.forward_chars(s_start as i32);
                    let mut span_end = line_start;
                    span_end.forward_chars(s_end as i32);
                    buffer.apply_tag(&style_tag, &span_start, &span_end);
                }
            }
        }
    }
}

fn clear_line_tags(buffer: &gtk4::TextBuffer, line_idx: i32) {
    if let Some(line_start) = buffer.iter_at_line(line_idx) {
        let mut line_end = line_start;
        if !line_end.ends_line() {
            line_end.forward_to_line_end();
        }
        let tag_table = buffer.tag_table();
        for &tag_name in ALL_SLATE_TAGS {
            if let Some(tag) = tag_table.lookup(tag_name) {
                buffer.remove_tag(&tag, &line_start, &line_end);
            }
        }
    }
}

#[derive(Clone)]
pub struct ConcealController {
    buffer: gtk4::TextBuffer,
    last_cursor_pos: Rc<Cell<i32>>,
    last_line: Rc<Cell<i32>>,
    last_line_count: Rc<Cell<i32>>,
    updating: Rc<Cell<bool>>,
    enabled: Rc<Cell<bool>>,
}

impl ConcealController {
    pub fn new(buffer: &gtk4::TextBuffer) -> Self {
        register_tags(&buffer.tag_table());

        let controller = Self {
            buffer: buffer.clone(),
            last_cursor_pos: Rc::new(Cell::new(buffer.cursor_position())),
            last_line: Rc::new(Cell::new(0)),
            last_line_count: Rc::new(Cell::new(buffer.line_count())),
            updating: Rc::new(Cell::new(false)),
            enabled: Rc::new(Cell::new(true)),
        };

        controller.setup_signals();
        controller.sync_all();
        controller
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.set(enabled);
        if enabled {
            self.sync_all();
        } else {
            self.clear_all();
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    fn setup_signals(&self) {
        let this = self.clone();
        self.buffer.connect_cursor_position_notify(move |_| {
            this.on_cursor_moved();
        });

        let this = self.clone();
        self.buffer.connect_changed(move |_| {
            this.on_content_changed();
        });
    }

    fn on_cursor_moved(&self) {
        if !self.enabled.get() || self.updating.get() {
            return;
        }
        self.updating.set(true);

        let cursor_pos = self.buffer.cursor_position();
        let old_pos = self.last_cursor_pos.get();
        let old_line = self.last_line.get();

        let cursor_iter = self.buffer.iter_at_offset(cursor_pos);
        let cur_line = cursor_iter.line();
        let cur_col = cursor_iter.line_offset() as usize;

        let sel_info = self.buffer.selection_bounds().map(|(s, e)| {
            (s.line(), s.line_offset() as usize, e.line(), e.line_offset() as usize)
        });

        if cur_line != old_line {
            // Cursor moved to a different line:
            // 1. Previous line is now inactive
            update_line(&self.buffer, old_line, None);
            // 2. New line is active
            let cursor_info = match sel_info {
                Some((sl, sc, el, ec)) if sl == cur_line && el == cur_line => Some((sc, ec)),
                _ => Some((cur_col, cur_col)),
            };
            update_line(&self.buffer, cur_line, cursor_info);
            self.last_line.set(cur_line);
            self.last_cursor_pos.set(cursor_pos);
        } else if cursor_pos != old_pos {
            // Cursor moved within the same line:
            let cursor_info = match sel_info {
                Some((sl, sc, el, ec)) if sl == cur_line && el == cur_line => Some((sc, ec)),
                _ => Some((cur_col, cur_col)),
            };
            update_line(&self.buffer, cur_line, cursor_info);
            self.last_cursor_pos.set(cursor_pos);
        }

        self.updating.set(false);
    }

    fn on_content_changed(&self) {
        if !self.enabled.get() || self.updating.get() {
            return;
        }
        self.updating.set(true);

        let current_line_count = self.buffer.line_count();
        let prev_line_count = self.last_line_count.get();

        let cursor_pos = self.buffer.cursor_position();
        let cursor_iter = self.buffer.iter_at_offset(cursor_pos);
        let cur_line = cursor_iter.line();
        let cur_col = cursor_iter.line_offset() as usize;

        let sel_info = self.buffer.selection_bounds().map(|(s, e)| {
            (s.line(), s.line_offset() as usize, e.line(), e.line_offset() as usize)
        });
        let cursor_info = match sel_info {
            Some((sl, sc, el, ec)) if sl == cur_line && el == cur_line => Some((sc, ec)),
            _ => Some((cur_col, cur_col)),
        };

        if current_line_count != prev_line_count {
            self.last_line_count.set(current_line_count);
            self.last_line.set(cur_line);
            self.last_cursor_pos.set(cursor_pos);

            for line in 0..current_line_count {
                if line == cur_line {
                    update_line(&self.buffer, line, cursor_info);
                } else {
                    update_line(&self.buffer, line, None);
                }
            }
        } else {
            self.last_line.set(cur_line);
            self.last_cursor_pos.set(cursor_pos);
            update_line(&self.buffer, cur_line, cursor_info);
        }

        self.updating.set(false);
    }

    pub fn sync_all(&self) {
        if !self.enabled.get() || self.updating.get() {
            return;
        }
        self.updating.set(true);

        let total_lines = self.buffer.line_count();
        self.last_line_count.set(total_lines);

        let cursor_pos = self.buffer.cursor_position();
        let cursor_iter = self.buffer.iter_at_offset(cursor_pos);
        let cur_line = cursor_iter.line();
        let cur_col = cursor_iter.line_offset() as usize;
        self.last_line.set(cur_line);
        self.last_cursor_pos.set(cursor_pos);

        for line in 0..total_lines {
            if line == cur_line {
                update_line(&self.buffer, line, Some((cur_col, cur_col)));
            } else {
                update_line(&self.buffer, line, None);
            }
        }

        self.updating.set(false);
    }

    pub fn clear_all(&self) {
        if self.updating.get() {
            return;
        }
        self.updating.set(true);
        let total_lines = self.buffer.line_count();
        for line in 0..total_lines {
            clear_line_tags(&self.buffer, line);
        }
        self.updating.set(false);
    }
}
