mod claude_reader;
mod jsonl_parser;

use anyhow::Result;
use claude_reader::ClaudeReader;

fn main() -> Result<()> {
    let reader = ClaudeReader::new()?;
    
    if reader.check_directory_exists() {
        println!("Claude projects directory found!");
        
        let project_dirs = reader.list_project_directories()?;
        println!("Found {} project directories", project_dirs.len());
        
        let jsonl_files = reader.find_jsonl_files()?;
        println!("Found {} JSONL files", jsonl_files.len());
        
        for file in jsonl_files.iter().take(5) {
            if let Some(project_name) = reader.get_project_name_from_path(file) {
                println!("  - {} (project: {})", file.display(), project_name);
            }
        }
    } else {
        println!("Claude projects directory not found at ~/.claude/projects/");
    }
    
    Ok(())
}
