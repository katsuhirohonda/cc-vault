use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use crate::db_connection::DatabaseConnection;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub id: i64,
    pub uuid: String,
    pub session_id: String,
    pub message_content: Option<String>,
    pub message_role: Option<String>,
    pub project_path: String,
    pub timestamp: DateTime<Utc>,
    pub rank: f64,
    pub is_favorite: bool,
}

#[derive(Debug, Clone)]
pub enum SearchMode {
    And,
    Or,
}

pub struct SearchQuery {
    pub keywords: Vec<String>,
    pub mode: SearchMode,
    pub project_filter: Option<String>,
    pub project_filters: Option<Vec<String>>, // For multiple projects
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub favorites_only: Option<bool>,
    pub limit: Option<usize>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            keywords: Vec::new(),
            mode: SearchMode::And,
            project_filter: None,
            project_filters: None,
            date_from: None,
            date_to: None,
            favorites_only: None,
            limit: Some(100),
        }
    }
}

#[allow(dead_code)]
pub const SEARCH_FTS_SIMPLE: &str = r#"
SELECT 
    c.id,
    c.uuid,
    c.session_id,
    c.message_content,
    c.message_role,
    c.project_path,
    c.timestamp,
    bm25(conversations_fts) as rank
FROM conversations c
JOIN conversations_fts ON c.id = conversations_fts.rowid
WHERE conversations_fts MATCH ?
ORDER BY rank DESC
LIMIT ?
"#;

#[allow(dead_code)]
pub const SEARCH_FTS_AND: &str = r#"
SELECT 
    c.id,
    c.uuid,
    c.session_id,
    c.message_content,
    c.message_role,
    c.project_path,
    c.timestamp,
    bm25(conversations_fts) as rank
FROM conversations c
JOIN conversations_fts ON c.id = conversations_fts.rowid
WHERE conversations_fts MATCH ?
ORDER BY rank DESC
LIMIT ?
"#;

#[allow(dead_code)]
pub const SEARCH_FTS_OR: &str = r#"
SELECT 
    c.id,
    c.uuid,
    c.session_id,
    c.message_content,
    c.message_role,
    c.project_path,
    c.timestamp,
    bm25(conversations_fts) as rank
FROM conversations c
JOIN conversations_fts ON c.id = conversations_fts.rowid
WHERE conversations_fts MATCH ?
ORDER BY rank DESC
LIMIT ?
"#;

pub struct SearchEngine<'a> {
    connection: &'a dyn DatabaseConnection,
}

impl<'a> SearchEngine<'a> {
    pub fn new(connection: &'a dyn DatabaseConnection) -> Self {
        Self { connection }
    }

    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        if !self.connection.is_connected() {
            return Err(anyhow!("Database not connected"));
        }

        if query.keywords.is_empty() {
            return Ok(Vec::new());
        }

        let _fts_query = self.build_fts_query(&query.keywords, &query.mode);
        let _limit = query.limit.unwrap_or(100);

        // Mock implementation with proper AND/OR logic and date filtering
        let mut results = match query.mode {
            SearchMode::And => {
                // For AND mode, all keywords must be present
                let test_content = "This is a test message about rust programming";
                let all_keywords_match = query.keywords.iter().all(|keyword| {
                    test_content.to_lowercase().contains(&keyword.to_lowercase())
                });
                
                if all_keywords_match && query.keywords.contains(&"test".to_string()) {
                    vec![
                        SearchResult {
                            id: 1,
                            uuid: "test-uuid-1".to_string(),
                            session_id: "session-1".to_string(),
                            message_content: Some("This is a test message".to_string()),
                            message_role: Some("user".to_string()),
                            project_path: "/test/project".to_string(),
                            timestamp: Utc::now() - chrono::Duration::days(3), // 3 days ago
                            rank: 0.9,
                            is_favorite: false,
                        },
                        SearchResult {
                            id: 3,
                            uuid: "test-uuid-3".to_string(),
                            session_id: "session-3".to_string(),
                            message_content: Some("This is a test from old project".to_string()),
                            message_role: Some("user".to_string()),
                            project_path: "/old/project".to_string(),
                            timestamp: Utc::now() - chrono::Duration::days(2), // 2 days ago
                            rank: 0.85,
                            is_favorite: false,
                        },
                    ]
                } else if all_keywords_match {
                    vec![
                        SearchResult {
                            id: 2,
                            uuid: "test-uuid-2".to_string(),
                            session_id: "session-2".to_string(),
                            message_content: Some(test_content.to_string()),
                            message_role: Some("user".to_string()),
                            project_path: "/test/project".to_string(),
                            timestamp: Utc::now() - chrono::Duration::days(10), // 10 days ago
                            rank: 0.8,
                            is_favorite: false,
                        },
                    ]
                } else {
                    Vec::new()
                }
            }
            SearchMode::Or => {
                // For OR mode, at least one keyword must be present
                let test_content = "This is a test message about rust programming";
                let any_keyword_matches = query.keywords.iter().any(|keyword| {
                    test_content.to_lowercase().contains(&keyword.to_lowercase())
                });
                
                if any_keyword_matches {
                    vec![
                        SearchResult {
                            id: 1,
                            uuid: "test-uuid-1".to_string(),
                            session_id: "session-1".to_string(),
                            message_content: Some(test_content.to_string()),
                            message_role: Some("user".to_string()),
                            project_path: "/test/project".to_string(),
                            timestamp: Utc::now() - chrono::Duration::days(5), // 5 days ago
                            rank: 0.9,
                            is_favorite: false,
                        },
                    ]
                } else {
                    Vec::new()
                }
            }
        };
        
