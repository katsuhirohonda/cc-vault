mod claude_reader;
mod jsonl_parser;
mod db_connection;
mod db_schema;
mod data_importer;
mod search;

use anyhow::Result;
use claude_reader::ClaudeReader;
use jsonl_parser::JsonlParser;
use std::fs;

fn main() -> Result<()> {
    let reader = ClaudeReader::new()?;
    let parser = JsonlParser::new();
    
    if reader.check_directory_exists() {
        println!("Claude projects directory found!");
        
        let project_dirs = reader.list_project_directories()?;
        println!("Found {} project directories", project_dirs.len());
        
        let jsonl_files = reader.find_jsonl_files()?;
        println!("Found {} JSONL files", jsonl_files.len());
        
        for file in jsonl_files.iter().take(1) {
            if let Some(project_name) = reader.get_project_name_from_path(file) {
                println!("\n\x1b[1mSample from project: {}\x1b[0m", project_name);
                println!("File: {}", file.display());
                
                let content = fs::read_to_string(file)?;
                let lines: Vec<&str> = content.lines().take(3).collect();
                
                println!("First {} messages:", lines.len());
                for (i, line) in lines.iter().enumerate() {
                    match parser.parse_single_message(line) {
                        Ok(msg) => {
                            println!("  {}. [{}] {}: {}",
                                i + 1,
                                msg.message_type,
                                msg.message.role.as_deref().unwrap_or("unknown"),
                                msg.uuid
                            );
                        }
                        Err(e) => {
                            println!("  {}. Error parsing: {}", i + 1, e);
                        }
                    }
                }
            }
        }
    } else {
        println!("Claude projects directory not found at ~/.claude/projects/");
    }
    
    Ok(())
}
