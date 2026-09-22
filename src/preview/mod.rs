use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub struct PreviewView {
    scrolled: gtk4::ScrolledWindow,
    content_box: gtk4::Box,
}

impl PreviewView {
    pub fn new() -> Self {
        let content_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(36)
            .hexpand(true)
            .build();

        let clamp = libadwaita::Clamp::builder()
            .maximum_size(780)
            .tightening_threshold(560)
            .child(&content_box)
            .build();

        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&clamp)
            .hexpand(true)
            .vexpand(true)
            .build();

        Self {
            scrolled,
            content_box,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.scrolled
    }

    pub fn render_markdown(&self, markdown_text: &str) {
        // Clear existing widgets
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        if markdown_text.trim().is_empty() {
            let empty_label = gtk4::Label::builder()
                .label("Document vide")
                .css_classes(["dim-label"])
                .margin_top(48)
                .build();
            self.content_box.append(&empty_label);
            return;
        }

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(markdown_text, options);
        let mut renderer = AstRenderer::new(&self.content_box);
        renderer.render(parser);
    }
}

impl Default for PreviewView {
    fn default() -> Self {
        Self::new()
    }
}

struct AstRenderer<'a> {
    parent: &'a gtk4::Box,
    inline_markup: String,
    in_heading: Option<HeadingLevel>,
    in_code_block: Option<String>,
    code_block_text: String,
    in_blockquote: bool,
    blockquote_box: Option<gtk4::Box>,
    in_list: Vec<ListContext>,
    in_table: bool,
    table_grid: Option<gtk4::Grid>,
    table_row: i32,
    table_col: i32,
    table_is_head: bool,
    table_alignments: Vec<Alignment>,
}

#[derive(Debug, Clone)]
enum ListContext {
    Bullet,
    Numbered(usize),
}

impl<'a> AstRenderer<'a> {
    fn new(parent: &'a gtk4::Box) -> Self {
        Self {
            parent,
            inline_markup: String::new(),
            in_heading: None,
            in_code_block: None,
            code_block_text: String::new(),
            in_blockquote: false,
            blockquote_box: None,
            in_list: Vec::new(),
            in_table: false,
            table_grid: None,
            table_row: 0,
            table_col: 0,
            table_is_head: false,
            table_alignments: Vec::new(),
        }
    }

    fn render(&mut self, parser: Parser) {
        for event in parser {
            match event {
                Event::Start(tag) => self.handle_start_tag(tag),
                Event::End(tag_end) => self.handle_end_tag(tag_end),
                Event::Text(text) => self.handle_text(&text),
                Event::Code(code) => self.handle_inline_code(&code),
                Event::Html(html) => self.handle_text(&html),
                Event::SoftBreak => self.inline_markup.push(' '),
                Event::HardBreak => self.inline_markup.push('\n'),
                Event::Rule => self.append_rule(),
                Event::TaskListMarker(checked) => self.handle_task_list_marker(checked),
                _ => {}
            }
        }
    }

