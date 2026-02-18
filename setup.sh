#!/bin/bash

set -e

echo "🚀 Setting up opencode-parallel development environment..."

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed."
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust installed successfully"
else
    echo "✅ Rust is already installed ($(rustc --version))"
fi

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found in PATH"
    echo "Please restart your terminal or run: source $HOME/.cargo/env"
    exit 1
fi

# Update Rust
echo "🔄 Updating Rust..."
rustup update

# Install useful tools
echo "🔧 Installing development tools..."
rustup component add rustfmt clippy

# Build the project
echo "🏗️  Building project..."
cargo build

# Run tests
echo "🧪 Running tests..."
cargo test

# Create config directory
CONFIG_DIR="$HOME/.config/opencode-parallel"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "📁 Creating config directory at $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"
fi

echo ""
echo "✨ Setup complete!"
echo ""
echo "Quick start commands:"
echo "  cargo run                    # Start TUI with default settings"
echo "  cargo run -- tui --agents 8  # Start TUI with 8 agents"
echo "  cargo run -- providers       # List configured providers"
echo "  cargo run -- auth anthropic  # Configure Anthropic API key"
echo ""
echo "Development commands:"
echo "  cargo test                   # Run tests"
echo "  cargo fmt                    # Format code"
echo "  cargo clippy                 # Run linter"
echo "  cargo build --release        # Build optimized binary"
echo ""
echo "📚 See README.md for more information"
