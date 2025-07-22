use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeMessage {
    pub parent_uuid: Option<String>,
    pub is_sidechain: bool,
    pub user_type: String,
    pub cwd: String,
    pub session_id: String,
    pub version: String,
    pub git_branch: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
    pub message: MessageContent,
    pub uuid: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageContent {
    pub role: Option<String>,
    pub content: Option<Value>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub model: Option<String>,
}

pub struct JsonlParser;

#[allow(dead_code)]
impl JsonlParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_single_message(&self, json_str: &str) -> Result<ClaudeMessage> {
        serde_json::from_str(json_str)
            .context("Failed to parse JSON message")
    }

    pub fn parse_multiple_messages(&self, jsonl_content: &str) -> Result<Vec<ClaudeMessage>> {
        let mut messages = Vec::new();
        
        for line in jsonl_content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let message = self.parse_single_message(trimmed)?;
                messages.push(message);
            }
        }
        
        Ok(messages)
    }

    pub fn parse_multiple_messages_skip_errors(&self, jsonl_content: &str) -> Vec<(usize, Result<ClaudeMessage>)> {
        let mut results = Vec::new();
        
        for (line_num, line) in jsonl_content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let result = self.parse_single_message(trimmed);
                results.push((line_num + 1, result));
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_single_json_message() {
        let json_str = r#"{"parentUuid":null,"isSidechain":false,"userType":"external","cwd":"/Users/honda/dev/cc-vault","sessionId":"28035087-fe7b-4b2e-843a-ce802161e23c","version":"1.0.56","gitBranch":"main","type":"user","message":{"role":"user","content":"TODO.md の続きをやって2からだよ"},"uuid":"fd9e5f80-43b8-4825-ab31-f531d688d30b","timestamp":"2025-07-21T12:48:30.283Z"}"#;
        
        let parser = JsonlParser::new();
        let result = parser.parse_single_message(json_str);
        
        assert!(result.is_ok());
        let message = result.unwrap();
        
        assert_eq!(message.parent_uuid, None);
        assert_eq!(message.is_sidechain, false);
        assert_eq!(message.user_type, "external");
        assert_eq!(message.cwd, "/Users/honda/dev/cc-vault");
        assert_eq!(message.session_id, "28035087-fe7b-4b2e-843a-ce802161e23c");
        assert_eq!(message.version, "1.0.56");
        assert_eq!(message.git_branch, Some("main".to_string()));
        assert_eq!(message.message_type, "user");
        assert_eq!(message.uuid, "fd9e5f80-43b8-4825-ab31-f531d688d30b");
        
        assert_eq!(message.message.role, Some("user".to_string()));
    }

    #[test]
    fn test_parse_multiple_jsonl_lines() {
        let jsonl_content = r#"{"parentUuid":null,"isSidechain":false,"userType":"external","cwd":"/Users/honda/dev/cc-vault","sessionId":"session1","version":"1.0.56","gitBranch":"main","type":"user","message":{"role":"user","content":"First message"},"uuid":"uuid1","timestamp":"2025-07-21T12:48:30.283Z"}
{"parentUuid":"uuid1","isSidechain":false,"userType":"external","cwd":"/Users/honda/dev/cc-vault","sessionId":"session1","version":"1.0.56","gitBranch":"main","type":"assistant","message":{"role":"assistant","content":"Second message"},"uuid":"uuid2","timestamp":"2025-07-21T12:48:31.283Z"}
{"parentUuid":"uuid2","isSidechain":false,"userType":"external","cwd":"/Users/honda/dev/cc-vault","sessionId":"session1","version":"1.0.56","gitBranch":"main","type":"user","message":{"role":"user","content":"Third message"},"uuid":"uuid3","timestamp":"2025-07-21T12:48:32.283Z"}"#;
        
        let parser = JsonlParser::new();
        let result = parser.parse_multiple_messages(jsonl_content);
        
        assert!(result.is_ok());
        let messages = result.unwrap();
        
        assert_eq!(messages.len(), 3);
        
        assert_eq!(messages[0].uuid, "uuid1");
        assert_eq!(messages[0].message_type, "user");
        assert_eq!(messages[0].message.role, Some("user".to_string()));
        
        assert_eq!(messages[1].uuid, "uuid2");
        assert_eq!(messages[1].message_type, "assistant");
        assert_eq!(messages[1].message.role, Some("assistant".to_string()));
        assert_eq!(messages[1].parent_uuid, Some("uuid1".to_string()));
        
        assert_eq!(messages[2].uuid, "uuid3");
        assert_eq!(messages[2].message_type, "user");
        assert_eq!(messages[2].parent_uuid, Some("uuid2".to_string()));
    }

    #[test]
    fn test_handle_invalid_json() {
        let invalid_json = r#"{"invalid": json, not parseable}"#;
        
        let parser = JsonlParser::new();
        let result = parser.parse_single_message(invalid_json);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse JSON message"));
    }

    #[test]
    fn test_parse_with_invalid_lines_skip_errors() {
        let mixed_content = r#"{"parentUuid":null,"isSidechain":false,"userType":"external","cwd":"/test","sessionId":"session1","version":"1.0","gitBranch":"main","type":"user","message":{"role":"user","content":"Valid"},"uuid":"uuid1","timestamp":"2025-07-21T12:48:30.283Z"}
{invalid json line}
{"parentUuid":"uuid1","isSidechain":false,"userType":"external","cwd":"/test","sessionId":"session1","version":"1.0","gitBranch":"main","type":"user","message":{"role":"user","content":"Also valid"},"uuid":"uuid2","timestamp":"2025-07-21T12:48:31.283Z"}"#;
        
        let parser = JsonlParser::new();
        let result = parser.parse_multiple_messages(mixed_content);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_skip_errors_method() {
        let mixed_content = r#"{"parentUuid":null,"isSidechain":false,"userType":"external","cwd":"/test","sessionId":"session1","version":"1.0","gitBranch":"main","type":"user","message":{"role":"user","content":"Valid"},"uuid":"uuid1","timestamp":"2025-07-21T12:48:30.283Z"}
{invalid json line}
{"parentUuid":"uuid1","isSidechain":false,"userType":"external","cwd":"/test","sessionId":"session1","version":"1.0","gitBranch":"main","type":"user","message":{"role":"user","content":"Also valid"},"uuid":"uuid2","timestamp":"2025-07-21T12:48:31.283Z"}"#;
        
        let parser = JsonlParser::new();
        let results = parser.parse_multiple_messages_skip_errors(mixed_content);
        
        assert_eq!(results.len(), 3);
        
        assert!(results[0].1.is_ok());
        assert_eq!(results[0].0, 1);
        assert_eq!(results[0].1.as_ref().unwrap().uuid, "uuid1");
        
        assert!(results[1].1.is_err());
        assert_eq!(results[1].0, 2);
        
        assert!(results[2].1.is_ok());
        assert_eq!(results[2].0, 3);
        assert_eq!(results[2].1.as_ref().unwrap().uuid, "uuid2");
    }

    #[test]
    fn test_handle_empty_lines_and_whitespace() {
        let content_with_empty = r#"
{"parentUuid":null,"isSidechain":false,"userType":"external","cwd":"/test","sessionId":"session1","version":"1.0","gitBranch":"main","type":"user","message":{"role":"user","content":"First"},"uuid":"uuid1","timestamp":"2025-07-21T12:48:30.283Z"}

        

{"parentUuid":"uuid1","isSidechain":false,"userType":"external","cwd":"/test","sessionId":"session1","version":"1.0","gitBranch":"main","type":"user","message":{"role":"user","content":"Second"},"uuid":"uuid2","timestamp":"2025-07-21T12:48:31.283Z"}
    
"#;
        
        let parser = JsonlParser::new();
        let result = parser.parse_multiple_messages(content_with_empty);
        
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].uuid, "uuid1");
        assert_eq!(messages[1].uuid, "uuid2");
    }
}