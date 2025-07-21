use anyhow::{anyhow, Result};
use crate::db_connection::DatabaseConnection;

pub const CREATE_CONVERSATIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS conversations (
    id INTEGER PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    parent_uuid TEXT,
    session_id TEXT NOT NULL,
    user_type TEXT NOT NULL,
    message_type TEXT NOT NULL,
    message_role TEXT,
    message_content TEXT,
    project_path TEXT NOT NULL,
    cwd TEXT NOT NULL,
    git_branch TEXT,
    version TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    is_favorite BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
)"#;

pub const CREATE_UUID_INDEX: &str = 
    "CREATE INDEX IF NOT EXISTS idx_conversations_uuid ON conversations(uuid)";

pub const CREATE_SESSION_INDEX: &str = 
    "CREATE INDEX IF NOT EXISTS idx_conversations_session ON conversations(session_id)";

pub const CREATE_TIMESTAMP_INDEX: &str = 
    "CREATE INDEX IF NOT EXISTS idx_conversations_timestamp ON conversations(timestamp)";

pub const CREATE_PROJECT_INDEX: &str = 
    "CREATE INDEX IF NOT EXISTS idx_conversations_project ON conversations(project_path)";

pub const CREATE_FTS_INDEX: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS conversations_fts USING fts5(
    uuid UNINDEXED,
    message_content,
    content=conversations,
    content_rowid=id
)"#;

pub const CREATE_FTS_TRIGGERS: &str = r#"
CREATE TRIGGER IF NOT EXISTS conversations_fts_insert 
AFTER INSERT ON conversations 
BEGIN
    INSERT INTO conversations_fts(rowid, uuid, message_content) 
    VALUES (new.id, new.uuid, new.message_content);
END;

CREATE TRIGGER IF NOT EXISTS conversations_fts_delete 
AFTER DELETE ON conversations 
BEGIN
    DELETE FROM conversations_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS conversations_fts_update 
AFTER UPDATE ON conversations 
BEGIN
    DELETE FROM conversations_fts WHERE rowid = old.id;
    INSERT INTO conversations_fts(rowid, uuid, message_content) 
    VALUES (new.id, new.uuid, new.message_content);
END;
"#;

pub const DROP_CONVERSATIONS_TABLE: &str = "DROP TABLE IF EXISTS conversations";
pub const DROP_FTS_TABLE: &str = "DROP TABLE IF EXISTS conversations_fts";

pub struct SchemaManager<'a> {
    connection: &'a dyn DatabaseConnection,
}

impl<'a> SchemaManager<'a> {
    pub fn new(connection: &'a dyn DatabaseConnection) -> Self {
        Self { connection }
    }

    pub fn create_schema(&self) -> Result<()> {
        if !self.connection.is_connected() {
            return Err(anyhow!("Database not connected"));
        }

        // Create main table
        self.connection.execute(CREATE_CONVERSATIONS_TABLE)?;
        
        // Create indexes
        self.connection.execute(CREATE_UUID_INDEX)?;
        self.connection.execute(CREATE_SESSION_INDEX)?;
        self.connection.execute(CREATE_TIMESTAMP_INDEX)?;
        self.connection.execute(CREATE_PROJECT_INDEX)?;
        
        Ok(())
    }

    pub fn create_fts_indexes(&self) -> Result<()> {
        if !self.connection.is_connected() {
            return Err(anyhow!("Database not connected"));
        }

        // Create FTS virtual table
        self.connection.execute(CREATE_FTS_INDEX)?;
        
        // Create FTS triggers
        self.connection.execute(CREATE_FTS_TRIGGERS)?;
        
        Ok(())
    }

    pub fn drop_schema(&self) -> Result<()> {
        if !self.connection.is_connected() {
            return Err(anyhow!("Database not connected"));
        }

        // Drop FTS table first (due to foreign key constraints)
        self.connection.execute(DROP_FTS_TABLE)?;
        
        // Drop main table
        self.connection.execute(DROP_CONVERSATIONS_TABLE)?;
        
        Ok(())
    }

    pub fn migrate_up(&self) -> Result<()> {
        self.create_schema()?;
        self.create_fts_indexes()?;
        Ok(())
    }

    pub fn migrate_down(&self) -> Result<()> {
        self.drop_schema()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_connection::MockDatabaseConnection;
    use mockall::predicate::*;

    #[test]
    fn test_create_conversations_table() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
            
        mock_conn.expect_execute()
            .with(eq(CREATE_CONVERSATIONS_TABLE))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_conn.expect_execute()
            .with(eq(CREATE_UUID_INDEX))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_conn.expect_execute()
            .with(eq(CREATE_SESSION_INDEX))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_conn.expect_execute()
            .with(eq(CREATE_TIMESTAMP_INDEX))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_conn.expect_execute()
            .with(eq(CREATE_PROJECT_INDEX))
            .times(1)
            .returning(|_| Ok(()));
        
        let schema_manager = SchemaManager::new(&mock_conn);
        let result = schema_manager.create_schema();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_schema_when_not_connected() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| false);
        
        let schema_manager = SchemaManager::new(&mock_conn);
        let result = schema_manager.create_schema();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database not connected"));
    }

    #[test]
    fn test_create_fts_indexes() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
            
        mock_conn.expect_execute()
            .with(eq(CREATE_FTS_INDEX))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_conn.expect_execute()
            .with(eq(CREATE_FTS_TRIGGERS))
            .times(1)
            .returning(|_| Ok(()));
        
        let schema_manager = SchemaManager::new(&mock_conn);
        let result = schema_manager.create_fts_indexes();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_schema() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
            
        mock_conn.expect_execute()
            .with(eq(DROP_FTS_TABLE))
            .times(1)
            .returning(|_| Ok(()));
            
        mock_conn.expect_execute()
            .with(eq(DROP_CONVERSATIONS_TABLE))
            .times(1)
            .returning(|_| Ok(()));
        
        let schema_manager = SchemaManager::new(&mock_conn);
        let result = schema_manager.drop_schema();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_up() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        // Expect is_connected to be called twice (once for create_schema, once for create_fts_indexes)
        mock_conn.expect_is_connected()
            .times(2)
            .returning(|| true);
            
        // Expect all table and index creation calls
        mock_conn.expect_execute()
            .times(7)  // 5 for create_schema + 2 for create_fts_indexes
            .returning(|_| Ok(()));
        
        let schema_manager = SchemaManager::new(&mock_conn);
        let result = schema_manager.migrate_up();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_down() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
            
        mock_conn.expect_execute()
            .times(2)  // DROP_FTS_TABLE and DROP_CONVERSATIONS_TABLE
            .returning(|_| Ok(()));
        
        let schema_manager = SchemaManager::new(&mock_conn);
        let result = schema_manager.migrate_down();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_idempotent_schema_creation() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        // Set up expectations for two consecutive migrate_up calls
        mock_conn.expect_is_connected()
            .times(4)  // 2 calls per migrate_up, 2 migrate_up calls
            .returning(|| true);
            
        mock_conn.expect_execute()
            .times(14)  // 7 calls per migrate_up, 2 migrate_up calls
            .returning(|_| Ok(()));
        
        let schema_manager = SchemaManager::new(&mock_conn);
        
        // First migration
        let result1 = schema_manager.migrate_up();
        assert!(result1.is_ok());
        
        // Second migration (should be idempotent)
        let result2 = schema_manager.migrate_up();
        assert!(result2.is_ok());
    }
}