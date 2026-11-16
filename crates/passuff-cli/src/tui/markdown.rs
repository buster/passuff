#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownStyle {
    Normal,
    Heading,
    Dim,
    Code,
    Rule,
}

#[derive(Debug, Clone)]
pub struct MarkdownLine {
    pub text: String,
    pub style: MarkdownStyle,
}

pub fn render(text: &str) -> Vec<MarkdownLine> {
    let mut lines = Vec::new();
    let mut in_code = false;
    let mut pending_blank = false;
    for raw in text.lines() {
        let trimmed = raw.trim_end();
        if trimmed.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            lines.push(MarkdownLine {
                text: format!("  {trimmed}"),
                style: MarkdownStyle::Code,
            });
            continue;
        }
        if trimmed.trim().is_empty() {
            if !lines.is_empty() {
                pending_blank = true;
            }
            continue;
        }
        if pending_blank {
            lines.push(MarkdownLine {
                text: String::new(),
                style: MarkdownStyle::Normal,
            });
            pending_blank = false;
        }
        let content = trimmed.trim_start();
        if let Some(heading) = content.strip_prefix('#') {
            let heading = heading.trim_start_matches('#').trim();
            lines.push(MarkdownLine {
                text: inline(heading),
                style: MarkdownStyle::Heading,
            });
            continue;
        }
        if content.chars().all(|character| character == '-')
            || content.chars().all(|character| character == '*')
        {
            lines.push(MarkdownLine {
                text: "─".repeat(40),
                style: MarkdownStyle::Rule,
            });
            continue;
        }
        let (prefix, body) = if let Some(rest) = content.strip_prefix("- [ ] ") {
            ("☐ ".to_string(), rest)
        } else if let Some(rest) = content.strip_prefix("- [x] ") {
            ("☑ ".to_string(), rest)
        } else if let Some(rest) = content.strip_prefix("- ") {
            ("• ".to_string(), rest)
        } else if let Some(rest) = content.strip_prefix("* ") {
            ("• ".to_string(), rest)
        } else if let Some((number, rest)) = numbered(content) {
            (number, rest)
        } else {
            (String::new(), content)
        };
        if prefix.is_empty() {
            lines.push(MarkdownLine {
                text: inline(content),
                style: MarkdownStyle::Normal,
            });
        } else {
            let body = inline(body);
            let indent = " ".repeat(prefix.chars().count());
            let mut first = true;
            for part in body.split('\n') {
                if first {
                    lines.push(MarkdownLine {
                        text: format!("{prefix}{part}"),
                        style: MarkdownStyle::Normal,
                    });
                    first = false;
                } else {
                    lines.push(MarkdownLine {
                        text: format!("{indent}{part}"),
                        style: MarkdownStyle::Normal,
                    });
                }
            }
        }
    }
    if pending_blank {
        // A trailing blank is not meaningful; consecutive blanks already
        // collapsed to one.
    }
    lines
}

fn numbered(content: &str) -> Option<(String, &str)> {
    let dot = content.find(". ")?;
    let number = &content[..dot];
    if number.is_empty() || !number.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    Some((format!("{number}. "), &content[dot + 2..]))
}

fn inline(text: &str) -> String {
    let mut output = text.replace("**", "").replace("__", "");
    output = output.replace('`', "");
    output = strip_links(&output);
    while let Some(start) = output.find('[') {
        let Some(close) = output[start..].find(']') else {
            break;
        };
        let close = start + close;
        if output[close + 1..].starts_with('(') {
            if let Some(end) = output[close + 1..].find(')') {
                let end = close + 1 + end;
                let label = output[start + 1..close].to_string();
                output.replace_range(start..=end, &label);
                continue;
            }
        }
        break;
    }
    output
}

fn strip_links(text: &str) -> String {
    text.to_string()
}
