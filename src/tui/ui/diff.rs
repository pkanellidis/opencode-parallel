//! Unified diff generation for file edits using the similar crate.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use similar::{ChangeTag, TextDiff};

use super::theme::{ERROR, SUCCESS, TEXT_DIM, TEXT_SECONDARY};

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub content: String,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Equal,
    Delete,
    Insert,
}

pub fn generate_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut result = Vec::new();
    let mut old_line = 1usize;
    let mut new_line = 1usize;

    for change in diff.iter_all_changes() {
        let line = DiffLine {
            tag: match change.tag() {
                ChangeTag::Delete => DiffTag::Delete,
                ChangeTag::Insert => DiffTag::Insert,
                ChangeTag::Equal => DiffTag::Equal,
            },
            content: change.value().trim_end_matches('\n').to_string(),
            old_line_num: match change.tag() {
                ChangeTag::Delete | ChangeTag::Equal => {
                    let num = old_line;
                    old_line += 1;
                    Some(num)
                }
                ChangeTag::Insert => None,
            },
            new_line_num: match change.tag() {
                ChangeTag::Insert | ChangeTag::Equal => {
                    let num = new_line;
                    new_line += 1;
                    Some(num)
                }
                ChangeTag::Delete => None,
            },
        };
        result.push(line);
    }

    result
}

pub fn generate_unified_diff(
    old_text: &str,
    new_text: &str,
    file_path: &str,
    context_lines: usize,
) -> Vec<DiffLine> {
    let all_lines = generate_diff(old_text, new_text);

    if all_lines.iter().all(|l| l.tag == DiffTag::Equal) {
        return Vec::new();
    }

    let change_indices: Vec<usize> = all_lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.tag != DiffTag::Equal)
        .map(|(i, _)| i)
        .collect();

    if change_indices.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();

    result.push(DiffLine {
        tag: DiffTag::Equal,
        content: format!("--- a/{}", file_path),
        old_line_num: None,
        new_line_num: None,
    });
    result.push(DiffLine {
        tag: DiffTag::Equal,
        content: format!("+++ b/{}", file_path),
        old_line_num: None,
        new_line_num: None,
    });

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut current_start = change_indices[0].saturating_sub(context_lines);
    let mut current_end = (change_indices[0] + context_lines + 1).min(all_lines.len());

    for &idx in &change_indices[1..] {
        let new_start = idx.saturating_sub(context_lines);
        let new_end = (idx + context_lines + 1).min(all_lines.len());

        if new_start <= current_end {
            current_end = new_end;
        } else {
            ranges.push((current_start, current_end));
            current_start = new_start;
            current_end = new_end;
        }
    }
    ranges.push((current_start, current_end));

    for (start, end) in ranges {
        let old_start = all_lines[start..end]
            .iter()
            .filter_map(|l| l.old_line_num)
            .min()
            .unwrap_or(1);
        let new_start = all_lines[start..end]
            .iter()
            .filter_map(|l| l.new_line_num)
            .min()
            .unwrap_or(1);
        let old_count = all_lines[start..end]
            .iter()
            .filter(|l| l.old_line_num.is_some())
            .count();
        let new_count = all_lines[start..end]
            .iter()
            .filter(|l| l.new_line_num.is_some())
            .count();

        result.push(DiffLine {
            tag: DiffTag::Equal,
            content: format!(
                "@@ -{},{} +{},{} @@",
                old_start, old_count, new_start, new_count
            ),
            old_line_num: None,
            new_line_num: None,
        });

        for line in &all_lines[start..end] {
            result.push(line.clone());
        }
    }

    result
}

