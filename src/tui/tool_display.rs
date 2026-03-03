//! Tool call visualization types and formatting for the TUI.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::ui::diff::{diff_to_lines, generate_unified_diff};
use super::ui::syntax::{detect_language_from_path, highlight_code_with_line_numbers};
use super::ui::theme::{
    ACCENT, ACCENT_SECONDARY, ERROR, SUCCESS, TEXT_DIM, TEXT_PRIMARY, TEXT_SECONDARY,
};

#[derive(Debug, Clone)]
pub enum ToolCallStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub tool_name: String,
    pub status: ToolCallStatus,
    pub parameters: serde_json::Value,
    pub result: Option<ToolCallResult>,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub enum ToolCallResult {
    Read {
        file_path: String,
        content: String,
        line_count: usize,
    },
    Edit {
        file_path: String,
        old_string: String,
        new_string: String,
    },
    Write {
        file_path: String,
        content: String,
    },
    Bash {
        command: String,
        output: String,
        exit_code: Option<i32>,
    },
    Glob {
        pattern: String,
        matches: Vec<String>,
    },
    Grep {
        pattern: String,
        matches: Vec<GrepMatch>,
    },
    Task {
        description: String,
        result: String,
    },
    Generic {
        summary: String,
    },
}

#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub file: String,
    pub line: usize,
    pub content: String,
}

impl ToolCallInfo {
    pub fn new(tool_name: String, parameters: serde_json::Value) -> Self {
        let display_name = format_tool_display_name(&tool_name, &parameters);
        Self {
            tool_name,
            status: ToolCallStatus::Pending,
            parameters,
            result: None,
            display_name,
        }
    }

    pub fn set_running(&mut self) {
        self.status = ToolCallStatus::Running;
    }

    pub fn set_completed(&mut self, result: ToolCallResult) {
        self.status = ToolCallStatus::Completed;
        self.result = Some(result);
    }

    pub fn set_failed(&mut self, error: String) {
        self.status = ToolCallStatus::Failed(error);
    }

    pub fn render_header(&self) -> Line<'static> {
        let (icon, icon_color) = match &self.status {
            ToolCallStatus::Pending => ("◌", TEXT_DIM),
            ToolCallStatus::Running => ("⚙", Color::Rgb(100, 180, 255)),
            ToolCallStatus::Completed => ("✓", SUCCESS),
            ToolCallStatus::Failed(_) => ("✗", ERROR),
        };

