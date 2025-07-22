use anyhow::{anyhow, Result};
use duckdb::{Connection, params};
use std::sync::{Arc, Mutex};
use std::path::Path;
use crate::db_connection::{DatabaseConnection, ConnectionConfig};

pub struct RealDuckDBConnection {
    connection: Arc<Mutex<Option<Connection>>>,
    config: ConnectionConfig,
}

impl RealDuckDBConnection {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            config,
        }
    }
    
    pub fn with_path(db_path: &Path) -> Result<Self> {
        let config = ConnectionConfig {
            database: db_path.to_string_lossy().to_string(),
            ..Default::default()
        };
        Ok(Self::new(config))
    }
}

impl DatabaseConnection for RealDuckDBConnection {
    fn connect(&self) -> Result<()> {
        let mut conn_guard = self.connection.lock()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;
        
        // Connect to DuckDB (creates file if doesn't exist)
        let conn = if self.config.database.is_empty() || self.config.database == ":memory:" {
            Connection::open_in_memory()
                .map_err(|e| anyhow!("Failed to create in-memory DuckDB: {}", e))?
        } else {
            Connection::open(&self.config.database)
                .map_err(|e| anyhow!("Failed to connect to DuckDB: {}", e))?
        };
        
        *conn_guard = Some(conn);
        Ok(())
    }
    
    fn disconnect(&self) -> Result<()> {
        let mut conn_guard = self.connection.lock()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;
        
        *conn_guard = None;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connection.lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }
    
    fn execute(&self, query: &str) -> Result<()> {
        let conn_guard = self.connection.lock()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;
        
        let conn = conn_guard.as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        
        conn.execute(query, params![])
            .map_err(|e| anyhow!("Failed to execute query: {}", e))?;
        
        Ok(())
    }
}

// Extended connection with query support
pub trait ExtendedDatabaseConnection: DatabaseConnection {
    fn query_row<T, F>(&self, query: &str, mapper: F) -> Result<Option<T>>
    where
        F: FnOnce(&duckdb::Row) -> Result<T>;
        
    fn query_all<T, F>(&self, query: &str, mapper: F) -> Result<Vec<T>>
    where
        F: Fn(&duckdb::Row) -> Result<T>;
        
    fn execute_batch(&self, queries: &[&str]) -> Result<()>;
}

impl ExtendedDatabaseConnection for RealDuckDBConnection {
    fn query_row<T, F>(&self, query: &str, mapper: F) -> Result<Option<T>>
    where
        F: FnOnce(&duckdb::Row) -> Result<T>
    {
        let conn_guard = self.connection.lock()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;
        
        let conn = conn_guard.as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        
        let mut stmt = conn.prepare(query)
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;
        
        let mut rows = stmt.query(params![])
            .map_err(|e| anyhow!("Failed to execute query: {}", e))?;
        
        match rows.next()? {
            Some(row) => Ok(Some(mapper(&row)?)),
            None => Ok(None),
        }
    }
    
    fn query_all<T, F>(&self, query: &str, mapper: F) -> Result<Vec<T>>
    where
        F: Fn(&duckdb::Row) -> Result<T>
    {
        let conn_guard = self.connection.lock()
            .map_err(|e| anyhow!("Lock poisoned: {}", e))?;
        
        let conn = conn_guard.as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;
        
        let mut stmt = conn.prepare(query)
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;
        
        let mut rows = stmt.query(params![])
            .map_err(|e| anyhow!("Failed to execute query: {}", e))?;
        
        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(mapper(&row)?);
        }
        
        Ok(results)
    }
    
    fn execute_batch(&self, queries: &[&str]) -> Result<()> {
        for query in queries {
            self.execute(query)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_real_duckdb_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let conn = RealDuckDBConnection::with_path(&db_path).unwrap();
        
        // Test connection
        assert!(conn.connect().is_ok());
        assert!(conn.is_connected());
        
        // Test table creation
        let create_table = r#"
            CREATE TABLE test_table (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
        "#;
        assert!(conn.execute(create_table).is_ok());
        
        // Test disconnect
        assert!(conn.disconnect().is_ok());
        assert!(!conn.is_connected());
    }
    
    #[test]
    fn test_in_memory_connection() {
        let config = ConnectionConfig::default();
        let conn = RealDuckDBConnection::new(config);
        
        assert!(conn.connect().is_ok());
        assert!(conn.is_connected());
        
        // Create and query table
        conn.execute("CREATE TABLE test (id INTEGER, value TEXT)").unwrap();
        conn.execute("INSERT INTO test VALUES (1, 'hello')").unwrap();
        
        // Verify data
        let result: Option<String> = conn.query_row(
            "SELECT value FROM test WHERE id = 1",
            |row| Ok(row.get(0)?)
        ).unwrap();
        
        assert_eq!(result, Some("hello".to_string()));
    }
}