mod claude_reader;
mod jsonl_parser;
mod db_connection;
mod db_schema;
mod data_importer;
mod search;
mod cli;

#[cfg(feature = "tui")]
mod tui;

use anyhow::Result;
use cli::Cli;

#[cfg(test)]
use db_connection::MockDatabaseConnection;

#[cfg(not(test))]
use db_connection::DatabaseConnection;

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    
    // For now, use a mock database connection
    // TODO: Replace with real database connection
    #[cfg(test)]
    {
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        cli.execute(&mock_conn)?;
    }
    
    #[cfg(not(test))]
    {
        // For now, we'll create a simple mock implementation
        // This will be replaced with a real DuckDB connection later
        struct TempMockConnection;
        
        impl DatabaseConnection for TempMockConnection {
            fn connect(&self) -> Result<()> {
                Ok(())
            }
            
            fn disconnect(&self) -> Result<()> {
                Ok(())
            }
            
            fn is_connected(&self) -> bool {
                true
            }
            
            fn execute(&self, _query: &str) -> Result<()> {
                Ok(())
            }
        }
        
        let conn = TempMockConnection;
        cli.execute(&conn)?;
    }
    
    Ok(())
}