        let tool_style = Style::default().fg(ACCENT).add_modifier(Modifier::BOLD);
        let display = self.display_name.clone();

        Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
            Span::styled(display, tool_style),
        ])
    }

    pub fn render_details(&self, max_lines: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        match &self.result {
            Some(ToolCallResult::Edit {
                file_path,
                old_string,
                new_string,
            }) => {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("File: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(file_path.clone(), Style::default().fg(ACCENT_SECONDARY)),
                ]));

                let diff_lines = generate_unified_diff(old_string, new_string, file_path, 3);
                let styled_diff = diff_to_lines(&diff_lines);

                for (i, line) in styled_diff.into_iter().enumerate() {
                    if i >= max_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  ... {} more lines", diff_lines.len() - i),
                            Style::default().fg(TEXT_DIM),
                        )));
                        break;
                    }
                    let mut indented = vec![Span::raw("  ")];
                    indented.extend(line.spans);
                    lines.push(Line::from(indented));
                }
            }
            Some(ToolCallResult::Write { file_path, content }) => {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("File: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(file_path.clone(), Style::default().fg(ACCENT_SECONDARY)),
                ]));

                let lang = detect_language_from_path(file_path);
                let highlighted = highlight_code_with_line_numbers(content, lang, 1);
                let line_count = content.lines().count();

                lines.push(Line::from(Span::styled(
                    format!("  ({} lines written)", line_count),
                    Style::default().fg(TEXT_DIM),
                )));

                for (i, line) in highlighted.into_iter().enumerate() {
                    if i >= max_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  ... {} more lines", line_count - i),
                            Style::default().fg(TEXT_DIM),
                        )));
                        break;
                    }
                    let mut indented = vec![Span::raw("  ")];
                    indented.extend(line.spans);
                    lines.push(Line::from(indented));
                }
            }
            Some(ToolCallResult::Read {
                file_path,
                content,
                line_count,
            }) => {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("File: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(file_path.clone(), Style::default().fg(ACCENT_SECONDARY)),
                    Span::styled(
                        format!(" ({} lines)", line_count),
                        Style::default().fg(TEXT_DIM),
                    ),
                ]));

                let preview_lines: Vec<&str> = content.lines().take(5).collect();
                if !preview_lines.is_empty() {
                    let lang = detect_language_from_path(file_path);
                    let preview_content = preview_lines.join("\n");
                    let highlighted = highlight_code_with_line_numbers(&preview_content, lang, 1);

                    for line in highlighted.into_iter().take(max_lines) {
                        let mut indented = vec![Span::raw("  ")];
                        indented.extend(line.spans);
                        lines.push(Line::from(indented));
                    }

                    if *line_count > 5 {
                        lines.push(Line::from(Span::styled(
                            format!("  ... {} more lines", line_count - 5),
                            Style::default().fg(TEXT_DIM),
                        )));
                    }
                }
            }
            Some(ToolCallResult::Bash {
                command,
                output,
                exit_code,
            }) => {
                lines.push(Line::from(vec![
                    Span::styled("  $ ", Style::default().fg(SUCCESS)),
                    Span::styled(
                        truncate_string(command, 80),
                        Style::default().fg(TEXT_PRIMARY),
                    ),
                ]));

                if let Some(code) = exit_code {
                    let code_style = if *code == 0 {
                        Style::default().fg(SUCCESS)
                    } else {
                        Style::default().fg(ERROR)
                    };
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(format!("exit: {}", code), code_style),
                    ]));
                }

                if !output.is_empty() {
                    for (i, line) in output.lines().enumerate() {
                        if i >= max_lines {
                            let remaining = output.lines().count() - i;
                            lines.push(Line::from(Span::styled(
                                format!("  ... {} more lines", remaining),
                                Style::default().fg(TEXT_DIM),
                            )));
                            break;
                        }
                        lines.push(Line::from(Span::styled(
                            format!("  {}", truncate_string(line, 100)),
                            Style::default().fg(TEXT_SECONDARY),
                        )));
                    }
                }
            }
            Some(ToolCallResult::Glob { pattern, matches }) => {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("Pattern: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(pattern.clone(), Style::default().fg(ACCENT_SECONDARY)),
                    Span::styled(
                        format!(" ({} matches)", matches.len()),
                        Style::default().fg(TEXT_DIM),
                    ),
                ]));

                for (i, m) in matches.iter().enumerate() {
                    if i >= max_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  ... {} more matches", matches.len() - i),
                            Style::default().fg(TEXT_DIM),
                        )));
                        break;
                    }
                    lines.push(Line::from(Span::styled(
                        format!("  {}", m),
                        Style::default().fg(TEXT_SECONDARY),
                    )));
                }
            }
            Some(ToolCallResult::Grep { pattern, matches }) => {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("Pattern: ", Style::default().fg(TEXT_DIM)),
                    Span::styled(pattern.clone(), Style::default().fg(ACCENT_SECONDARY)),
                    Span::styled(
                        format!(" ({} matches)", matches.len()),
                        Style::default().fg(TEXT_DIM),
                    ),
                ]));

                for (i, m) in matches.iter().enumerate() {
                    if i >= max_lines {
                        lines.push(Line::from(Span::styled(
                            format!("  ... {} more matches", matches.len() - i),
                            Style::default().fg(TEXT_DIM),
                        )));
                        break;
                    }
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {}:{}: ", m.file, m.line),
                            Style::default().fg(TEXT_DIM),
                        ),
                        Span::styled(
                            truncate_string(&m.content, 60),
                            Style::default().fg(TEXT_SECONDARY),
                        ),
                    ]));
                }
            }
            Some(ToolCallResult::Task {
                description,
                result,
            }) => {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(description.clone(), Style::default().fg(TEXT_SECONDARY)),
                ]));

                for (i, line) in result.lines().enumerate() {
                    if i >= max_lines {
                        let remaining = result.lines().count() - i;
                        lines.push(Line::from(Span::styled(
                            format!("  ... {} more lines", remaining),
                            Style::default().fg(TEXT_DIM),
                        )));
                        break;
                    }
                    lines.push(Line::from(Span::styled(
                        format!("  {}", truncate_string(line, 100)),
                        Style::default().fg(TEXT_SECONDARY),
                    )));
                }
            }
            Some(ToolCallResult::Generic { summary }) => {
                for line in summary.lines().take(max_lines) {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", truncate_string(line, 100)),
                        Style::default().fg(TEXT_SECONDARY),
                    )));
                }
            }
            None => {
                if let ToolCallStatus::Running = self.status {
                    lines.push(Line::from(Span::styled(
                        "  Running...",
                        Style::default().fg(TEXT_DIM),
                    )));
                }
            }
        }

        if let ToolCallStatus::Failed(ref error) = self.status {
            lines.push(Line::from(Span::styled(
                format!("  Error: {}", truncate_string(error, 80)),
                Style::default().fg(ERROR),
            )));
        }

        lines
    }
}

