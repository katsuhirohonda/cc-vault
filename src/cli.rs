use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::claude_reader::ClaudeReader;
use crate::jsonl_parser::JsonlParser;
use crate::db_connection::DatabaseConnection;
use crate::data_importer::DataImporter;
use crate::search::{SearchEngine, SearchQuery, SearchMode};

#[cfg(feature = "tui")]
use crate::tui::run_tui;

#[cfg(test)]
use crate::db_connection::MockDatabaseConnection;

#[derive(Debug, Parser)]
#[command(name = "cc-vault")]
#[command(about = "A tool to manage and search Claude Code conversation history")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Import conversations from Claude Code
    Import {
        /// Project path to import (default: all projects)
        #[arg(short, long)]
        project: Option<String>,
        
        /// Force re-import even if already imported
        #[arg(short, long)]
        force: bool,
    },
    
    /// Search conversations
    Search {
        /// Keywords to search for
        keywords: Vec<String>,
        
        /// Search mode (and/or)
        #[arg(short, long, default_value = "and")]
        mode: String,
        
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
        
        /// Date from (e.g., "2024-01-01" or "last week")
        #[arg(long)]
        from: Option<String>,
        
        /// Date to (e.g., "2024-01-31" or "yesterday")
        #[arg(long)]
        to: Option<String>,
        
        /// Show only favorites
        #[arg(short, long)]
        favorites: bool,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    
    /// Mark or unmark conversations as favorite
    Favorite {
        /// Conversation ID
        id: i64,
        
        /// Remove from favorites (default: add to favorites)
        #[arg(short, long)]
        remove: bool,
    },
    
    /// Launch interactive TUI mode
    #[cfg(feature = "tui")]
    Tui,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
    
    pub fn execute(&self, connection: &dyn DatabaseConnection) -> Result<()> {
        match &self.command {
            Commands::Import { project, force } => {
                self.execute_import(connection, project.as_deref(), *force)
            }
            Commands::Search { 
                keywords, 
                mode, 
                project, 
                from, 
                to, 
                favorites, 
                limit 
            } => {
                self.execute_search(
                    connection, 
                    keywords, 
                    mode, 
                    project.as_deref(), 
                    from.as_deref(), 
                    to.as_deref(), 
                    *favorites, 
                    *limit
                )
            }
            Commands::Favorite { id, remove } => {
                self.execute_favorite(connection, *id, *remove)
            }
            #[cfg(feature = "tui")]
            Commands::Tui => {
                run_tui(connection)
            }
        }
    }
    
    fn execute_import(&self, connection: &dyn DatabaseConnection, project: Option<&str>, _force: bool) -> Result<()> {
        let _reader = ClaudeReader::new()?;
        let _parser = JsonlParser::new();
        let _importer = DataImporter::new(connection);
        
        // For now, just return Ok - full implementation to follow
        println!("Importing conversations...");
        if let Some(proj) = project {
            println!("Project: {}", proj);
        }
        
        Ok(())
    }
    
    fn execute_search(
        &self, 
        connection: &dyn DatabaseConnection, 
        keywords: &[String], 
        mode: &str,
        project: Option<&str>,
        _from: Option<&str>,
        _to: Option<&str>,
        favorites: bool,
        limit: usize
    ) -> Result<()> {
        let search_engine = SearchEngine::new(connection);
        
        let search_mode = match mode {
            "or" => SearchMode::Or,
            _ => SearchMode::And,
        };
        
        let query = SearchQuery {
            keywords: keywords.to_vec(),
            mode: search_mode,
            project_filter: project.map(|s| s.to_string()),
            project_filters: None,
            date_from: None, // TODO: Parse date strings
            date_to: None,   // TODO: Parse date strings
            favorites_only: Some(favorites),
            limit: Some(limit),
        };
        
        let results = search_engine.search(&query)?;
        
        println!("Found {} results", results.len());
        for result in results.iter().take(5) {
            println!("- [{}] {}", result.id, result.message_content.as_deref().unwrap_or("(no content)"));
        }
        
        Ok(())
    }
    
    fn execute_favorite(&self, connection: &dyn DatabaseConnection, id: i64, remove: bool) -> Result<()> {
        let search_engine = SearchEngine::new(connection);
        
        if remove {
            search_engine.unmark_as_favorite(id)?;
            println!("Removed conversation {} from favorites", id);
        } else {
            search_engine.mark_as_favorite(id)?;
            println!("Added conversation {} to favorites", id);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_import_command() {
        let args = vec!["cc-vault", "import"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Import { project, force } => {
                assert_eq!(project, None);
                assert_eq!(force, false);
            }
            _ => panic!("Expected Import command"),
        }
    }
    
    #[test]
    fn test_parse_import_with_options() {
        let args = vec!["cc-vault", "import", "--project", "/my/project", "--force"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Import { project, force } => {
                assert_eq!(project, Some("/my/project".to_string()));
                assert_eq!(force, true);
            }
            _ => panic!("Expected Import command"),
        }
    }
    
    #[test]
    fn test_parse_search_command() {
        let args = vec!["cc-vault", "search", "rust", "programming"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Search { keywords, mode, .. } => {
                assert_eq!(keywords, vec!["rust", "programming"]);
                assert_eq!(mode, "and");
            }
            _ => panic!("Expected Search command"),
        }
    }
    
    #[test]
    fn test_parse_search_with_all_options() {
        let args = vec![
            "cc-vault", "search", "test",
            "--mode", "or",
            "--project", "/my/project",
            "--from", "2024-01-01",
            "--to", "2024-01-31",
            "--favorites",
            "--limit", "50"
        ];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Search { 
                keywords, 
                mode, 
                project, 
                from, 
                to, 
                favorites, 
                limit 
            } => {
                assert_eq!(keywords, vec!["test"]);
                assert_eq!(mode, "or");
                assert_eq!(project, Some("/my/project".to_string()));
                assert_eq!(from, Some("2024-01-01".to_string()));
                assert_eq!(to, Some("2024-01-31".to_string()));
                assert_eq!(favorites, true);
                assert_eq!(limit, 50);
            }
            _ => panic!("Expected Search command"),
        }
    }
    
    #[test]
    fn test_parse_favorite_command() {
        let args = vec!["cc-vault", "favorite", "123"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Favorite { id, remove } => {
                assert_eq!(id, 123);
                assert_eq!(remove, false);
            }
            _ => panic!("Expected Favorite command"),
        }
    }
    
    #[test]
    fn test_parse_favorite_remove() {
        let args = vec!["cc-vault", "favorite", "456", "--remove"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Favorite { id, remove } => {
                assert_eq!(id, 456);
                assert_eq!(remove, true);
            }
            _ => panic!("Expected Favorite command"),
        }
    }
    
    #[test]
    fn test_parse_invalid_command() {
        let args = vec!["cc-vault", "invalid"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_err());
    }
    
    #[test]
    fn test_parse_missing_required_arg() {
        let args = vec!["cc-vault", "favorite"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_err());
    }
    
    #[test]
    fn test_execute_import_command() {
        let args = vec!["cc-vault", "import"];
        let cli = Cli::try_parse_from(args).unwrap();
        
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        
        let result = cli.execute(&mock_conn);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_execute_search_command() {
        let args = vec!["cc-vault", "search", "test"];
        let cli = Cli::try_parse_from(args).unwrap();
        
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        
        let result = cli.execute(&mock_conn);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_execute_favorite_command() {
        let args = vec!["cc-vault", "favorite", "123"];
        let cli = Cli::try_parse_from(args).unwrap();
        
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        
        let result = cli.execute(&mock_conn);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_execute_favorite_remove_command() {
        let args = vec!["cc-vault", "favorite", "456", "--remove"];
        let cli = Cli::try_parse_from(args).unwrap();
        
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        
        let result = cli.execute(&mock_conn);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_help_message() {
        let args = vec!["cc-vault", "--help"];
        let result = Cli::try_parse_from(args);
        
        // Help flag should cause an error (clap behavior)
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        
        // Check that help message contains expected content
        assert!(err_str.contains("A tool to manage and search Claude Code conversation history"));
        assert!(err_str.contains("Commands:"));
        assert!(err_str.contains("import"));
        assert!(err_str.contains("search"));
        assert!(err_str.contains("favorite"));
    }
    
    #[test]
    fn test_subcommand_help() {
        let args = vec!["cc-vault", "search", "--help"];
        let result = Cli::try_parse_from(args);
        
        // Help flag should cause an error (clap behavior)
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        
        // Check that search help contains expected options
        assert!(err_str.contains("Search conversations"));
        assert!(err_str.contains("--mode"));
        assert!(err_str.contains("--project"));
        assert!(err_str.contains("--favorites"));
    }
    
    #[cfg(feature = "tui")]
    #[test]
    fn test_parse_tui_command() {
        let args = vec!["cc-vault", "tui"];
        let cli = Cli::try_parse_from(args);
        
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        
        match cli.command {
            Commands::Tui => {
                // TUI command parsed successfully
            }
            _ => panic!("Expected Tui command"),
        }
    }
}