use anyhow::{Context, Result};
use dirs::home_dir;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ClaudeReader {
    claude_projects_path: PathBuf,
}

impl ClaudeReader {
    pub fn new() -> Result<Self> {
        let home = home_dir().context("Failed to get home directory")?;
        let claude_projects_path = home.join(".claude").join("projects");
        
        Ok(Self {
            claude_projects_path,
        })
    }

    pub fn check_directory_exists(&self) -> bool {
        self.claude_projects_path.exists() && self.claude_projects_path.is_dir()
    }

    pub fn list_project_directories(&self) -> Result<Vec<PathBuf>> {
        if !self.check_directory_exists() {
            return Ok(Vec::new());
        }

        let mut project_dirs = Vec::new();
        
        for entry in std::fs::read_dir(&self.claude_projects_path)
            .context("Failed to read Claude projects directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_dir() {
                project_dirs.push(path);
            }
        }
        
        Ok(project_dirs)
    }

    pub fn find_jsonl_files(&self) -> Result<Vec<PathBuf>> {
        if !self.check_directory_exists() {
            return Ok(Vec::new());
        }

        let mut jsonl_files = Vec::new();
        
        for entry in WalkDir::new(&self.claude_projects_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                jsonl_files.push(path.to_path_buf());
            }
        }
        
        Ok(jsonl_files)
    }

    pub fn get_project_name_from_path(&self, jsonl_path: &Path) -> Option<String> {
        jsonl_path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_check_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let claude_path = temp_dir.path().join(".claude").join("projects");
        fs::create_dir_all(&claude_path).unwrap();
        
        let reader = ClaudeReader {
            claude_projects_path: claude_path.clone(),
        };
        
        assert!(reader.check_directory_exists());
        
        fs::remove_dir_all(&claude_path).unwrap();
        assert!(!reader.check_directory_exists());
    }

    #[test]
    fn test_list_project_directories() {
        let temp_dir = TempDir::new().unwrap();
        let claude_path = temp_dir.path().join(".claude").join("projects");
        fs::create_dir_all(&claude_path).unwrap();
        
        fs::create_dir(&claude_path.join("project1")).unwrap();
        fs::create_dir(&claude_path.join("project2")).unwrap();
        fs::File::create(&claude_path.join("not_a_dir.txt")).unwrap();
        
        let reader = ClaudeReader {
            claude_projects_path: claude_path.clone(),
        };
        
        let dirs = reader.list_project_directories().unwrap();
        assert_eq!(dirs.len(), 2);
    }

    #[test]
    fn test_find_jsonl_files() {
        let temp_dir = TempDir::new().unwrap();
        let claude_path = temp_dir.path().join(".claude").join("projects");
        let project_path = claude_path.join("test_project");
        fs::create_dir_all(&project_path).unwrap();
        
        fs::File::create(&project_path.join("conversation1.jsonl")).unwrap();
        fs::File::create(&project_path.join("conversation2.jsonl")).unwrap();
        fs::File::create(&project_path.join("not_jsonl.txt")).unwrap();
        
        let reader = ClaudeReader {
            claude_projects_path: claude_path.clone(),
        };
        
        let files = reader.find_jsonl_files().unwrap();
        assert_eq!(files.len(), 2);
        
        for file in &files {
            assert!(file.extension().unwrap() == "jsonl");
        }
    }

    #[test]
    fn test_get_project_name_from_path() {
        let reader = ClaudeReader::new().unwrap();
        let path = PathBuf::from("/home/user/.claude/projects/my-project/conversation.jsonl");
        
        assert_eq!(
            reader.get_project_name_from_path(&path),
            Some("my-project".to_string())
        );
    }
}