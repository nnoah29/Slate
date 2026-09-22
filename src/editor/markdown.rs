pub fn toggle_wrap(selected: &str, prefix: &str, suffix: &str) -> (String, usize, usize) {
    if selected.is_empty() {
        let inserted = format!("{prefix}{suffix}");
        let cursor_offset = prefix.chars().count();
        return (inserted, cursor_offset, cursor_offset);
    }

    if selected.starts_with(prefix) && selected.ends_with(suffix) && selected.len() >= prefix.len() + suffix.len() {
        // Unwrap
        let unwrapped = &selected[prefix.len()..selected.len() - suffix.len()];
        let len = unwrapped.chars().count();
        (unwrapped.to_string(), 0, len)
    } else {
        // Wrap
        let wrapped = format!("{prefix}{selected}{suffix}");
        let len = wrapped.chars().count();
        (wrapped, 0, len)
    }
}

pub fn wrap_link(selected: &str) -> (String, usize, usize) {
    if selected.is_empty() {
        let template = "[texte](url)";
        (template.to_string(), 1, 6)
    } else if selected.starts_with("http://") || selected.starts_with("https://") {
        let template = format!("[lien]({selected})");
        (template, 1, 5)
    } else {
        let template = format!("[{selected}](url)");
        let start = 1 + selected.chars().count() + 2;
        let end = start + 3;
        (template, start, end)
    }
}

pub fn cycle_heading_prefix(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    if let Some(rest) = trimmed.strip_prefix("###### ") {
        format!("{indent}{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("##### ") {
        format!("{indent}###### {rest}")
    } else if let Some(rest) = trimmed.strip_prefix("#### ") {
        format!("{indent}##### {rest}")
    } else if let Some(rest) = trimmed.strip_prefix("### ") {
        format!("{indent}#### {rest}")
    } else if let Some(rest) = trimmed.strip_prefix("## ") {
        format!("{indent}### {rest}")
    } else if let Some(rest) = trimmed.strip_prefix("# ") {
        format!("{indent}## {rest}")
    } else {
        format!("{indent}# {trimmed}")
    }
}

pub fn toggle_bullet_list(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    if let Some(rest) = trimmed.strip_prefix("- ") {
        format!("{indent}{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("* ") {
        format!("{indent}{rest}")
    } else {
        format!("{indent}- {trimmed}")
    }
}

pub fn toggle_blockquote(line: &str) -> String {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = &line[..indent_len];

    if let Some(rest) = trimmed.strip_prefix("> ") {
        format!("{indent}{rest}")
    } else {
        format!("{indent}> {trimmed}")
    }
}

pub fn toggle_numbered_list(lines: &[&str]) -> Vec<String> {
    let all_numbered = lines.iter().all(|l| {
        let trimmed = l.trim_start();
        trimmed.chars().next().map_or(false, |c| c.is_ascii_digit())
            && trimmed.find(". ").is_some()
    });

    if all_numbered {
        lines
            .iter()
            .map(|l| {
                let trimmed = l.trim_start();
                let indent_len = l.len() - trimmed.len();
                let indent = &l[..indent_len];
                if let Some(pos) = trimmed.find(". ") {
                    format!("{indent}{}", &trimmed[pos + 2..])
                } else {
                    l.to_string()
                }
            })
            .collect()
    } else {
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let trimmed = l.trim_start();
                let indent_len = l.len() - trimmed.len();
                let indent = &l[..indent_len];
                let num = i + 1;
                format!("{indent}{num}. {trimmed}")
            })
            .collect()
    }
}

pub fn wrap_code(selected: &str) -> (String, usize, usize) {
    if selected.is_empty() {
        let snippet = "```\n\n```";
        (snippet.to_string(), 4, 4)
    } else if selected.contains('\n') {
        let snippet = format!("```\n{selected}\n```");
        let len = snippet.chars().count();
        (snippet, 0, len)
    } else {
        toggle_wrap(selected, "`", "`")
    }
}

pub fn indent_text(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|l| format!("    {l}")).collect()
}

pub fn unindent_text(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .map(|l| {
            if let Some(rest) = l.strip_prefix("    ") {
                rest.to_string()
            } else if let Some(rest) = l.strip_prefix('\t') {
                rest.to_string()
            } else {
                l.trim_start_matches(' ').to_string()
            }
        })
        .collect()
}