fn format_tool_display_name(tool_name: &str, params: &serde_json::Value) -> String {
    match tool_name {
        "read" => {
            let path = params
                .get("filePath")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("read {}", shorten_path(path))
        }
        "write" => {
            let path = params
                .get("filePath")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("write {}", shorten_path(path))
        }
        "edit" => {
            let path = params
                .get("filePath")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("edit {}", shorten_path(path))
        }
        "bash" => {
            let cmd = params
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("...");
            let short_cmd = if cmd.len() > 40 {
                format!("{}...", &cmd[..40])
            } else {
                cmd.to_string()
            };
            format!("bash {}", short_cmd)
        }
        "glob" => {
            let pattern = params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("*");
            format!("glob {}", pattern)
        }
        "grep" => {
            let pattern = params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("...");
            format!("grep \"{}\"", truncate_string(pattern, 30))
        }
        "task" => {
            let desc = params
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("subtask");
            format!("task: {}", truncate_string(desc, 40))
        }
        _ => {
            if params.is_null() || params.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                tool_name.to_string()
            } else {
                format!("{} ...", tool_name)
            }
        }
    }
}

fn shorten_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 3 {
        return path.to_string();
    }
    let last_parts: Vec<&str> = parts.iter().rev().take(3).copied().collect();
    format!(
        ".../{}",
        last_parts.into_iter().rev().collect::<Vec<_>>().join("/")
    )
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

