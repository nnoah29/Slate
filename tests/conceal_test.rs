use slate::editor::conceal::{parse_markdown_spans, SpanType};

#[test]
fn test_heading_h1() {
    let line = "# Mon grand titre";
    let spans = parse_markdown_spans(line);

    // Span 0: "# " concealed
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 2);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    // Span 1: "Mon grand titre" H1
    assert_eq!(spans[1].start, 2);
    assert_eq!(spans[1].end, 17);
    assert_eq!(spans[1].span_type, SpanType::H1);
}

#[test]
fn test_heading_h2_with_french_accent() {
    let line = "## Réunion d'été";
    let spans = parse_markdown_spans(line);

    // Span 0: "## " concealed
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 3);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    // Span 1: "Réunion d'été" H2 (Unicode chars count = 13, so end = 3 + 13 = 16)
    assert_eq!(spans[1].start, 3);
    assert_eq!(spans[1].end, 16);
    assert_eq!(spans[1].span_type, SpanType::H2);
}

#[test]
fn test_heading_h3_to_h6() {
    let h3 = parse_markdown_spans("### Titre 3");
    assert_eq!(h3[0].span_type, SpanType::Conceal);
    assert_eq!(h3[0].end, 4);
    assert_eq!(h3[1].span_type, SpanType::H3);

    let h6 = parse_markdown_spans("###### Titre 6");
    assert_eq!(h6[0].span_type, SpanType::Conceal);
    assert_eq!(h6[0].end, 7);
    assert_eq!(h6[1].span_type, SpanType::H6);
}

#[test]
fn test_bold_spans() {
    let line = "Voici du **texte en gras** ici.";
    let spans = parse_markdown_spans(line);

    // Delim 1: "**" at 9..11
    assert_eq!(spans[0].start, 9);
    assert_eq!(spans[0].end, 11);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    // Content: "texte en gras" at 11..24
    assert_eq!(spans[1].start, 11);
    assert_eq!(spans[1].end, 24);
    assert_eq!(spans[1].span_type, SpanType::Bold);

    // Delim 2: "**" at 24..26
    assert_eq!(spans[2].start, 24);
    assert_eq!(spans[2].end, 26);
    assert_eq!(spans[2].span_type, SpanType::Conceal);
}

#[test]
fn test_italic_spans() {
    let line = "Texte en *italique* simple.";
    let spans = parse_markdown_spans(line);

    assert_eq!(spans[0].start, 9);
    assert_eq!(spans[0].end, 10);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    assert_eq!(spans[1].start, 10);
    assert_eq!(spans[1].end, 18);
    assert_eq!(spans[1].span_type, SpanType::Italic);

    assert_eq!(spans[2].start, 18);
    assert_eq!(spans[2].end, 19);
    assert_eq!(spans[2].span_type, SpanType::Conceal);
}

#[test]
fn test_strikethrough_spans() {
    let line = "Mot ~~barré~~ terminé.";
    let spans = parse_markdown_spans(line);

    assert_eq!(spans[0].start, 4);
    assert_eq!(spans[0].end, 6);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    assert_eq!(spans[1].start, 6);
    assert_eq!(spans[1].end, 11);
    assert_eq!(spans[1].span_type, SpanType::Strike);

    assert_eq!(spans[2].start, 11);
    assert_eq!(spans[2].end, 13);
    assert_eq!(spans[2].span_type, SpanType::Conceal);
}

#[test]
fn test_inline_code_spans() {
    let line = "Exécute `cargo build --release` svp.";
    let spans = parse_markdown_spans(line);

    assert_eq!(spans[0].start, 8);
    assert_eq!(spans[0].end, 9);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    assert_eq!(spans[1].start, 9);
    assert_eq!(spans[1].end, 30);
    assert_eq!(spans[1].span_type, SpanType::Code);

    assert_eq!(spans[2].start, 30);
    assert_eq!(spans[2].end, 31);
    assert_eq!(spans[2].span_type, SpanType::Conceal);
}

#[test]
fn test_link_spans() {
    let line = "Documentation sur [Slate](https://github.com/slate).";
    let spans = parse_markdown_spans(line);

    // Delim 1: "[" at 18..19
    assert_eq!(spans[0].start, 18);
    assert_eq!(spans[0].end, 19);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    // Link label: "Slate" at 19..24
    assert_eq!(spans[1].start, 19);
    assert_eq!(spans[1].end, 24);
    assert_eq!(spans[1].span_type, SpanType::Link);

    // Delim 2: "](https://github.com/slate)" at 24..51
    assert_eq!(spans[2].start, 24);
    assert_eq!(spans[2].end, 51);
    assert_eq!(spans[2].span_type, SpanType::Conceal);
}

#[test]
fn test_blockquote_spans() {
    let line = "> Citation inspirante.";
    let spans = parse_markdown_spans(line);

    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].end, 2);
    assert_eq!(spans[0].span_type, SpanType::Conceal);

    assert_eq!(spans[1].start, 2);
    assert_eq!(spans[1].end, 22);
    assert_eq!(spans[1].span_type, SpanType::Quote);
}

#[test]
fn test_multiple_spans_on_single_line() {
    let line = "Du **gras** et du `code` et du *italique*.";
    let spans = parse_markdown_spans(line);

    // Should contain:
    // ** (conceal), gras (bold), ** (conceal)
    // ` (conceal), code (code), ` (conceal)
    // * (conceal), italique (italic), * (conceal)
    assert_eq!(spans.len(), 9);
    assert_eq!(spans[1].span_type, SpanType::Bold);
    assert_eq!(spans[4].span_type, SpanType::Code);
    assert_eq!(spans[7].span_type, SpanType::Italic);
}

#[test]
fn test_unclosed_delimiters_ignored() {
    let line = "Ceci a une seule * étoile et deux ** étoiles.";
    let spans = parse_markdown_spans(line);
    // Unclosed markers should not produce valid conceal spans
    assert!(spans.is_empty());
}

#[test]
fn test_user_screenshot_elements() {
    use slate::editor::conceal::parse_markdown_elements;

    let line = "**gras** *italique* _italique_";
    let elements = parse_markdown_elements(line);

    assert_eq!(elements.len(), 3);

    // 1. **gras** at 0..8
    assert_eq!(elements[0].start, 0);
    assert_eq!(elements[0].end, 8);
    assert_eq!(elements[0].delims, vec![(0, 2), (6, 8)]);
    assert_eq!(elements[0].styles[0].2, SpanType::Bold);

    // 2. *italique* at 9..19
    assert_eq!(elements[1].start, 9);
    assert_eq!(elements[1].end, 19);
    assert_eq!(elements[1].delims, vec![(9, 10), (18, 19)]);
    assert_eq!(elements[1].styles[0].2, SpanType::Italic);

    // 3. _italique_ at 20..30
    assert_eq!(elements[2].start, 20);
    assert_eq!(elements[2].end, 30);
    assert_eq!(elements[2].delims, vec![(20, 21), (29, 30)]);
    assert_eq!(elements[2].styles[0].2, SpanType::Italic);
}
