use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use crate::search::{SearchResult, SearchEngine, SearchQuery, SearchMode};
use crate::db_connection::DatabaseConnection;

#[derive(Debug, PartialEq)]
pub enum AppState {
    SearchInput,
    ResultsList,
    ViewingResult,
}

pub struct App {
    pub state: AppState,
    pub search_input: String,
    pub search_results: Vec<SearchResult>,
    pub selected_index: usize,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::SearchInput,
            search_input: String::new(),
            search_results: Vec::new(),
            selected_index: 0,
            should_quit: false,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            AppState::SearchInput => self.handle_search_input(key),
            AppState::ResultsList => self.handle_results_list(key),
            AppState::ViewingResult => self.handle_viewing_result(key),
        }
    }

    fn handle_search_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.search_input.push(c);
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            KeyCode::Enter => {
                if !self.search_input.is_empty() {
                    self.state = AppState::ResultsList;
                }
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_results_list(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_index < self.search_results.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if !self.search_results.is_empty() {
                    self.state = AppState::ViewingResult;
                }
            }
            KeyCode::Esc => {
                self.state = AppState::SearchInput;
                self.selected_index = 0;
            }
            _ => {}
        }
    }

    fn handle_viewing_result(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.state = AppState::ResultsList;
            }
            _ => {}
        }
    }

    pub fn perform_search(&mut self, connection: &dyn DatabaseConnection) -> Result<()> {
        let search_engine = SearchEngine::new(connection);
        let keywords: Vec<String> = self.search_input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let query = SearchQuery {
            keywords,
            mode: SearchMode::And,
            ..Default::default()
        };

        self.search_results = search_engine.search(&query)?;
        self.selected_index = 0;
        Ok(())
    }
}

pub fn run_tui(connection: &dyn DatabaseConnection) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, connection);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    connection: &dyn DatabaseConnection,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') && app.state == AppState::ResultsList {
                return Ok(());
            }
            
            app.handle_key(key.code);
            
            // Perform search when entering results list
            if app.state == AppState::ResultsList && app.search_results.is_empty() {
                app.perform_search(connection)?;
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());

    render_search_input(f, app, chunks[0]);
    
    match app.state {
        AppState::SearchInput => render_help(f, chunks[1]),
        AppState::ResultsList => render_results_list(f, app, chunks[1]),
        AppState::ViewingResult => render_result_view(f, app, chunks[1]),
    }
}

fn render_search_input(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.search_input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(input, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("Enter keywords to search Claude Code conversations"),
        Line::from(""),
        Line::from("Commands:"),
        Line::from("  Enter - Search"),
        Line::from("  Esc   - Quit"),
    ];
    
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, area);
}

fn render_results_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let content = format!(
                "[{}] {} - {}",
                result.id,
                result.timestamp.format("%Y-%m-%d %H:%M"),
                result.message_content.as_deref().unwrap_or("(empty)")
                    .chars()
                    .take(50)
                    .collect::<String>()
            );
            
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Results"));
    f.render_widget(list, area);
}

fn render_result_view(f: &mut Frame, app: &App, area: Rect) {
    if let Some(result) = app.search_results.get(app.selected_index) {
        let text = vec![
            Line::from(vec![
                Span::raw("ID: "),
                Span::styled(result.id.to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("UUID: "),
                Span::styled(&result.uuid, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Project: "),
                Span::styled(&result.project_path, Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::raw("Time: "),
                Span::styled(
                    result.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from("Content:"),
            Line::from(result.message_content.as_deref().unwrap_or("(empty)")),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_connection::MockDatabaseConnection;

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        assert_eq!(app.state, AppState::SearchInput);
        assert_eq!(app.search_input, "");
        assert_eq!(app.search_results.len(), 0);
        assert_eq!(app.selected_index, 0);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_search_input_handling() {
        let mut app = App::new();
        
        // Type some characters
        app.handle_key(KeyCode::Char('t'));
        app.handle_key(KeyCode::Char('e'));
        app.handle_key(KeyCode::Char('s'));
        app.handle_key(KeyCode::Char('t'));
        assert_eq!(app.search_input, "test");
        
        // Backspace
        app.handle_key(KeyCode::Backspace);
        assert_eq!(app.search_input, "tes");
        
        // Enter should transition to results list
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.state, AppState::ResultsList);
    }

    #[test]
    fn test_results_navigation() {
        let mut app = App::new();
        app.state = AppState::ResultsList;
        
        // Add some mock results
        app.search_results = vec![
            SearchResult {
                id: 1,
                uuid: "uuid1".to_string(),
                session_id: "session1".to_string(),
                message_content: Some("Result 1".to_string()),
                message_role: Some("user".to_string()),
                project_path: "/project1".to_string(),
                timestamp: chrono::Utc::now(),
                rank: 0.9,
                is_favorite: false,
            },
            SearchResult {
                id: 2,
                uuid: "uuid2".to_string(),
                session_id: "session2".to_string(),
                message_content: Some("Result 2".to_string()),
                message_role: Some("assistant".to_string()),
                project_path: "/project2".to_string(),
                timestamp: chrono::Utc::now(),
                rank: 0.8,
                is_favorite: false,
            },
        ];
        
        // Test navigation
        assert_eq!(app.selected_index, 0);
        
        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected_index, 1);
        
        app.handle_key(KeyCode::Down);
        assert_eq!(app.selected_index, 1); // Should not go beyond last item
        
        app.handle_key(KeyCode::Up);
        assert_eq!(app.selected_index, 0);
        
        app.handle_key(KeyCode::Up);
        assert_eq!(app.selected_index, 0); // Should not go below 0
    }

    #[test]
    fn test_state_transitions() {
        let mut app = App::new();
        
        // SearchInput -> ResultsList
        app.search_input = "test".to_string();
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.state, AppState::ResultsList);
        
        // Add a result for viewing
        app.search_results.push(SearchResult {
            id: 1,
            uuid: "uuid1".to_string(),
            session_id: "session1".to_string(),
            message_content: Some("Test result".to_string()),
            message_role: Some("user".to_string()),
            project_path: "/test".to_string(),
            timestamp: chrono::Utc::now(),
            rank: 0.9,
            is_favorite: false,
        });
        
        // ResultsList -> ViewingResult
        app.handle_key(KeyCode::Enter);
        assert_eq!(app.state, AppState::ViewingResult);
        
        // ViewingResult -> ResultsList
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.state, AppState::ResultsList);
        
        // ResultsList -> SearchInput
        app.handle_key(KeyCode::Esc);
        assert_eq!(app.state, AppState::SearchInput);
    }

    #[test]
    fn test_perform_search() {
        let mut app = App::new();
        app.search_input = "test".to_string();
        
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        
        let result = app.perform_search(&mock_conn);
        assert!(result.is_ok());
    }

    #[test]
    fn test_quit_handling() {
        let mut app = App::new();
        
        // Esc in search input should quit
        app.handle_key(KeyCode::Esc);
        assert!(app.should_quit);
    }
}