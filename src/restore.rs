use colored::*;
use inquire::{Select, Confirm};
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use std::collections::HashMap;
use chrono::NaiveDateTime;
use regex::Regex;
use crate::create_regex;
use humansize::{format_size, BINARY};

use crate::Config;

pub struct Restorer {
    config: Config,
}

impl Restorer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    async fn get_backup_versions(&self, backup_dir: &Path) -> Result<Vec<(String, PathBuf)>, std::io::Error> {
        let mut versions = Vec::new();
        
        if !backup_dir.exists() {
            return Ok(versions);
        }

        // If versioning is disabled, treat the backup dir itself as the only "version"
        if !self.config.backup.versioning {
            versions.push(("current".to_string(), backup_dir.to_path_buf()));
            return Ok(versions);
        }

        let mut dir_reader = fs::read_dir(backup_dir).await?;
        while let Some(entry) = dir_reader.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                if NaiveDateTime::parse_from_str(&dir_name, "%Y%m%d_%H%M%S").is_ok() {
                    versions.push((dir_name, path));
                }
            }
        }

        versions.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(versions)
    }

    fn get_backup_items(&self, version_dir: &Path) -> Result<HashMap<String, PathBuf>, std::io::Error> {
        let mut items = HashMap::new();
        let base_dir = PathBuf::from(&self.config.dir);
        
        // Create regex patterns from config
        let file_patterns = create_regex!(&self.config.file_patterns);
        let dir_patterns = create_regex!(&self.config.dir_patterns);
        let mut processed_dirs = std::collections::HashSet::new();

        for entry in WalkDir::new(version_dir) {
            let entry = entry?;
            let path = entry.path();
            
            if let Ok(relative) = path.strip_prefix(version_dir) {
                let relative_str = relative.to_string_lossy();
                let is_dir = path.is_dir();

                // Check if any parent directory has already been processed
                let should_skip = relative
                    .ancestors()
                    .any(|ancestor| processed_dirs.contains(&ancestor.to_string_lossy().to_string()));

                if should_skip {
                    continue;
                }

                // Check if the path matches our patterns
                let matches_pattern = if is_dir {
                    dir_patterns.iter().any(|re| re.is_match(&relative_str))
                } else {
                    file_patterns.iter().any(|re| re.is_match(&relative_str))
                };

                if matches_pattern {
                    let target_path = base_dir.join(relative);
                    items.insert(relative.display().to_string(), target_path);
                    
                    if is_dir {
                        processed_dirs.insert(relative.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(items)
    }

    pub async fn restore(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n{}", "━".repeat(80).bright_black());
        println!("🔄 {} {}", "Restore Process".bold(), "Starting...".bright_cyan());
        println!("{}", "━".repeat(80).bright_black());

        if !self.config.backup.enabled {
            println!("{}", "⚠️  Backup functionality is not enabled in the configuration.".red());
            return Ok(());
        }

        let backup_dir = PathBuf::from(shellexpand::tilde(&self.config.backup.dir).into_owned());
        if !backup_dir.exists() {
            println!("{}", format!("⚠️  Backup directory not found: {}", backup_dir.display()).red());
            return Ok(());
        }

        // Get available versions or use main backup dir
        let versions = self.get_backup_versions(&backup_dir).await?;
        if versions.is_empty() {
            println!("{}", "⚠️  No backup versions found.".yellow());
            return Ok(());
        }

        let version_path = if self.config.backup.versioning {
            // Let user select which version to restore from
            let version_options: Vec<String> = versions.iter()
                .map(|(timestamp, _)| {
                    if timestamp == "current" {
                        "📁 Current backup".to_string()
                    } else if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp, "%Y%m%d_%H%M%S") {
                        format!("📅 Backup from {}", dt.format("%Y-%m-%d %H:%M:%S"))
                    } else {
                        timestamp.to_string()
                    }
                })
                .collect();

            let version_options_clone = version_options.clone();
            let selected_version = Select::new("Select version to restore from:", version_options)
                .with_help_message("↑↓ to move, enter to select")
                .prompt()?;

            let version_index = version_options_clone.iter().position(|x| x == &selected_version).unwrap();
            versions[version_index].1.clone()
        } else {
            // Use the main backup directory directly
            backup_dir
        };

        // Get items in the selected version
        let items = self.get_backup_items(&version_path)?;
        if items.is_empty() {
            println!("{}", "⚠️  No items found in this backup version.".yellow());
            return Ok(());
        }

        // Calculate sizes
        let mut total_restore_size = 0u64;
        let current_project_size = fs_extra::dir::get_size(&self.config.dir).unwrap_or(0);

        println!("\n📋 Items to restore:");
        for (relative_path, target_path) in &items {
            let source_path = version_path.join(relative_path);
            let item_size = if source_path.is_dir() {
                fs_extra::dir::get_size(&source_path).unwrap_or(0)
            } else {
                source_path.metadata()
                    .map(|m| m.len())
                    .unwrap_or(0)
            };
            total_restore_size += item_size;

            println!("  {} {} → {}", "→".bright_blue(), 
                relative_path.bright_white(),
                target_path.display().to_string().bright_cyan()
            );
        }

        let final_size = current_project_size + total_restore_size;

        println!("\n📊 Size Summary:");
        println!("  {} Current project size: {}", "→".bright_blue(), 
            format_size(current_project_size, BINARY).bright_white());
        println!("  {} Size to restore: {}", "→".bright_blue(), 
            format_size(total_restore_size, BINARY).bright_white());
        println!("  {} Final size after restore: {}", "→".bright_blue(), 
            format_size(final_size, BINARY).bright_white());

        println!("\n{}", "━".repeat(80).bright_black());
        let confirm = Confirm::new("🤔 Do you want to proceed with the restore?")
            .with_default(false)
            .with_help_message("This will overwrite existing files")
            .prompt()?;

        if !confirm {
            println!("{}", "🚫 Restore cancelled.".yellow());
            return Ok(());
        }

        println!("\n🚀 Starting restore process...");
        let mut success_count = 0;
        let mut error_count = 0;

        for (relative_path, target_path) in items {
            println!("\n{}", "Processing:".bright_blue());
            println!("  {} Source: {}", "→".bright_blue(), relative_path.bright_white());
            println!("  {} Target: {}", "→".bright_blue(), target_path.display().to_string().bright_white());

            // Create parent directories if they don't exist
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    match fs::create_dir_all(parent).await {
                        Ok(_) => println!("  {} Created directory: {}", "📁".bold(), parent.display()),
                        Err(e) => {
                            println!("  {} {}", "❌".bold(), 
                                format!("Failed to create directory {}: {}", parent.display(), e).red()
                            );
                            error_count += 1;
                            continue;
                        }
                    }
                }
            }

            let source_path = version_path.join(&relative_path);
            if source_path.is_dir() {
                let options = fs_extra::dir::CopyOptions::new()
                    .overwrite(true)
                    .content_only(false);

                match fs_extra::dir::copy(&source_path, target_path.parent().unwrap(), &options) {
                    Ok(_) => {
                        println!("  {} {}", "✅".bold(), "Directory restored successfully".green());
                        success_count += 1;
                    },
                    Err(e) => {
                        println!("  {} {}", "❌".bold(), 
                            format!("Failed to restore directory: {}", e).red()
                        );
                        error_count += 1;
                    }
                }
            } else {
                match fs::copy(&source_path, &target_path).await {
                    Ok(_) => {
                        println!("  {} {}", "✅".bold(), "File restored successfully".green());
                        success_count += 1;
                    },
                    Err(e) => {
                        println!("  {} {}", "❌".bold(), 
                            format!("Failed to restore file: {}", e).red()
                        );
                        error_count += 1;
                    }
                }
            }
        }

        println!("\n{}", "━".repeat(80).bright_black());
        println!("📊 Restore Summary:");
        println!("  {} Successful: {}", "→".bright_blue(), success_count.to_string().green());
        println!("  {} Failed: {}", "→".bright_blue(), error_count.to_string().red());
        println!("{}", "━".repeat(80).bright_black());

        Ok(())
    }
}