    fn handle_start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.inline_markup.clear();
            }
            Tag::Heading { level, .. } => {
                self.in_heading = Some(level);
                self.inline_markup.clear();
            }
            Tag::BlockQuote(_) => {
                self.in_blockquote = true;
                let bq_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(6)
                    .margin_start(16)
                    .margin_top(4)
                    .margin_bottom(4)
                    .build();
                bq_box.add_css_class("blockquote");
                self.blockquote_box = Some(bq_box);
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                self.in_code_block = Some(lang);
                self.code_block_text.clear();
            }
            Tag::List(first_num) => {
                let ctx = match first_num {
                    Some(n) => ListContext::Numbered(n as usize),
                    None => ListContext::Bullet,
                };
                self.in_list.push(ctx);
            }
            Tag::Item => {
                self.inline_markup.clear();
            }
            Tag::Emphasis => {
                self.inline_markup.push_str("<i>");
            }
            Tag::Strong => {
                self.inline_markup.push_str("<b>");
            }
            Tag::Strikethrough => {
                self.inline_markup.push_str("<s>");
            }
            Tag::Link { dest_url, .. } => {
                let escaped_url = glib::markup_escape_text(&dest_url);
                self.inline_markup.push_str(&format!("<a href=\"{escaped_url}\">"));
            }
            Tag::Table(alignments) => {
                self.in_table = true;
                self.table_alignments = alignments;
                self.table_row = 0;
                self.table_col = 0;

                let grid = gtk4::Grid::builder()
                    .column_spacing(12)
                    .row_spacing(6)
                    .margin_top(8)
                    .margin_bottom(8)
                    .build();
                grid.add_css_class("table-view");
                self.table_grid = Some(grid);
            }
            Tag::TableHead => {
                self.table_is_head = true;
                self.table_col = 0;
            }
            Tag::TableRow => {
                self.table_is_head = false;
                self.table_col = 0;
            }
            Tag::TableCell => {
                self.inline_markup.clear();
            }
            _ => {}
        }
    }

    fn handle_end_tag(&mut self, tag_end: TagEnd) {
        match tag_end {
            TagEnd::Paragraph => {
                if !self.inline_markup.trim().is_empty() {
                    let label = self.create_markup_label(&self.inline_markup);
                    if let Some(ref bq) = self.blockquote_box {
                        bq.append(&label);
                    } else {
                        self.parent.append(&label);
                    }
                }
                self.inline_markup.clear();
            }
            TagEnd::Heading(_) => {
                if let Some(level) = self.in_heading.take() {
                    let heading_label = self.create_heading_label(&self.inline_markup, level);
                    self.parent.append(&heading_label);
                }
                self.inline_markup.clear();
            }
            TagEnd::BlockQuote(_) => {
                self.in_blockquote = false;
                if let Some(bq) = self.blockquote_box.take() {
                    self.parent.append(&bq);
                }
            }
            TagEnd::CodeBlock => {
                let lang = self.in_code_block.take().unwrap_or_default();
                let widget = self.create_code_block(&self.code_block_text, &lang);
                if let Some(ref bq) = self.blockquote_box {
                    bq.append(&widget);
                } else {
                    self.parent.append(&widget);
                }
                self.code_block_text.clear();
            }
            TagEnd::List(_) => {
                self.in_list.pop();
            }
            TagEnd::Item => {
                if !self.inline_markup.trim().is_empty() {
                    let markup = self.inline_markup.clone();
                    let item_widget = self.create_list_item(&markup);
                    if let Some(ref bq) = self.blockquote_box {
                        bq.append(&item_widget);
                    } else {
                        self.parent.append(&item_widget);
                    }
                }
                self.inline_markup.clear();
            }
            TagEnd::Emphasis => {
                self.inline_markup.push_str("</i>");
            }
            TagEnd::Strong => {
                self.inline_markup.push_str("</b>");
            }
            TagEnd::Strikethrough => {
                self.inline_markup.push_str("</s>");
            }
            TagEnd::Link => {
                self.inline_markup.push_str("</a>");
            }
            TagEnd::TableCell => {
                if let Some(ref grid) = self.table_grid {
                    let cell_markup = if self.table_is_head {
                        format!("<b>{}</b>", self.inline_markup.trim())
                    } else {
                        self.inline_markup.trim().to_string()
                    };
                    let label = self.create_markup_label(&cell_markup);
                    label.set_xalign(0.0);
                    label.add_css_class("table-cell");
                    grid.attach(&label, self.table_col, self.table_row, 1, 1);
                    self.table_col += 1;
                }
                self.inline_markup.clear();
            }
            TagEnd::TableRow | TagEnd::TableHead => {
                self.table_row += 1;
            }
            TagEnd::Table => {
                self.in_table = false;
                if let Some(grid) = self.table_grid.take() {
                    self.parent.append(&grid);
                }
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, text: &str) {
        if self.in_code_block.is_some() {
            self.code_block_text.push_str(text);
        } else {
            let escaped = glib::markup_escape_text(text);
            self.inline_markup.push_str(&escaped);
        }
    }

    fn handle_inline_code(&mut self, code: &str) {
        let escaped = glib::markup_escape_text(code);
        self.inline_markup.push_str(&format!("<tt>{escaped}</tt>"));
    }

    fn handle_task_list_marker(&mut self, checked: bool) {
        let check = if checked { "☑ " } else { "☐ " };
        self.inline_markup.push_str(check);
    }

    fn append_rule(&mut self) {
        let sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        self.parent.append(&sep);
    }

    fn create_markup_label(&self, markup: &str) -> gtk4::Label {
        let label = gtk4::Label::builder()
            .use_markup(true)
            .label(markup)
            .wrap(true)
            .wrap_mode(gtk4::pango::WrapMode::WordChar)
            .selectable(true)
            .xalign(0.0)
            .build();

        label.connect_activate_link(|_, uri| {
            let _ = gtk4::gio::AppInfo::launch_default_for_uri(uri, None::<&gtk4::gio::AppLaunchContext>);
            glib::Propagation::Stop
        });

        label
    }

    fn create_heading_label(&self, markup: &str, level: HeadingLevel) -> gtk4::Label {
        let label = self.create_markup_label(markup);
        let (class, top_m, bot_m) = match level {
            HeadingLevel::H1 => ("title-1", 20, 10),
            HeadingLevel::H2 => ("title-2", 16, 8),
            HeadingLevel::H3 => ("title-3", 12, 6),
            HeadingLevel::H4 => ("title-4", 10, 4),
            HeadingLevel::H5 => ("heading", 8, 4),
            HeadingLevel::H6 => ("caption-heading", 6, 2),
        };
        label.add_css_class(class);
        label.set_margin_top(top_m);
        label.set_margin_bottom(bot_m);
        label
    }

    fn create_list_item(&mut self, markup: &str) -> gtk4::Box {
        let row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .margin_start((self.in_list.len() * 16) as i32)
            .margin_top(2)
            .margin_bottom(2)
            .build();

        let badge = match self.in_list.last_mut() {
            Some(ListContext::Numbered(n)) => {
                let text = format!("{n}.");
                *n += 1;
                gtk4::Label::builder()
                    .label(&text)
                    .css_classes(["dim-label"])
                    .xalign(1.0)
                    .width_chars(3)
                    .build()
            }
            _ => gtk4::Label::builder()
                .label("•")
                .css_classes(["accent"])
                .xalign(0.5)
                .width_chars(2)
                .build(),
        };

        let label = self.create_markup_label(markup);
        row.append(&badge);
        row.append(&label);
        row
    }

    fn create_code_block(&self, code: &str, lang: &str) -> gtk4::Box {
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(0)
            .margin_top(8)
            .margin_bottom(8)
            .build();
        card.add_css_class("card");
        card.add_css_class("code-block");

        // Header with language and copy button
        let header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .margin_start(12)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        let lang_label = gtk4::Label::builder()
            .label(if lang.is_empty() { "code" } else { lang })
            .css_classes(["dim-label", "caption"])
            .hexpand(true)
            .xalign(0.0)
            .build();

        let copy_btn = gtk4::Button::builder()
            .label("Copier")
            .tooltip_text("Copier dans le presse-papier")
            .css_classes(["flat", "caption"])
            .build();

        let code_copy = code.to_string();
        copy_btn.connect_clicked(move |btn| {
            if let Some(display) = gdk::Display::default() {
                display.clipboard().set_text(&code_copy);
                btn.set_label("Copié !");
                let btn_weak = btn.downgrade();
                glib::timeout_add_seconds_local_once(2, move || {
                    if let Some(b) = btn_weak.upgrade() {
                        b.set_label("Copier");
                    }
                });
            }
        });

        header.append(&lang_label);
        header.append(&copy_btn);

        let sep = gtk4::Separator::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();

        let code_label = gtk4::Label::builder()
            .label(code)
            .selectable(true)
            .wrap(false)
            .xalign(0.0)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .margin_bottom(12)
            .build();
        code_label.add_css_class("monospace");

        let code_scroller = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .child(&code_label)
            .build();

        card.append(&header);
        card.append(&sep);
        card.append(&code_scroller);
        card
    }
}