pub fn parse_tool_result(
    tool_name: &str,
    params: &serde_json::Value,
    output: Option<&serde_json::Value>,
) -> Option<ToolCallResult> {
    match tool_name {
        "read" => {
            let file_path = params
                .get("filePath")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())?
                .to_string();
            let content = output
                .and_then(|o| o.get("content").or(Some(o)))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let line_count = content.lines().count();
            Some(ToolCallResult::Read {
                file_path,
                content,
                line_count,
            })
        }
        "edit" => {
            let file_path = params
                .get("filePath")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())?
                .to_string();
            let old_string = params
                .get("oldString")
                .or_else(|| params.get("old_string"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let new_string = params
                .get("newString")
                .or_else(|| params.get("new_string"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(ToolCallResult::Edit {
                file_path,
                old_string,
                new_string,
            })
        }
        "write" => {
            let file_path = params
                .get("filePath")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())?
                .to_string();
            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(ToolCallResult::Write { file_path, content })
        }
        "bash" => {
            let command = params
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let output_text = output
                .and_then(|o| o.get("output").or(o.get("stdout")).or(Some(o)))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let exit_code = output
                .and_then(|o| o.get("exitCode").or(o.get("exit_code")))
                .and_then(|v| v.as_i64())
                .map(|c| c as i32);
            Some(ToolCallResult::Bash {
                command,
                output: output_text,
                exit_code,
            })
        }
        "glob" => {
            let pattern = params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let matches = output
                .and_then(|o| o.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Some(ToolCallResult::Glob { pattern, matches })
        }
        "grep" => {
            let pattern = params
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let matches = output
                .and_then(|o| o.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            let file = v.get("file").and_then(|f| f.as_str())?.to_string();
                            let line = v.get("line").and_then(|l| l.as_u64())? as usize;
                            let content = v
                                .get("content")
                                .and_then(|c| c.as_str())
                                .unwrap_or("")
                                .to_string();
                            Some(GrepMatch {
                                file,
                                line,
                                content,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(ToolCallResult::Grep { pattern, matches })
        }
        "task" => {
            let description = params
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let result = output.and_then(|o| o.as_str()).unwrap_or("").to_string();
            Some(ToolCallResult::Task {
                description,
                result,
            })
        }
        _ => {
            let summary = output
                .map(|o| {
                    if let Some(s) = o.as_str() {
                        s.to_string()
                    } else {
                        serde_json::to_string_pretty(o).unwrap_or_default()
                    }
                })
                .unwrap_or_default();
            Some(ToolCallResult::Generic { summary })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_call_info_new_sets_display_name() {
        let params = json!({"filePath": "/src/main.rs"});
        let info = ToolCallInfo::new("read".to_string(), params);
        assert!(info.display_name.contains("main.rs"));
    }

    #[test]
    fn format_tool_display_name_shortens_path() {
        let params = json!({"filePath": "/very/long/path/to/file.rs"});
        let name = format_tool_display_name("read", &params);
        assert!(name.contains("..."));
    }

    #[test]
    fn format_tool_display_name_bash_truncates_command() {
        let params = json!({"command": "echo this is a very long command that should be truncated at some point"});
        let name = format_tool_display_name("bash", &params);
        assert!(name.contains("..."));
    }

    #[test]
    fn parse_tool_result_read() {
        let params = json!({"filePath": "/src/main.rs"});
        let output = json!({"content": "fn main() {}\n"});
        let result = parse_tool_result("read", &params, Some(&output));
        assert!(matches!(result, Some(ToolCallResult::Read { .. })));
    }

    #[test]
    fn parse_tool_result_edit() {
        let params = json!({
            "filePath": "/src/main.rs",
            "oldString": "old code",
            "newString": "new code"
        });
        let result = parse_tool_result("edit", &params, None);
        assert!(matches!(result, Some(ToolCallResult::Edit { .. })));
    }

    #[test]
    fn shorten_path_leaves_short_paths() {
        assert_eq!(shorten_path("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn shorten_path_truncates_long_paths() {
        let short = shorten_path("/very/long/path/to/file.rs");
        assert!(short.starts_with("..."));
        assert!(short.contains("file.rs"));
    }

    #[test]
    fn tool_call_status_transitions() {
        let params = json!({});
        let mut info = ToolCallInfo::new("test".to_string(), params);
        assert!(matches!(info.status, ToolCallStatus::Pending));

        info.set_running();
        assert!(matches!(info.status, ToolCallStatus::Running));

        info.set_completed(ToolCallResult::Generic {
            summary: "done".to_string(),
        });
        assert!(matches!(info.status, ToolCallStatus::Completed));
    }
}
