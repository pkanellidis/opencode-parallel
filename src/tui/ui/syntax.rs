//! Syntax highlighting support for code rendering in the TUI.

use once_cell::sync::Lazy;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

fn syntect_color_to_ratatui(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

fn syntect_style_to_ratatui(style: syntect::highlighting::Style) -> Style {
    let mut ratatui_style = Style::default().fg(syntect_color_to_ratatui(style.foreground));
    if style.font_style.contains(FontStyle::BOLD) {
        ratatui_style = ratatui_style.bold();
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        ratatui_style = ratatui_style.italic();
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        ratatui_style = ratatui_style.underlined();
    }
    ratatui_style
}

pub fn detect_language_from_path(file_path: &str) -> Option<&'static str> {
    let path = Path::new(file_path);
    let extension = path.extension()?.to_str()?;
    match extension {
        "rs" => Some("Rust"),
        "py" => Some("Python"),
        "js" => Some("JavaScript"),
        "ts" => Some("TypeScript"),
        "tsx" => Some("TypeScript"),
        "jsx" => Some("JavaScript"),
        "go" => Some("Go"),
        "rb" => Some("Ruby"),
        "java" => Some("Java"),
        "c" | "h" => Some("C"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("C++"),
        "cs" => Some("C#"),
        "php" => Some("PHP"),
        "swift" => Some("Swift"),
        "kt" | "kts" => Some("Kotlin"),
        "scala" => Some("Scala"),
        "sh" | "bash" | "zsh" => Some("Bourne Again Shell (bash)"),
        "json" => Some("JSON"),
        "yaml" | "yml" => Some("YAML"),
        "toml" => Some("TOML"),
        "xml" => Some("XML"),
        "html" | "htm" => Some("HTML"),
        "css" => Some("CSS"),
        "scss" | "sass" => Some("SCSS"),
        "sql" => Some("SQL"),
        "md" | "markdown" => Some("Markdown"),
        _ => None,
    }
}

pub fn highlight_code(code: &str, language: Option<&str>) -> Vec<Line<'static>> {
    let syntax = language
        .and_then(|lang| SYNTAX_SET.find_syntax_by_name(lang))
        .or_else(|| SYNTAX_SET.find_syntax_by_extension("txt"));

    let syntax = match syntax {
        Some(s) => s,
        None => return code.lines().map(|l| Line::from(l.to_string())).collect(),
    };

    let theme = &THEME_SET.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);

    code.lines()
        .map(|line| {
            let line_with_newline = format!("{}\n", line);
            let ranges = highlighter
                .highlight_line(&line_with_newline, &SYNTAX_SET)
                .unwrap_or_default();

            let spans: Vec<Span<'static>> = ranges
                .into_iter()
                .map(|(style, text)| {
                    Span::styled(
                        text.trim_end_matches('\n').to_string(),
                        syntect_style_to_ratatui(style),
                    )
                })
                .collect();

            Line::from(spans)
        })
        .collect()
}

pub fn highlight_code_with_line_numbers(
    code: &str,
    language: Option<&str>,
    start_line: usize,
) -> Vec<Line<'static>> {
    let highlighted = highlight_code(code, language);
    let line_num_width = (start_line + highlighted.len()).to_string().len();

    highlighted
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = start_line + i;
            let mut spans: Vec<Span<'static>> = vec![Span::styled(
                format!("{:>width$} │ ", line_num, width = line_num_width),
                Style::default().fg(Color::Rgb(90, 90, 100)),
            )];
            spans.extend(line.spans);
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_rust_extension() {
        assert_eq!(detect_language_from_path("src/main.rs"), Some("Rust"));
    }

    #[test]
    fn detect_python_extension() {
        assert_eq!(detect_language_from_path("script.py"), Some("Python"));
    }

    #[test]
    fn detect_typescript_extension() {
        assert_eq!(detect_language_from_path("app.tsx"), Some("TypeScript"));
    }

    #[test]
    fn detect_unknown_extension_returns_none() {
        assert_eq!(detect_language_from_path("file.unknown"), None);
    }

    #[test]
    fn highlight_code_returns_lines() {
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        let lines = highlight_code(code, Some("Rust"));
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn highlight_with_line_numbers_adds_numbers() {
        let code = "let x = 1;";
        let lines = highlight_code_with_line_numbers(code, Some("Rust"), 1);
        assert_eq!(lines.len(), 1);
        let first_span = &lines[0].spans[0];
        assert!(first_span.content.contains("1"));
    }
}
