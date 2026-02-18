# Contributing to opencode-parallel

Thank you for your interest in contributing to opencode-parallel! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.70+ (`rustup` recommended)
- Git
- A code editor (VS Code with rust-analyzer recommended)

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/opencode-parallel.git
cd opencode-parallel

# Build the project
cargo build

# Run tests
cargo test

# Run the application
cargo run

# Run with specific command
cargo run -- tui --agents 4
```

## Development Workflow

### Branch Strategy

- `main` - Stable releases
- `dev` - Development branch
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates

### Making Changes

1. Fork the repository
2. Create a feature branch from `dev`
3. Make your changes
4. Write or update tests
5. Ensure all tests pass
6. Submit a pull request

```bash
git checkout dev
git checkout -b feature/my-new-feature
# Make changes
cargo test
cargo fmt
cargo clippy
git add .
git commit -m "feat: add new feature"
git push origin feature/my-new-feature
```

## Code Style

### Rust Conventions

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Maximum line length: 100 characters

### Naming Conventions

- Types: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Documentation

- All public items must have doc comments
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in doc comments

```rust
/// Executes an agent task asynchronously.
///
/// # Arguments
///
/// * `agent` - The agent configuration to execute
/// * `tx` - Channel sender for progress updates
///
/// # Returns
///
/// Returns the updated agent configuration with results
///
/// # Errors
///
/// Returns error if API call fails or agent configuration is invalid
///
/// # Example
///
/// ```no_run
/// let agent = AgentConfig::new("anthropic", "claude-3", "Fix bug");
/// let (tx, rx) = mpsc::channel(100);
/// let result = run_agent(agent, tx).await?;
/// ```
pub async fn run_agent(
    agent: AgentConfig, 
    tx: mpsc::Sender<String>
) -> Result<AgentConfig> {
    // Implementation
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_agent_lifecycle

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = AgentConfig::new("anthropic", "claude-3", "task");
        assert_eq!(agent.status, AgentStatus::Pending);
        assert!(agent.id.len() > 0);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        // Async test implementation
    }
}
```

## Project Structure

```
opencode-parallel/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── agent.rs         # Agent logic and state
│   ├── executor.rs      # Task execution
│   └── tui.rs          # Terminal UI
├── tests/              # Integration tests
├── examples/           # Example usage
├── Cargo.toml          # Dependencies
└── README.md           # Documentation
```

## Adding Features

### Adding a New Command

1. Add command to `Commands` enum in `main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    
    /// Your new command description
    MyCommand {
        #[arg(short, long)]
        my_arg: String,
    },
}
```

2. Handle command in `main()`:

```rust
match cli.command {
    // ... existing handlers
    
    Some(Commands::MyCommand { my_arg }) => {
        my_module::handle_command(&my_arg)?;
    }
}
```

3. Implement the handler in appropriate module

### Adding a New Provider

1. Create provider module in `src/providers/`:

```rust
// src/providers/my_provider.rs
use anyhow::Result;

pub struct MyProvider {
    api_key: String,
}

impl MyProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
    
    pub async fn execute(&self, task: &str) -> Result<String> {
        // Implementation
    }
}
```

2. Register provider in `agent.rs`

### UI Components

When adding TUI components:

1. Create widget in `tui.rs`
2. Add to layout constraints
3. Handle relevant keyboard events
4. Update help text

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(tui): add agent cancellation support

Add ability to cancel running agents via 'c' key in TUI.
Updates agent status to Cancelled and stops execution.

Closes #42

fix(executor): prevent panic on empty task list

Check for empty tasks before spawning agents.

docs(readme): update installation instructions

Add cargo install method and update brew formula.
```

## Pull Request Process

1. **Update Documentation**: Update README.md, ARCHITECTURE.md, or other docs if needed
2. **Add Tests**: Include tests for new features
3. **Update CHANGELOG**: Add entry to CHANGELOG.md (if it exists)
4. **Pass CI**: Ensure all CI checks pass
5. **Request Review**: Request review from maintainers

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe tests added or how changes were tested

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-reviewed code
- [ ] Commented complex sections
- [ ] Updated documentation
- [ ] Added tests
- [ ] All tests passing
- [ ] No new warnings
```

## Performance Guidelines

- Prefer async operations for I/O
- Use bounded channels to prevent memory issues
- Profile before optimizing
- Document performance-critical sections

## Security

- Never commit API keys or secrets
- Use environment variables for sensitive data
- Validate all user input
- Sanitize file paths

## Getting Help

- **Issues**: Use GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions
- **Discord**: Join our Discord server (coming soon)

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other contributors

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project documentation

Thank you for contributing to opencode-parallel!
