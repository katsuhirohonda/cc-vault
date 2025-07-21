# cc-vault Development TODO

Each task represents one commit. All commit messages and code comments should be in English.

## High Priority

### 1. Initialize Rust project [DONE]
- cargo init
- Create .devcontainer directory
- Configure devcontainer.json (Rust, Docker, DuckDB)
- Setup basic Cargo.toml dependencies
**Commit:** `feat: initialize Rust project with devcontainer setup`

### 2. Claude Code data reader module
- Check ~/.claude/projects/ directory existence
- List project directories
- Find .jsonl files
- Error handling
**Commit:** `feat: add Claude Code data reader module`

### 3. JSON Lines parser
- Parse single JSON message
- Parse multiple JSONL lines
- Handle invalid JSON
- Define data structures
**Commit:** `feat: implement JSON Lines parser`

### 4. DuckDB connection module
- Connect to Docker container
- Connection error handling
- Retry logic
- Connection pooling
**Commit:** `feat: add DuckDB connection module`

### 5. Database schema creation
- Create conversations table
- Create FTS indexes
- Schema migration management
**Commit:** `feat: create database schema with FTS indexes`

### 6. Import functionality
- Insert new data
- UUID duplicate checking (UPSERT)
- Diff detection with last update time
- Bulk import optimization
**Commit:** `feat: implement data import with duplicate detection`

## Medium Priority

### 7. Search - Keyword search
- FTS search implementation
- Multiple keyword support (AND/OR)
- Search result ranking
**Commit:** `feat: add keyword search with FTS`

### 8. Search - Date range filter
- Date range WHERE clause
- Relative date parsing ("last week", etc.)
**Commit:** `feat: implement date range filtering`

### 9. Search - Project filter
- Project path filtering
- Multiple project selection with IN clause
**Commit:** `feat: add project-based filtering`

### 10. Favorite functionality
- Update favorite flag
- List favorites
- Filter by favorites
**Commit:** `feat: implement favorite marking and filtering`

### 11. CLI interface
- Setup clap for argument parsing
- Implement subcommands (import, sync, search)
- Help messages
**Commit:** `feat: create CLI interface with clap`

### 13. Docker Compose configuration
- Create docker-compose.yml
- DuckDB container setup
- Volume mounting
- Network configuration
**Commit:** `feat: add Docker Compose for DuckDB`

## Low Priority

### 12. TUI interface
- Setup ratatui
- Keyboard event handling
- Search result display
- Interactive selection
**Commit:** `feat: implement TUI with ratatui`

## Development Guidelines

- All code comments in English
- Follow Rust naming conventions
- Use descriptive variable names
- Add error handling for all I/O operations
