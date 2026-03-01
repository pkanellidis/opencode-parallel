//! Slash command parsing and handling.
//!
//! Commands are user inputs starting with `/` that trigger specific actions.

/// A parsed slash command.
#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    /// Show help information.
    Help,
    /// List all projects.
    Projects,
    /// Show current project.
    ProjectCurrent,
    /// Show current path.
    Path,
    /// Clear chat messages.
    Clear,
    /// Create a new session with optional name.
    NewSession(Option<String>),
    /// List all sessions.
    ListSessions,
    /// Rename the current session.
    RenameSession(String),
    /// Delete the current session.
    DeleteSession,
    /// List available models.
    Models,
    /// Open model selector.
    ModelSelect,
    /// Set model directly (provider, model).
    ModelSet(String, String),
    /// Reply to a worker's question (worker_id, message).
    Reply(u32, String),
    /// Show current config.
    Config,
    /// Unknown command.
    Unknown(String),
}

/// A command suggestion for autocomplete.
pub struct CommandSuggestion {
    /// The full command text (e.g., "/help").
    pub command: &'static str,
    /// Description of what the command does.
    pub description: &'static str,
}

/// All available command suggestions.
pub const COMMAND_SUGGESTIONS: &[CommandSuggestion] = &[
    CommandSuggestion {
        command: "/help",
        description: "Show available commands",
    },
    CommandSuggestion {
        command: "/new",
        description: "Create new session",
    },
    CommandSuggestion {
        command: "/sessions",
        description: "List all sessions",
    },
    CommandSuggestion {
        command: "/rename",
        description: "Rename current session",
    },
    CommandSuggestion {
        command: "/delete",
        description: "Delete current session",
    },
    CommandSuggestion {
        command: "/models",
        description: "List available models",
    },
    CommandSuggestion {
        command: "/model",
        description: "Set model (provider/model)",
    },
    CommandSuggestion {
        command: "/reply",
        description: "Reply to worker (/reply #N message)",
    },
    CommandSuggestion {
        command: "/projects",
        description: "List all projects",
    },
    CommandSuggestion {
        command: "/project current",
        description: "Show current project",
    },
    CommandSuggestion {
        command: "/path",
        description: "Show current working path",
    },
    CommandSuggestion {
        command: "/clear",
        description: "Clear chat messages",
    },
    CommandSuggestion {
        command: "/config",
        description: "Show current server config",
    },
];

/// Returns command suggestions matching the given input prefix.
pub fn get_suggestions(input: &str) -> Vec<&'static CommandSuggestion> {
    if !input.starts_with('/') {
        return vec![];
    }

    let search = input.to_lowercase();
    COMMAND_SUGGESTIONS
        .iter()
        .filter(|s| s.command.starts_with(&search))
        .collect()
}

