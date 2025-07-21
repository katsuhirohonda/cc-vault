# cc-vault Development TODO

Each task represents one commit. All commit messages and code comments should be in English.

## Development Process
1. **ALWAYS output guidelines before starting any task**
2. Follow TDD methodology:
   - Red: Write a failing test first
   - Green: Write minimum code to make the test pass
   - Refactor: Improve the code while keeping tests green

## High Priority

### 1. Initialize Rust project [DONE]
- cargo init
- Create .devcontainer directory
- Configure devcontainer.json (Rust, Docker, DuckDB)
- Setup basic Cargo.toml dependencies
**Commit:** `feat: initialize Rust project with devcontainer setup`

### 2. Claude Code data reader module (TDD)
**Guidelines:**
- Start with test for checking directory existence
- Use mockall for filesystem mocking if needed
- Focus on one behavior at a time

**Red-Green-Refactor cycles:**
1. Test: Directory existence check → Implement → Refactor
2. Test: List project directories → Implement → Refactor
3. Test: Find .jsonl files → Implement → Refactor
4. Test: Error handling scenarios → Implement → Refactor

**Commit:** `feat: add Claude Code data reader module`

### 3. JSON Lines parser (TDD)
**Guidelines:**
- Define data structures first (structs for messages)
- Test parsing with sample JSON data
- Handle edge cases systematically

**Red-Green-Refactor cycles:**
1. Test: Parse valid single JSON message → Implement → Refactor
2. Test: Parse multiple JSONL lines → Implement → Refactor
3. Test: Handle invalid JSON gracefully → Implement → Refactor
4. Test: Handle empty lines and malformed data → Implement → Refactor

**Commit:** `feat: implement JSON Lines parser`

### 4. DuckDB connection module (TDD)
**Guidelines:**
- Mock DuckDB connection for unit tests
- Integration tests with actual Docker container
- Test connection failures and retries

**Red-Green-Refactor cycles:**
1. Test: Successful connection → Implement → Refactor
2. Test: Connection error handling → Implement → Refactor
3. Test: Retry logic with backoff → Implement → Refactor
4. Test: Connection pooling → Implement → Refactor

**Commit:** `feat: add DuckDB connection module`

### 5. Database schema creation (TDD)
**Guidelines:**
- Test schema creation with in-memory DuckDB
- Verify table structure and indexes
- Test migration idempotency

**Red-Green-Refactor cycles:**
1. Test: Create conversations table → Implement → Refactor
2. Test: Create FTS indexes → Implement → Refactor
3. Test: Schema migration (up/down) → Implement → Refactor
4. Test: Idempotent schema creation → Implement → Refactor

**Commit:** `feat: create database schema with FTS indexes`

### 6. Import functionality (TDD)
**Guidelines:**
- Test with sample JSONL data
- Mock database for unit tests
- Test duplicate handling scenarios

**Red-Green-Refactor cycles:**
1. Test: Insert single conversation → Implement → Refactor
2. Test: UUID duplicate detection → Implement → Refactor
3. Test: Update existing with new timestamp → Implement → Refactor
4. Test: Bulk import performance → Implement → Refactor

**Commit:** `feat: implement data import with duplicate detection`

## Medium Priority

### 7. Search - Keyword search (TDD)
**Guidelines:**
- Test search queries with sample data
- Test ranking algorithm
- Test AND/OR logic combinations

**Red-Green-Refactor cycles:**
1. Test: Simple keyword search → Implement → Refactor
2. Test: Multiple keywords (AND) → Implement → Refactor
3. Test: Multiple keywords (OR) → Implement → Refactor
4. Test: Search result ranking → Implement → Refactor

**Commit:** `feat: add keyword search with FTS`

### 8. Search - Date range filter (TDD)
**Guidelines:**
- Test various date formats
- Test relative date parsing
- Test edge cases (invalid dates)

**Red-Green-Refactor cycles:**
1. Test: Absolute date range → Implement → Refactor
2. Test: Relative dates ("last week") → Implement → Refactor
3. Test: Edge cases and errors → Implement → Refactor

**Commit:** `feat: implement date range filtering`

### 9. Search - Project filter (TDD)
**Guidelines:**
- Test single project filter
- Test multiple project selection
- Test non-existent projects

**Red-Green-Refactor cycles:**
1. Test: Single project filter → Implement → Refactor
2. Test: Multiple projects (IN clause) → Implement → Refactor
3. Test: Invalid project handling → Implement → Refactor

**Commit:** `feat: add project-based filtering`

### 10. Favorite functionality (TDD)
**Guidelines:**
- Test favorite flag updates
- Test listing favorites
- Test combining with other filters

**Red-Green-Refactor cycles:**
1. Test: Mark as favorite → Implement → Refactor
2. Test: Unmark favorite → Implement → Refactor
3. Test: List all favorites → Implement → Refactor
4. Test: Filter with other criteria → Implement → Refactor

**Commit:** `feat: implement favorite marking and filtering`

### 11. CLI interface (TDD)
**Guidelines:**
- Test argument parsing
- Test subcommand routing
- Test help output

**Red-Green-Refactor cycles:**
1. Test: Parse basic arguments → Implement → Refactor
2. Test: Import subcommand → Implement → Refactor
3. Test: Search subcommand → Implement → Refactor
4. Test: Help and error messages → Implement → Refactor

**Commit:** `feat: create CLI interface with clap`

### 13. Docker Compose configuration
**Guidelines:**
- No TDD needed (configuration file)
- Test container startup manually
- Ensure persistent volume

**Tasks:**
- Create docker-compose.yml
- DuckDB container setup
- Volume mounting for persistence
- Network configuration

**Commit:** `feat: add Docker Compose for DuckDB`

## Low Priority

### 12. TUI interface (TDD)
**Guidelines:**
- Mock terminal for testing
- Test keyboard event handling
- Test UI state management

**Red-Green-Refactor cycles:**
1. Test: Initialize TUI \u2192 Implement \u2192 Refactor
2. Test: Keyboard navigation \u2192 Implement \u2192 Refactor
3. Test: Search input handling \u2192 Implement \u2192 Refactor
4. Test: Result selection \u2192 Implement \u2192 Refactor

**Commit:** `feat: implement TUI with ratatui`

## Development Guidelines

- All code comments in English
- Follow Rust naming conventions
- Use descriptive variable names
- Add error handling for all I/O operations
