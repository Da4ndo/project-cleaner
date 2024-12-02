use serde::Deserialize;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use colored::*;
use clap::{Command, Arg};
use std::env;

mod cleaner;
mod restore;

#[derive(Deserialize, Clone)]
pub struct Config {
    dir: String,
    backup: BackupConfig,
    file_patterns: Vec<String>,
    dir_patterns: Vec<String>,
    exception_files: Vec<String>,
    exception_dirs: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct BackupConfig {
    enabled: bool,
    dir: String,
    versioning: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Command::new("project-cleaner")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Da4ndo <contact@da4ndo.com>")
        .about("A tool to clean up project directories with configurable patterns and backup options")
        .arg(Arg::new("restore")
            .short('r')
            .long("restore")
            .action(clap::ArgAction::SetTrue)
            .help("Restore files from backup"));

    let matches = app.get_matches();

    // Load configuration
    let config = load_config().await?;

    if matches.get_flag("restore") {
        println!("{}", "Starting restore process...".bright_cyan());
        let restorer = restore::Restorer::new(config);
        restorer.restore().await?;
        println!("{}", "Restore process completed successfully.".bright_green());
    } else {
        // Print configuration
        println!("{}", "📝 Configuration:".bright_blue().bold());
        println!("  {} {}: {}", "→".bright_black(), "Project directory".yellow(), config.dir.yellow());
        println!("  {} {}", "→".bright_black(), "Backup settings:".bright_blue());
        println!("    - {}: {}", "Enabled".bright_blue(), config.backup.enabled.to_string().bright_white());
        println!("    - {}: {}", "Directory".bright_blue(), config.backup.dir.bright_white());
        println!("    - {}: {}", "Versioning".bright_blue(), config.backup.versioning.to_string().bright_white());
        
        println!("\n  {} {}", "→".bright_black(), "File patterns to clean:".bright_blue());
        for pattern in &config.file_patterns {
            println!("    - {}", pattern.bright_white());
        }
        
        println!("\n  {} {}", "→".bright_black(), "Directory patterns to clean:".bright_blue());
        for pattern in &config.dir_patterns {
            println!("    - {}", pattern.bright_white());
        }
        
        println!("\n  {} {}", "→".bright_black(), "Exception files:".bright_blue());
        for pattern in &config.exception_files {
            println!("    - {}", pattern.bright_white());
        }
        
        println!("\n  {} {}", "→".bright_black(), "Exception directories:".bright_blue());
        for pattern in &config.exception_dirs {
            println!("    - {}", pattern.bright_white());
        }

        println!("\n{}", "Starting cleanup process...".bright_cyan());
        // Create and run processor
        let processor = cleaner::processor::Processor::new(config);
        processor.process().await?;

        println!("{}", "Cleanup process completed successfully.".bright_green());
    }

    Ok(())
}

//TODO Implement .conf rather .json
async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = if cfg!(debug_assertions) {
        // Debug mode - look in current directory
        PathBuf::from("clean.config.json")
    } else {
        // Release mode - look in /etc/project-cleaner/
        PathBuf::from("/etc/project-cleaner/clean.config.json")
    };

    println!("\n{}", "📁 Loading configuration:".bright_blue().bold());
    println!("  {} Path: {}", "→".bright_black(), config_path.display().to_string().bright_white());

    let mut file = tokio::fs::File::open(&config_path).await
        .map_err(|e| format!("Failed to open config file at {}: {}", config_path.display(), e))?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    
    Ok(serde_json::from_str(&contents)?)
}