/// Parses a string into a SlashCommand.
///
/// Returns `None` if the input doesn't start with `/`.
pub fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
    if parts.is_empty() {
        return Some(SlashCommand::Help);
    }

    match parts[0].to_lowercase().as_str() {
        "help" | "h" | "?" => Some(SlashCommand::Help),

        "new" | "n" => {
            let name = if parts.len() > 1 {
                Some(parts[1..].join(" "))
            } else {
                None
            };
            Some(SlashCommand::NewSession(name))
        }

        "sessions" | "ls" => Some(SlashCommand::ListSessions),

        "rename" | "mv" => {
            if parts.len() > 1 {
                Some(SlashCommand::RenameSession(parts[1..].join(" ")))
            } else {
                Some(SlashCommand::Unknown("rename requires a name".to_string()))
            }
        }

        "delete" | "del" | "rm" => Some(SlashCommand::DeleteSession),

        "models" => Some(SlashCommand::Models),

        "model" | "m" => {
            if parts.len() > 1 {
                let model_spec = parts[1..].join(" ");
                if let Some((provider, model)) = model_spec.split_once('/') {
                    Some(SlashCommand::ModelSet(
                        provider.to_string(),
                        model.to_string(),
                    ))
                } else {
                    Some(SlashCommand::Unknown(
                        "model requires provider/model format".to_string(),
                    ))
                }
            } else {
                Some(SlashCommand::ModelSelect)
            }
        }

        "projects" | "project" | "proj" | "p" => {
            if parts.len() > 1 && parts[1] == "current" {
                Some(SlashCommand::ProjectCurrent)
            } else {
                Some(SlashCommand::Projects)
            }
        }

        "path" | "pwd" => Some(SlashCommand::Path),

        "clear" | "cls" => Some(SlashCommand::Clear),

        "config" | "cfg" => Some(SlashCommand::Config),

        "reply" | "r" => {
            if parts.len() > 2 {
                let worker_str = parts[1].trim_start_matches('#');
                if let Ok(worker_id) = worker_str.parse::<u32>() {
                    let message = parts[2..].join(" ");
                    Some(SlashCommand::Reply(worker_id, message))
                } else {
                    Some(SlashCommand::Unknown(
                        "reply requires worker number (e.g., /reply #1 yes)".to_string(),
                    ))
                }
            } else {
                Some(SlashCommand::Unknown(
                    "reply requires worker number and message (e.g., /reply #1 yes)".to_string(),
                ))
            }
        }

        cmd => Some(SlashCommand::Unknown(cmd.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_slash_command_tests {
        use super::*;

        #[test]
        fn returns_none_for_non_command() {
            assert!(parse_slash_command("hello").is_none());
            assert!(parse_slash_command("").is_none());
        }

        #[test]
        fn parses_help() {
            assert_eq!(parse_slash_command("/help"), Some(SlashCommand::Help));
            assert_eq!(parse_slash_command("/h"), Some(SlashCommand::Help));
            assert_eq!(parse_slash_command("/?"), Some(SlashCommand::Help));
            assert_eq!(parse_slash_command("/"), Some(SlashCommand::Help));
        }

        #[test]
        fn parses_new_session() {
            assert_eq!(
                parse_slash_command("/new"),
                Some(SlashCommand::NewSession(None))
            );
            assert_eq!(
                parse_slash_command("/new My Session"),
                Some(SlashCommand::NewSession(Some("My Session".to_string())))
            );
            assert_eq!(
                parse_slash_command("/n Test"),
                Some(SlashCommand::NewSession(Some("Test".to_string())))
            );
        }

        #[test]
        fn parses_sessions() {
            assert_eq!(
                parse_slash_command("/sessions"),
                Some(SlashCommand::ListSessions)
            );
            assert_eq!(parse_slash_command("/ls"), Some(SlashCommand::ListSessions));
        }

        #[test]
        fn parses_rename() {
            assert_eq!(
                parse_slash_command("/rename New Name"),
                Some(SlashCommand::RenameSession("New Name".to_string()))
            );
            assert!(matches!(
                parse_slash_command("/rename"),
                Some(SlashCommand::Unknown(_))
            ));
        }

        #[test]
        fn parses_delete() {
            assert_eq!(
                parse_slash_command("/delete"),
                Some(SlashCommand::DeleteSession)
            );
            assert_eq!(
                parse_slash_command("/del"),
                Some(SlashCommand::DeleteSession)
            );
            assert_eq!(
                parse_slash_command("/rm"),
                Some(SlashCommand::DeleteSession)
            );
        }

        #[test]
        fn parses_models() {
            assert_eq!(parse_slash_command("/models"), Some(SlashCommand::Models));
        }

        #[test]
        fn parses_model_select() {
            assert_eq!(
                parse_slash_command("/model"),
                Some(SlashCommand::ModelSelect)
            );
            assert_eq!(parse_slash_command("/m"), Some(SlashCommand::ModelSelect));
        }

        #[test]
        fn parses_model_set() {
            assert_eq!(
                parse_slash_command("/model openai/gpt-4"),
                Some(SlashCommand::ModelSet(
                    "openai".to_string(),
                    "gpt-4".to_string()
                ))
            );
        }

        #[test]
        fn parses_model_set_invalid() {
            assert!(matches!(
                parse_slash_command("/model invalid"),
                Some(SlashCommand::Unknown(_))
            ));
        }

        #[test]
        fn parses_projects() {
            assert_eq!(
                parse_slash_command("/projects"),
                Some(SlashCommand::Projects)
            );
            assert_eq!(
                parse_slash_command("/project"),
                Some(SlashCommand::Projects)
            );
            assert_eq!(parse_slash_command("/p"), Some(SlashCommand::Projects));
        }

        #[test]
        fn parses_project_current() {
            assert_eq!(
                parse_slash_command("/project current"),
                Some(SlashCommand::ProjectCurrent)
            );
        }

        #[test]
        fn parses_path() {
            assert_eq!(parse_slash_command("/path"), Some(SlashCommand::Path));
            assert_eq!(parse_slash_command("/pwd"), Some(SlashCommand::Path));
        }

        #[test]
        fn parses_clear() {
            assert_eq!(parse_slash_command("/clear"), Some(SlashCommand::Clear));
            assert_eq!(parse_slash_command("/cls"), Some(SlashCommand::Clear));
        }

        #[test]
        fn parses_config() {
            assert_eq!(parse_slash_command("/config"), Some(SlashCommand::Config));
            assert_eq!(parse_slash_command("/cfg"), Some(SlashCommand::Config));
        }

        #[test]
        fn parses_reply() {
            assert_eq!(
                parse_slash_command("/reply #1 yes please"),
                Some(SlashCommand::Reply(1, "yes please".to_string()))
            );
            assert_eq!(
                parse_slash_command("/reply 2 no"),
                Some(SlashCommand::Reply(2, "no".to_string()))
            );
            assert_eq!(
                parse_slash_command("/r #3 ok"),
                Some(SlashCommand::Reply(3, "ok".to_string()))
            );
        }

        #[test]
        fn parses_reply_invalid() {
            assert!(matches!(
                parse_slash_command("/reply"),
                Some(SlashCommand::Unknown(_))
            ));
            assert!(matches!(
                parse_slash_command("/reply #1"),
                Some(SlashCommand::Unknown(_))
            ));
            assert!(matches!(
                parse_slash_command("/reply abc hello"),
                Some(SlashCommand::Unknown(_))
            ));
        }

        #[test]
        fn parses_unknown_command() {
            assert_eq!(
                parse_slash_command("/foobar"),
                Some(SlashCommand::Unknown("foobar".to_string()))
            );
        }

        #[test]
        fn handles_whitespace() {
            assert_eq!(parse_slash_command("  /help  "), Some(SlashCommand::Help));
        }

        #[test]
        fn case_insensitive() {
            assert_eq!(parse_slash_command("/HELP"), Some(SlashCommand::Help));
            assert_eq!(parse_slash_command("/Help"), Some(SlashCommand::Help));
        }
    }

    mod get_suggestions_tests {
        use super::*;

        #[test]
        fn returns_empty_for_non_command() {
            assert!(get_suggestions("hello").is_empty());
        }

        #[test]
        fn returns_all_for_slash_only() {
            let suggestions = get_suggestions("/");
            assert!(!suggestions.is_empty());
        }

        #[test]
        fn filters_by_prefix() {
            let suggestions = get_suggestions("/mo");
            assert!(suggestions.iter().all(|s| s.command.starts_with("/mo")));
        }

        #[test]
        fn returns_exact_match() {
            let suggestions = get_suggestions("/help");
            assert!(suggestions.iter().any(|s| s.command == "/help"));
        }
    }
}
