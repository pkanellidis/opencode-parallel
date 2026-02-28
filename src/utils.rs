//! Shared utility functions used across modules.

/// Truncates a string to a maximum number of characters, adding "..." if truncated.
///
/// # Examples
/// ```
/// use opencode_parallel::utils::truncate_str;
///
/// assert_eq!(truncate_str("hello", 10), "hello");
/// assert_eq!(truncate_str("hello world", 5), "hello...");
/// ```
pub fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        format!("{}...", chars[..max_chars].iter().collect::<String>())
    } else {
        s.to_string()
    }
}

/// Extracts question text from a JSON value representing a question tool input.
///
/// Handles both direct question objects and arrays of questions.
pub fn extract_question_text(input: &serde_json::Value) -> String {
    // Handle case where input is a string that needs to be parsed
    if let Some(input_str) = input.as_str() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(input_str) {
            return extract_question_text(&parsed);
        }
    }

    // Single question object
    if let Some(q) = input.get("question").and_then(|v| v.as_str()) {
        return q.to_string();
    }

    // Array of questions
    if let Some(questions) = input.get("questions").and_then(|v| v.as_array()) {
        let texts: Vec<&str> = questions
            .iter()
            .filter_map(|q| q.get("question").and_then(|v| v.as_str()))
            .collect();
        return texts.join("\n");
    }

    String::new()
}

/// Formats a tool call for display, showing the tool name and relevant input parameters.
pub fn format_tool_display(tool_name: &str, input: &serde_json::Value) -> String {
    if input.is_null() || input.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        tool_name.to_string()
    } else {
        format!("{} {}", tool_name, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    mod truncate_str_tests {
        use super::*;

        #[test]
        fn returns_original_when_shorter_than_max() {
            assert_eq!(truncate_str("hello", 10), "hello");
        }

        #[test]
        fn returns_original_when_equal_to_max() {
            assert_eq!(truncate_str("hello", 5), "hello");
        }

        #[test]
        fn truncates_with_ellipsis_when_longer_than_max() {
            assert_eq!(truncate_str("hello world", 5), "hello...");
        }

        #[test]
        fn handles_empty_string() {
            assert_eq!(truncate_str("", 5), "");
        }

        #[test]
        fn handles_unicode_characters() {
            assert_eq!(truncate_str("héllo wörld", 5), "héllo...");
        }

        #[test]
        fn handles_zero_max() {
            assert_eq!(truncate_str("hello", 0), "...");
        }
    }

    mod extract_question_text_tests {
        use super::*;

        #[test]
        fn extracts_single_question() {
            let input = json!({"question": "What is your name?"});
            assert_eq!(extract_question_text(&input), "What is your name?");
        }

        #[test]
        fn extracts_multiple_questions() {
            let input = json!({
                "questions": [
                    {"question": "First question?"},
                    {"question": "Second question?"}
                ]
            });
            assert_eq!(
                extract_question_text(&input),
                "First question?\nSecond question?"
            );
        }

        #[test]
        fn handles_string_input_needing_parse() {
            let input = json!(r#"{"question": "Parsed question?"}"#);
            assert_eq!(extract_question_text(&input), "Parsed question?");
        }

        #[test]
        fn returns_empty_for_invalid_input() {
            let input = json!({"foo": "bar"});
            assert_eq!(extract_question_text(&input), "");
        }

        #[test]
        fn returns_empty_for_null() {
            let input = json!(null);
            assert_eq!(extract_question_text(&input), "");
        }
    }

    mod format_tool_display_tests {
        use super::*;

        #[test]
        fn returns_just_tool_name_for_null_input() {
            let input = json!(null);
            assert_eq!(format_tool_display("read", &input), "read");
        }

        #[test]
        fn returns_just_tool_name_for_empty_object() {
            let input = json!({});
            assert_eq!(format_tool_display("read", &input), "read");
        }

        #[test]
        fn includes_input_for_non_empty_object() {
            let input = json!({"path": "/foo/bar"});
            assert_eq!(
                format_tool_display("read", &input),
                r#"read {"path":"/foo/bar"}"#
            );
        }
    }
}
