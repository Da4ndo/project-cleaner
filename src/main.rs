use serde::Deserialize;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

mod cleaner;

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    dir: String,
    backup_dir: String,
    file_patterns: Vec<String>,
    dir_patterns: Vec<String>,
    exception_files: Vec<String>,
    exception_dirs: Vec<String>,
    env: String,
}

#[tokio::main]
async fn main() {
    // Load configuration
    let mut file = match File::open("clean.config.json").await {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open config file: {}", e);
            return;
        }
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            return;
        }
    };

    let config: Config = match serde_json::from_str(&contents) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to parse config file: {}", e);
            return;
        }
    };

    let backup_dir = Path::new(&config.backup_dir);

    // Create backup directory if it doesn't exist
    match tokio::fs::create_dir_all(&backup_dir).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to create backup directory: {}", e);
            return;
        }
    };

    println!("Starting folder cleaning process...");
    let folder_cleaner = cleaner::folder::Folder::new(config.clone());
    match folder_cleaner.clean().await {
        Ok(_) => {
            println!("Folder cleaning completed successfully.");
        }
        Err(e) => {
            eprintln!("Failed to clean folders: {}", e);
        }
    }

    println!("Starting file cleaning process...");
    let file_cleaner = cleaner::file::File::new(config.clone());
    match file_cleaner.clean().await {
        Ok(_) => {
            println!("File cleaning completed successfully.");
        }
        Err(e) => {
            eprintln!("Failed to clean files: {}", e);
        }
    }
}