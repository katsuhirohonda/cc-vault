mod claude_reader;
mod jsonl_parser;
mod db_connection;
mod real_db_connection;
mod db_schema;
mod data_importer;
mod search;
mod cli;

#[cfg(feature = "tui")]
mod tui;

use anyhow::Result;
use cli::Cli;
use std::path::PathBuf;
use dirs::home_dir;

#[cfg(test)]
use db_connection::MockDatabaseConnection;

#[cfg(not(test))]
use crate::real_db_connection::RealDuckDBConnection;
#[cfg(not(test))]
use crate::db_connection::DatabaseConnection;

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse_args();
    
    #[cfg(test)]
    {
        let mut mock_conn = MockDatabaseConnection::new();
        mock_conn.expect_is_connected()
            .returning(|| true);
        cli.execute(&mock_conn)?;
    }
    
    #[cfg(not(test))]
    {
        // Use real DuckDB connection
        let db_path = get_database_path()?;
        let conn = RealDuckDBConnection::with_path(&db_path)?;
        
        // Connect to database
        conn.connect()?;
        
        // Initialize schema if needed
        let schema_manager = crate::db_schema::SchemaManager::new(&conn);
        schema_manager.create_schema()?;
        // Create FTS indexes for search functionality
        schema_manager.create_fts_indexes()?;
        
        // Execute command
        cli.execute(&conn)?;
        
        // Disconnect
        conn.disconnect()?;
    }
    
    Ok(())
}

#[cfg(not(test))]
fn get_database_path() -> Result<PathBuf> {
    let home = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    
    let cc_vault_dir = home.join(".cc-vault");
    
    // Create directory if it doesn't exist
    if !cc_vault_dir.exists() {
        std::fs::create_dir_all(&cc_vault_dir)?;
    }
    
    Ok(cc_vault_dir.join("conversations.db"))
}
