mod claude_reader;
mod jsonl_parser;
mod db_connection;
mod db_schema;
mod data_importer;
mod search;
mod cli;

use anyhow::Result;
use cli::Cli;
use db_connection::MockDatabaseConnection;

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    
    // For now, use a mock database connection
    // TODO: Replace with real database connection
    let mut mock_conn = MockDatabaseConnection::new();
    mock_conn.expect_is_connected()
        .returning(|| true);
    
    // Execute the command
    cli.execute(&mock_conn)?;
    
    Ok(())
}