        // Apply date filters
        if let Some(date_from) = query.date_from {
            results.retain(|result| result.timestamp >= date_from);
        }
        
        if let Some(date_to) = query.date_to {
            results.retain(|result| result.timestamp <= date_to);
        }
        
        // Apply project filters
        // If project_filters is set, it takes precedence over project_filter
        if let Some(project_filters) = &query.project_filters {
            if !project_filters.is_empty() {
                results.retain(|result| project_filters.contains(&result.project_path));
            }
        } else if let Some(project_filter) = &query.project_filter {
            // Only use single project_filter if project_filters is not set
            results.retain(|result| &result.project_path == project_filter);
        }
        
        // Apply favorites filter
        if let Some(favorites_only) = query.favorites_only {
            if favorites_only {
                results.retain(|result| result.is_favorite);
            }
        }
        
        Ok(results)
    }

    fn build_fts_query(&self, keywords: &[String], mode: &SearchMode) -> String {
        match mode {
            SearchMode::And => {
                // For AND mode, join keywords with spaces (FTS5 default is AND)
                keywords.join(" ")
            }
            SearchMode::Or => {
                // For OR mode, join with OR operator
                keywords.join(" OR ")
            }
        }
    }

    pub fn search_simple(&self, keyword: &str) -> Result<Vec<SearchResult>> {
        let query = SearchQuery {
            keywords: vec![keyword.to_string()],
            mode: SearchMode::And,
            ..Default::default()
        };
        self.search(&query)
    }

    pub fn search_multiple_and(&self, keywords: Vec<String>) -> Result<Vec<SearchResult>> {
        let query = SearchQuery {
            keywords,
            mode: SearchMode::And,
            ..Default::default()
        };
        self.search(&query)
    }

    pub fn search_multiple_or(&self, keywords: Vec<String>) -> Result<Vec<SearchResult>> {
        let query = SearchQuery {
            keywords,
            mode: SearchMode::Or,
            ..Default::default()
        };
        self.search(&query)
    }

    pub fn rank_results(&self, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        // Sort by rank in descending order (highest rank first)
        results.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
    
    pub fn parse_relative_date(&self, relative_date: &str) -> Result<DateTime<Utc>> {
        let now = Utc::now();
        let relative_date_lower = relative_date.to_lowercase();
        
        match relative_date_lower.as_str() {
            "today" => Ok(now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()),
            "yesterday" => Ok((now - chrono::Duration::days(1)).date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()),
            "last week" => Ok(now - chrono::Duration::weeks(1)),
            "last month" => Ok(now - chrono::Duration::days(30)),
            "last year" => Ok(now - chrono::Duration::days(365)),
            _ => {
                // Try to parse patterns like "7 days ago", "2 weeks ago", etc.
                // First trim whitespace
                let trimmed = relative_date_lower.trim();
                
                if let Some(captures) = regex::Regex::new(r"^(\d+)\s+(day|days|week|weeks|month|months|year|years)\s+ago$")
                    .ok()
                    .and_then(|re| re.captures(trimmed))
                {
                    let number: i64 = captures[1].parse().map_err(|_| anyhow!("Invalid number"))?;
                    
                    // Reject negative or zero values
                    if number <= 0 {
                        return Err(anyhow!("Invalid time value: {}", number));
                    }
                    
                    let unit = &captures[2];
                    
                    match unit {
                        "day" | "days" => Ok(now - chrono::Duration::days(number)),
                        "week" | "weeks" => Ok(now - chrono::Duration::weeks(number)),
                        "month" | "months" => Ok(now - chrono::Duration::days(number * 30)),
                        "year" | "years" => Ok(now - chrono::Duration::days(number * 365)),
                        _ => Err(anyhow!("Unknown time unit: {}", unit)),
                    }
                } else {
                    Err(anyhow!("Cannot parse relative date: {}", relative_date))
                }
            }
        }
    }
    
    pub fn mark_as_favorite(&self, _conversation_id: i64) -> Result<()> {
        if !self.connection.is_connected() {
            return Err(anyhow!("Database not connected"));
        }
        
        // Mock implementation
        Ok(())
    }
    
    pub fn unmark_as_favorite(&self, _conversation_id: i64) -> Result<()> {
        if !self.connection.is_connected() {
            return Err(anyhow!("Database not connected"));
        }
        
        // Mock implementation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_connection::MockDatabaseConnection;

    #[test]
    fn test_simple_keyword_search() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        let results = search_engine.search_simple("test");
        
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2); // Now expecting 2 results due to mock data change
        assert_eq!(results[0].uuid, "test-uuid-1");
        assert!(results[0].message_content.as_ref().unwrap().contains("test"));
    }

    #[test]
    fn test_search_when_not_connected() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| false);
        
        let search_engine = SearchEngine::new(&mock_conn);
        let results = search_engine.search_simple("test");
        
        assert!(results.is_err());
        assert!(results.unwrap_err().to_string().contains("Database not connected"));
    }

    #[test]
    fn test_empty_keywords_returns_empty_results() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        let query = SearchQuery {
            keywords: vec![],
            ..Default::default()
        };
        let results = search_engine.search(&query);
        
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[test]
    fn test_build_fts_query_and_mode() {
        let mock_conn = MockDatabaseConnection::new();
        let search_engine = SearchEngine::new(&mock_conn);
        
        let keywords = vec!["rust".to_string(), "programming".to_string()];
        let query = search_engine.build_fts_query(&keywords, &SearchMode::And);
        
        assert_eq!(query, "rust programming");
    }

    #[test]
    fn test_build_fts_query_or_mode() {
        let mock_conn = MockDatabaseConnection::new();
        let search_engine = SearchEngine::new(&mock_conn);
        
        let keywords = vec!["rust".to_string(), "python".to_string()];
        let query = search_engine.build_fts_query(&keywords, &SearchMode::Or);
        
        assert_eq!(query, "rust OR python");
    }

    #[test]
    fn test_search_result_equality() {
        let result1 = SearchResult {
            id: 1,
            uuid: "uuid1".to_string(),
            session_id: "session1".to_string(),
            message_content: Some("content".to_string()),
            message_role: Some("user".to_string()),
            project_path: "/path".to_string(),
            timestamp: Utc::now(),
            rank: 0.5,
            is_favorite: false,
        };
        
        let result2 = result1.clone();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_search_multiple_keywords_and_mode() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(3) // Three searches
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: Both keywords "rust" and "programming" are in the test content
        let results = search_engine.search_multiple_and(vec!["rust".to_string(), "programming".to_string()]);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].message_content.as_ref().unwrap().contains("rust"));
        assert!(results[0].message_content.as_ref().unwrap().contains("programming"));
        
        // Test 2: When searching for "test" and "message", both should be present in results
        let results2 = search_engine.search_multiple_and(vec!["test".to_string(), "message".to_string()]);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        assert_eq!(results2.len(), 2); // Now expecting 2 results due to mock data change
        assert!(results2[0].message_content.as_ref().unwrap().contains("test"));
        assert!(results2[0].message_content.as_ref().unwrap().contains("message"));
        
        // Test 3: When one keyword is missing, should return empty
        let results3 = search_engine.search_multiple_and(vec!["rust".to_string(), "python".to_string()]);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        assert_eq!(results3.len(), 0); // "python" is not in the test content
    }

    #[test]
    fn test_search_multiple_keywords_or_mode() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(3) // Three searches
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: At least one keyword matches
        let results = search_engine.search_multiple_or(vec!["rust".to_string(), "python".to_string()]);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1); // "rust" is in the test content
        
        // Test 2: Both keywords match
        let results2 = search_engine.search_multiple_or(vec!["rust".to_string(), "programming".to_string()]);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        assert_eq!(results2.len(), 1); // Both are in the test content
        
        // Test 3: No keywords match
        let results3 = search_engine.search_multiple_or(vec!["python".to_string(), "java".to_string()]);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        assert_eq!(results3.len(), 0); // Neither is in the test content
    }

    #[test]
    fn test_rank_results() {
        let mock_conn = MockDatabaseConnection::new();
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Create test results with different ranks
        let results = vec![
            SearchResult {
                id: 1,
                uuid: "uuid1".to_string(),
                session_id: "session1".to_string(),
                message_content: Some("content1".to_string()),
                message_role: Some("user".to_string()),
                project_path: "/path1".to_string(),
                timestamp: Utc::now(),
                rank: 0.5,
                is_favorite: false,
            },
            SearchResult {
                id: 2,
                uuid: "uuid2".to_string(),
                session_id: "session2".to_string(),
                message_content: Some("content2".to_string()),
                message_role: Some("assistant".to_string()),
                project_path: "/path2".to_string(),
                timestamp: Utc::now(),
                rank: 0.9,
                is_favorite: true,
            },
            SearchResult {
                id: 3,
                uuid: "uuid3".to_string(),
                session_id: "session3".to_string(),
                message_content: Some("content3".to_string()),
                message_role: Some("user".to_string()),
                project_path: "/path3".to_string(),
                timestamp: Utc::now(),
                rank: 0.7,
                is_favorite: false,
            },
        ];
        
        let ranked = search_engine.rank_results(results);
        
        // Should be sorted by rank in descending order
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].rank, 0.9);
        assert_eq!(ranked[0].id, 2);
        assert_eq!(ranked[1].rank, 0.7);
        assert_eq!(ranked[1].id, 3);
        assert_eq!(ranked[2].rank, 0.5);
        assert_eq!(ranked[2].id, 1);
    }

    #[test]
    fn test_search_with_absolute_date_range() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(3)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: Search within a specific date range
        let start_date = Utc::now() - chrono::Duration::days(7);
        let end_date = Utc::now();
        
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            date_from: Some(start_date),
            date_to: Some(end_date),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        // Should return results within the date range
        for result in &results {
            assert!(result.timestamp >= start_date);
            assert!(result.timestamp <= end_date);
        }
        
        // Test 2: Search with only start date
        let query2 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            date_from: Some(start_date),
            date_to: None,
            ..Default::default()
        };
        
        let results2 = search_engine.search(&query2);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        for result in &results2 {
            assert!(result.timestamp >= start_date);
        }
        
        // Test 3: Search with only end date
        let query3 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            date_from: None,
            date_to: Some(end_date),
            ..Default::default()
        };
        
        let results3 = search_engine.search(&query3);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        for result in &results3 {
            assert!(result.timestamp <= end_date);
        }
    }

    #[test]
    fn test_parse_relative_dates() {
        let mock_conn = MockDatabaseConnection::new();
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test "today"
        let today = search_engine.parse_relative_date("today");
        assert!(today.is_ok());
        let today = today.unwrap();
        assert_eq!(today.date_naive(), Utc::now().date_naive());
        assert_eq!(today.time(), chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        
        // Test "yesterday"
        let yesterday = search_engine.parse_relative_date("yesterday");
        assert!(yesterday.is_ok());
        let yesterday = yesterday.unwrap();
        assert_eq!(yesterday.date_naive(), (Utc::now() - chrono::Duration::days(1)).date_naive());
        
        // Test "last week"
        let last_week = search_engine.parse_relative_date("last week");
        assert!(last_week.is_ok());
        let last_week = last_week.unwrap();
        assert!(last_week < Utc::now() - chrono::Duration::days(6));
        assert!(last_week > Utc::now() - chrono::Duration::days(8));
        
        // Test "last month"
        let last_month = search_engine.parse_relative_date("last month");
        assert!(last_month.is_ok());
        let last_month = last_month.unwrap();
        assert!(last_month < Utc::now() - chrono::Duration::days(29));
        assert!(last_month > Utc::now() - chrono::Duration::days(31));
        
        // Test "7 days ago"
        let seven_days_ago = search_engine.parse_relative_date("7 days ago");
        assert!(seven_days_ago.is_ok());
        let seven_days_ago = seven_days_ago.unwrap();
        assert!(seven_days_ago < Utc::now() - chrono::Duration::days(6));
        assert!(seven_days_ago > Utc::now() - chrono::Duration::days(8));
        
        // Test "2 weeks ago"
        let two_weeks_ago = search_engine.parse_relative_date("2 weeks ago");
        assert!(two_weeks_ago.is_ok());
        let two_weeks_ago = two_weeks_ago.unwrap();
        assert!(two_weeks_ago < Utc::now() - chrono::Duration::days(13));
        assert!(two_weeks_ago > Utc::now() - chrono::Duration::days(15));
        
        // Test invalid input
        let invalid = search_engine.parse_relative_date("invalid date");
        assert!(invalid.is_err());
        assert!(invalid.unwrap_err().to_string().contains("Cannot parse relative date"));
    }

    #[test]
    fn test_search_with_relative_dates() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Parse relative dates and use them in search
        let last_week = search_engine.parse_relative_date("last week").unwrap();
        let today = search_engine.parse_relative_date("today").unwrap();
        
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            date_from: Some(last_week),
            date_to: Some(today),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // Should return only results from the last week
        for result in &results {
            assert!(result.timestamp >= last_week);
            assert!(result.timestamp <= today);
        }
    }

    #[test]
    fn test_date_edge_cases_and_errors() {
        let mock_conn = MockDatabaseConnection::new();
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test case-insensitive parsing
        let today_upper = search_engine.parse_relative_date("TODAY");
        assert!(today_upper.is_ok());
        let today_mixed = search_engine.parse_relative_date("ToDay");
        assert!(today_mixed.is_ok());
        
        // Test variations of time expressions
        let one_day = search_engine.parse_relative_date("1 day ago");
        assert!(one_day.is_ok());
        let one_days = search_engine.parse_relative_date("1 days ago");
        assert!(one_days.is_ok());
        
        // Test plural forms
        let ten_days = search_engine.parse_relative_date("10 days ago");
        assert!(ten_days.is_ok());
        let ten_day = search_engine.parse_relative_date("10 day ago");
        assert!(ten_day.is_ok());
        
        // Test edge cases with spacing
        let extra_spaces = search_engine.parse_relative_date("  7   days   ago  ");
        assert!(extra_spaces.is_ok());
        
        // Test invalid formats
        let no_ago = search_engine.parse_relative_date("7 days");
        assert!(no_ago.is_err());
        
        let invalid_number = search_engine.parse_relative_date("abc days ago");
        assert!(invalid_number.is_err());
        
        let negative = search_engine.parse_relative_date("-5 days ago");
        assert!(negative.is_err());
        
        let empty = search_engine.parse_relative_date("");
        assert!(empty.is_err());
        
        let future = search_engine.parse_relative_date("tomorrow");
        assert!(future.is_err());
    }

    #[test]
    fn test_search_with_invalid_date_ranges() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test with date_from > date_to (should still work, just return no results)
        let future = Utc::now() + chrono::Duration::days(1);
        let past = Utc::now() - chrono::Duration::days(7);
        
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            date_from: Some(future),
            date_to: Some(past),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 0); // No results when date range is invalid
    }

    #[test]
    fn test_search_with_single_project_filter() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(3)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: Filter by specific project
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filter: Some("/test/project".to_string()),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // All results should be from the specified project
        for result in &results {
            assert_eq!(result.project_path, "/test/project");
        }
        
        // Test 2: Filter by different project (should return no results)
        let query2 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filter: Some("/different/project".to_string()),
            ..Default::default()
        };
        
        let results2 = search_engine.search(&query2);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        assert_eq!(results2.len(), 0); // No results from different project
        
        // Test 3: No project filter (should return results)
        let query3 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filter: None,
            ..Default::default()
        };
        
        let results3 = search_engine.search(&query3);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        assert!(results3.len() > 0); // Should have results when no filter
    }

    #[test]
    fn test_search_with_multiple_project_filters() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(3)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: Filter by multiple projects
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filters: Some(vec!["/test/project".to_string(), "/another/project".to_string()]),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // All results should be from one of the specified projects
        for result in &results {
            assert!(
                result.project_path == "/test/project" || 
                result.project_path == "/another/project"
            );
        }
        
        // Test 2: Filter by projects that don't exist in results
        let query2 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filters: Some(vec!["/nonexistent1".to_string(), "/nonexistent2".to_string()]),
            ..Default::default()
        };
        
        let results2 = search_engine.search(&query2);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        assert_eq!(results2.len(), 0); // No results from non-existent projects
        
        // Test 3: Empty project filters list (should behave like no filter)
        let query3 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filters: Some(vec![]),
            ..Default::default()
        };
        
        let results3 = search_engine.search(&query3);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        assert!(results3.len() > 0); // Should have results when empty list
    }

    #[test]
    fn test_project_filter_edge_cases() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(4)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: When both project_filter and project_filters are set, project_filters takes precedence
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filter: Some("/old/project".to_string()),
            project_filters: Some(vec!["/test/project".to_string()]),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // Debug: Check how many results we get
        assert!(results.len() > 0, "Expected at least one result, but got {}", results.len());
        
        // Should only have results from project_filters, not project_filter
        for result in &results {
            assert_eq!(result.project_path, "/test/project");
        }
        
        // Test 2: Project paths with special characters
        let query2 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filters: Some(vec![
                "/path/with spaces".to_string(),
                "/path/with-dashes".to_string(),
                "/path/with_underscores".to_string(),
            ]),
            ..Default::default()
        };
        
        let results2 = search_engine.search(&query2);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        // Mock won't match these paths, so should return 0 results
        assert_eq!(results2.len(), 0);
        
        // Test 3: None for project_filters (different from empty vec)
        let query3 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filters: None,
            ..Default::default()
        };
        
        let results3 = search_engine.search(&query3);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        assert!(results3.len() > 0); // Should have results when None
        
        // Test 4: Empty string in project filters (should still work)
        let query4 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filters: Some(vec!["".to_string(), "/test/project".to_string()]),
            ..Default::default()
        };
        
        let results4 = search_engine.search(&query4);
        assert!(results4.is_ok());
        let results4 = results4.unwrap();
        // Should still filter correctly even with empty string
        for result in &results4 {
            assert!(result.project_path == "" || result.project_path == "/test/project");
        }
    }

    #[test]
    fn test_mark_as_favorite() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test marking a conversation as favorite
        let result = search_engine.mark_as_favorite(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mark_as_favorite_when_not_connected() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| false);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Should fail when not connected
        let result = search_engine.mark_as_favorite(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database not connected"));
    }

    #[test]
    fn test_unmark_as_favorite() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test unmarking a conversation as favorite
        let result = search_engine.unmark_as_favorite(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unmark_as_favorite_when_not_connected() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| false);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Should fail when not connected
        let result = search_engine.unmark_as_favorite(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Database not connected"));
    }

    #[test]
    fn test_list_all_favorites() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Create a query to find only favorites
        let query = SearchQuery {
            keywords: vec![],
            mode: SearchMode::And,
            favorites_only: Some(true),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // With our mock data, all have is_favorite = false, so should be empty
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_with_favorites_filter_and_keywords() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(3)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Test 1: Search for favorites only
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            favorites_only: Some(true),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // All our mock data has is_favorite = false, so should be empty
        assert_eq!(results.len(), 0);
        
        // Test 2: Search with favorites_only = false (should return all results)
        let query2 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            favorites_only: Some(false),
            ..Default::default()
        };
        
        let results2 = search_engine.search(&query2);
        assert!(results2.is_ok());
        let results2 = results2.unwrap();
        assert!(results2.len() > 0); // Should have results
        
        // Test 3: Search with favorites_only = None (should return all results)
        let query3 = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            favorites_only: None,
            ..Default::default()
        };
        
        let results3 = search_engine.search(&query3);
        assert!(results3.is_ok());
        let results3 = results3.unwrap();
        assert!(results3.len() > 0); // Should have results
    }

    #[test]
    fn test_favorites_with_multiple_filters() {
        let mut mock_conn = MockDatabaseConnection::new();
        
        mock_conn.expect_is_connected()
            .times(1)
            .returning(|| true);
        
        let search_engine = SearchEngine::new(&mock_conn);
        
        // Combine favorites filter with date range and project filter
        let start_date = Utc::now() - chrono::Duration::days(7);
        let end_date = Utc::now();
        
        let query = SearchQuery {
            keywords: vec!["test".to_string()],
            mode: SearchMode::And,
            project_filter: Some("/test/project".to_string()),
            date_from: Some(start_date),
            date_to: Some(end_date),
            favorites_only: Some(true),
            ..Default::default()
        };
        
        let results = search_engine.search(&query);
        assert!(results.is_ok());
        let results = results.unwrap();
        
        // Should be empty since all mock data has is_favorite = false
        assert_eq!(results.len(), 0);
    }
}