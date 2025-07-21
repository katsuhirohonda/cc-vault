# cc-vault

A Rust-based tool to manage and search Claude Code conversation history.

## Features

- Import Claude Code conversations from local storage
- Full-text search with DuckDB
- Filter by date range, project, and favorites
- CLI and TUI interfaces
- Docker-based DuckDB storage

## Quick Start

```bash
# Build the project
cargo build --release

# Import all conversations
cc-vault import

# Search conversations
cc-vault search "rust error handling"

# Interactive TUI mode
cc-vault interactive
```

## Development

This project uses devcontainers for a consistent development environment. Open in VS Code with the Dev Containers extension installed.

## Requirements

- Rust 1.75+
- Docker
- Access to `~/.claude/projects/` directory

See [REQUIREMENTS.md](REQUIREMENTS.md) for detailed specifications.