pub fn diff_to_lines(diff_lines: &[DiffLine]) -> Vec<Line<'static>> {
    diff_lines
        .iter()
        .map(|line| {
            let (prefix, style, bg) = match line.tag {
                DiffTag::Delete => (
                    "-",
                    Style::default().fg(ERROR),
                    Some(Color::Rgb(60, 20, 20)),
                ),
                DiffTag::Insert => (
                    "+",
                    Style::default().fg(SUCCESS),
                    Some(Color::Rgb(20, 50, 30)),
                ),
                DiffTag::Equal => {
                    if line.content.starts_with("@@") {
                        ("", Style::default().fg(Color::Rgb(130, 170, 255)), None)
                    } else if line.content.starts_with("---") || line.content.starts_with("+++") {
                        (
                            "",
                            Style::default()
                                .fg(TEXT_SECONDARY)
                                .add_modifier(Modifier::BOLD),
                            None,
                        )
                    } else {
                        (" ", Style::default().fg(TEXT_DIM), None)
                    }
                }
            };

            let line_num_str = match (line.old_line_num, line.new_line_num) {
                (Some(o), Some(n)) => format!("{:>4} {:>4} ", o, n),
                (Some(o), None) => format!("{:>4}      ", o),
                (None, Some(n)) => format!("     {:>4} ", n),
                (None, None) => "          ".to_string(),
            };

            let content = if line.content.starts_with("@@")
                || line.content.starts_with("---")
                || line.content.starts_with("+++")
            {
                line.content.clone()
            } else {
                format!("{}{}", prefix, line.content)
            };

            let mut final_style = style;
            if let Some(bg_color) = bg {
                final_style = final_style.bg(bg_color);
            }

            Line::from(vec![
                Span::styled(line_num_str, Style::default().fg(TEXT_DIM)),
                Span::styled(content, final_style),
            ])
        })
        .collect()
}

pub fn format_simple_diff(old_text: &str, new_text: &str) -> Vec<Line<'static>> {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut lines = Vec::new();

    for change in diff.iter_all_changes() {
        let (prefix, style) = match change.tag() {
            ChangeTag::Delete => ("-", Style::default().fg(ERROR)),
            ChangeTag::Insert => ("+", Style::default().fg(SUCCESS)),
            ChangeTag::Equal => (" ", Style::default().fg(TEXT_DIM)),
        };

        lines.push(Line::from(vec![Span::styled(
            format!("{}{}", prefix, change.value().trim_end_matches('\n')),
            style,
        )]));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_diff_detects_additions() {
        let old = "line1\nline2\n";
        let new = "line1\nline2\nline3\n";
        let diff = generate_diff(old, new);

        assert!(diff
            .iter()
            .any(|l| l.tag == DiffTag::Insert && l.content == "line3"));
    }

    #[test]
    fn generate_diff_detects_deletions() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline3\n";
        let diff = generate_diff(old, new);

        assert!(diff
            .iter()
            .any(|l| l.tag == DiffTag::Delete && l.content == "line2"));
    }

    #[test]
    fn generate_diff_detects_modifications() {
        let old = "hello world\n";
        let new = "hello rust\n";
        let diff = generate_diff(old, new);

        assert!(diff.iter().any(|l| l.tag == DiffTag::Delete));
        assert!(diff.iter().any(|l| l.tag == DiffTag::Insert));
    }

    #[test]
    fn unified_diff_includes_context() {
        let old = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";
        let new = "1\n2\n3\nCHANGED\n5\n6\n7\n8\n9\n10\n";
        let diff = generate_unified_diff(old, new, "test.txt", 2);

        assert!(diff.iter().any(|l| l.content.starts_with("@@")));
        assert!(diff.iter().any(|l| l.content == "--- a/test.txt"));
        assert!(diff.iter().any(|l| l.content == "+++ b/test.txt"));
    }

    #[test]
    fn no_changes_returns_empty() {
        let text = "same content\n";
        let diff = generate_unified_diff(text, text, "test.txt", 3);
        assert!(diff.is_empty());
    }

    #[test]
    fn diff_to_lines_produces_styled_output() {
        let old = "old\n";
        let new = "new\n";
        let diff = generate_diff(old, new);
        let lines = diff_to_lines(&diff);

        assert!(!lines.is_empty());
    }
}
