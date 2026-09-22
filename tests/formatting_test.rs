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
** Unit tests for Markdown wrapping, lists, quotes, and heading cycles.
*/

use slate::editor::markdown::*;

#[test]
fn test_toggle_wrap_bold() {
    let (wrapped, start, end) = toggle_wrap("Bonjour", "**", "**");
    assert_eq!(wrapped, "**Bonjour**");
    assert_eq!(start, 0);
    assert_eq!(end, 11);

    let (unwrapped, u_start, u_end) = toggle_wrap(&wrapped, "**", "**");
    assert_eq!(unwrapped, "Bonjour");
    assert_eq!(u_start, 0);
    assert_eq!(u_end, 7);
}

#[test]
fn test_toggle_wrap_empty() {
    let (template, start, end) = toggle_wrap("", "**", "**");
    assert_eq!(template, "****");
    assert_eq!(start, 2);
    assert_eq!(end, 2);
}

#[test]
fn test_toggle_wrap_italic_strikethrough() {
    let (italic, _, _) = toggle_wrap("texte", "*", "*");
    assert_eq!(italic, "*texte*");

    let (strike, _, _) = toggle_wrap("barré", "~~", "~~");
    assert_eq!(strike, "~~barré~~");
}

#[test]
fn test_wrap_link() {
    let (link, _, _) = wrap_link("Google");
    assert_eq!(link, "[Google](url)");

    let (empty_link, _, _) = wrap_link("");
    assert_eq!(empty_link, "[texte](url)");

    let (url_link, _, _) = wrap_link("https://example.com");
    assert_eq!(url_link, "[lien](https://example.com)");
}

#[test]
fn test_cycle_heading() {
    let l0 = "Titre de section";
    let l1 = cycle_heading_prefix(l0);
    assert_eq!(l1, "# Titre de section");

    let l2 = cycle_heading_prefix(&l1);
    assert_eq!(l2, "## Titre de section");

    let l3 = cycle_heading_prefix(&l2);
    assert_eq!(l3, "### Titre de section");

    let l4 = cycle_heading_prefix(&l3);
    assert_eq!(l4, "#### Titre de section");

    let l5 = cycle_heading_prefix(&l4);
    assert_eq!(l5, "##### Titre de section");

    let l6 = cycle_heading_prefix(&l5);
    assert_eq!(l6, "###### Titre de section");

    let l_none = cycle_heading_prefix(&l6);
    assert_eq!(l_none, "Titre de section");
}

#[test]
fn test_toggle_bullet_list() {
    let plain = "Élément";
    let bullet = toggle_bullet_list(plain);
    assert_eq!(bullet, "- Élément");

    let reverted = toggle_bullet_list(&bullet);
    assert_eq!(reverted, "Élément");
}

#[test]
fn test_toggle_blockquote() {
    let plain = "Citation importante";
    let quote = toggle_blockquote(plain);
    assert_eq!(quote, "> Citation importante");

    let reverted = toggle_blockquote(&quote);
    assert_eq!(reverted, "Citation importante");
}

#[test]
fn test_toggle_numbered_list() {
    let items = vec!["Premier", "Deuxième", "Troisième"];
    let numbered = toggle_numbered_list(&items);
    assert_eq!(numbered, vec!["1. Premier", "2. Deuxième", "3. Troisième"]);

    let as_refs: Vec<&str> = numbered.iter().map(|s| s.as_str()).collect();
    let unnumbered = toggle_numbered_list(&as_refs);
    assert_eq!(unnumbered, vec!["Premier", "Deuxième", "Troisième"]);
}

#[test]
fn test_wrap_code() {
    let inline = "let x = 10;";
    let (code, _, _) = wrap_code(inline);
    assert_eq!(code, "`let x = 10;`");

    let multiline = "fn main() {\n    println!(\"hi\");\n}";
    let (block, _, _) = wrap_code(multiline);
    assert_eq!(block, "```\nfn main() {\n    println!(\"hi\");\n}\n```");
}

#[test]
fn test_indent_and_unindent() {
    let lines = vec!["Line 1", "Line 2"];
    let indented = indent_text(&lines);
    assert_eq!(indented, vec!["    Line 1", "    Line 2"]);

    let refs: Vec<&str> = indented.iter().map(|s| s.as_str()).collect();
    let unindented = unindent_text(&refs);
    assert_eq!(unindented, vec!["Line 1", "Line 2"]);
}
