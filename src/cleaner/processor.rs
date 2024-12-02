use colored::*;
use inquire::Confirm;
use regex::Regex;
use shellexpand;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, create_dir_all};
use walkdir::WalkDir;
use futures::future::join_all;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::task::JoinHandle;

use super::types::*;
use crate::Config;

type CleanResult = Result<Vec<CleanTarget>, Box<dyn std::error::Error + Send>>;
type CleanJoinHandle = JoinHandle<CleanResult>;

// Add these macros at the top of the file
macro_rules! handle_path_operation {
    ($op:expr, $path:expr, $err_msg:expr) => {
        match $op {
            Ok(result) => result,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "{}",
                    format!("⚠️ Path not found: {}", $path.display()).yellow()
                );
                return Err(format!("Path not found: {}", $path.display()).into());
            }
            Err(e) => return Err(format!("{}: {}", $err_msg, e).into()),
        }
    };
}

macro_rules! safe_path_exists {
    ($path:expr) => {
        $path.try_exists().unwrap_or(false)
    };
}

#[macro_export]
macro_rules! create_regex {
    ($patterns:expr) => {
        $patterns
            .iter()
            .map(|p| Regex::new(&p).unwrap())
            .collect::<Vec<Regex>>()
    };
}


// Rename macro to be more specific about its purpose
macro_rules! ensure_backup_subdir {
    ($path:expr) => {
        if let Some(parent) = $path.parent() {
            if !safe_path_exists!(parent) {
                handle_path_operation!(
                    create_dir_all(parent).await,
                    parent,
                    "Failed to create backup subdirectory"
                );
            }
        }
    };
}

pub struct Processor {
    config: Config,
}

impl Processor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        let summary = self.scan_directory().await?;
        summary.display_summary();

        if summary.total_items == 0 {
            return Ok(());
        }

        let should_continue = Confirm::new("🤔 Do you want to proceed with the cleanup?")
            .with_default(false)
            .prompt()?;

        if !should_continue {
            println!("{}", "🚫 Cleanup cancelled.".yellow());
            return Ok(());
        }

        self.execute_cleanup(&summary).await?;
        Ok(())
    }

    async fn scan_directory(&self) -> Result<CleanSummary, Box<dyn std::error::Error>> {
        let source_dir = PathBuf::from(&self.config.dir);
        let backup_dir = PathBuf::from(&self.config.backup.dir);
        let total_items = Arc::new(AtomicUsize::new(0));
        let total_size = Arc::new(AtomicUsize::new(0));

        // Compile patterns once
        let file_patterns = Arc::new(create_regex!(&self.config.file_patterns));
        let dir_patterns = Arc::new(create_regex!(&self.config.dir_patterns));
        let exception_files = Arc::new(create_regex!(&self.config.exception_files));
        let exception_dirs = Arc::new(create_regex!(&self.config.exception_dirs));

        let mut root_entries = Vec::new();
        let mut dir_reader = tokio::fs::read_dir(&source_dir).await?;
        while let Ok(Some(entry)) = dir_reader.next_entry().await {
            root_entries.push(entry);
        }

        let chunk_size = (root_entries.len() / num_cpus::get()).max(1);
        let mut handles: Vec<CleanJoinHandle> = Vec::new();

        for chunk in root_entries.chunks(chunk_size) {
            let source_dir = source_dir.clone();
            let backup_dir = backup_dir.clone();
            let file_patterns = Arc::clone(&file_patterns);
            let dir_patterns = Arc::clone(&dir_patterns);
            let exception_files = Arc::clone(&exception_files);
            let exception_dirs = Arc::clone(&exception_dirs);
            let total_items = Arc::clone(&total_items);
            let total_size = Arc::clone(&total_size);
            
            let chunk_paths: Vec<PathBuf> = chunk.iter().map(|entry| entry.path()).collect();

            let handle = tokio::spawn(async move {
                let mut targets = Vec::new();
                let mut parent_dirs_to_clean = Vec::new();

                for path in chunk_paths {
                    if path.is_dir() {
                        let walker = WalkDir::new(&path).into_iter();
                        for entry in walker {
                            let entry = entry.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                            let path = entry.path();
                            
                            // Skip if any parent directory is already marked for cleanup
                            if parent_dirs_to_clean.iter().any(|parent: &PathBuf| path.starts_with(parent)) {
                                continue;
                            }

                            let relative_path = path.strip_prefix(&source_dir)
                                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                            let is_dir = path.is_dir();

                            if !Self::is_exception(relative_path, is_dir, &exception_files, &exception_dirs)
                                && Self::should_clean(relative_path, is_dir, &file_patterns, &dir_patterns)
                            {
                                let size = if is_dir {
                                    fs_extra::dir::get_size(path).unwrap_or(0)
                                } else {
                                    path.metadata()
                                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
                                        .len()
                                };

                                total_size.fetch_add(size as usize, Ordering::Relaxed);
                                total_items.fetch_add(1, Ordering::Relaxed);

                                let target = CleanTarget {
                                    source_path: path.to_path_buf(),
                                    backup_path: backup_dir.join(relative_path),
                                    is_directory: is_dir,
                                };

                                if is_dir {
                                    parent_dirs_to_clean.push(path.to_path_buf());
                                }

                                targets.push(target);
                            }
                        }
                    }
                }

                Ok(targets)
            });

            handles.push(handle);
        }

        // Collect results
        let results = join_all(handles).await;
        let mut all_targets = Vec::new();
        
        for result in results {
            match result? {
                Ok(targets) => all_targets.extend(targets),
                Err(e) => eprintln!("{}", format!("Error scanning directory: {}", e).red()),
            }
        }

        Ok(CleanSummary {
            total_items: total_items.load(Ordering::Relaxed),
            total_size: total_size.load(Ordering::Relaxed) as u64,
            source_total_size: fs_extra::dir::get_size(&source_dir).unwrap_or(0),
            items: all_targets,
        })
    }

    async fn execute_cleanup(&self, summary: &CleanSummary) -> Result<(), Box<dyn std::error::Error>> {
        let valid_items: Vec<_> = summary
            .items
            .iter()
            .filter(|target| safe_path_exists!(&target.source_path))
            .collect();

        println!("🔍 Found {} items to process", valid_items.len());

        let mut success_count = 0;
        let mut error_count = 0;

        if !self.config.backup.enabled {
            for target in &valid_items {
                handle_path_operation!(
                    if target.is_directory {
                        fs::remove_dir_all(&target.source_path).await
                    } else {
                        fs::remove_file(&target.source_path).await
                    },
                    &target.source_path,
                    "Failed to remove item"
                );
                println!(
                    "{}",
                    format!(
                        "🗑️  Removed {}: {}",
                        if target.is_directory {
                            "directory"
                        } else {
                            "file"
                        },
                        target.source_path.display()
                    )
                    .green()
                );
            }
            return Ok(());
        }

        let main_backup_dir =
            PathBuf::from(shellexpand::tilde(&self.config.backup.dir).into_owned());
        if !safe_path_exists!(&main_backup_dir) {
            handle_path_operation!(
                create_dir_all(&main_backup_dir).await,
                &main_backup_dir,
                "Failed to create main backup directory"
            );
            println!(
                "{}",
                format!("📁 Created backup directory: {}", main_backup_dir.display()).cyan()
            );
        }

        // Create timestamp directory directly after main backup directory if versioning is enabled
        let versioned_backup_dir = if self.config.backup.versioning {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            main_backup_dir.join(&timestamp)
        } else {
            main_backup_dir.clone()
        };

        for target in &valid_items {
            // Clear previous line and show processing status
            println!("\n{}", "━".repeat(80).bright_black());
            println!("{} Processing: {}", "⏳".bold(), target.source_path.display().to_string().bright_white());

            // Calculate the relative path from the source directory to the target
            let relative_path = target.source_path.strip_prefix(PathBuf::from(&self.config.dir))
                .map_err(|e| format!("Failed to calculate relative path: {}", e))?;

            // Create the final backup path by joining the versioned backup directory with the relative path
            let final_backup_path = versioned_backup_dir.join(relative_path);

            println!("  {} Source:      {}", "→".bright_blue(), target.source_path.display().to_string().bright_white());
            println!("  {} Destination: {}", "→".bright_blue(), final_backup_path.display().to_string().bright_white());

            ensure_backup_subdir!(&final_backup_path);

            if safe_path_exists!(&final_backup_path) && !self.config.backup.versioning {
                print!(
                    "🗑️ Removing existing backup: {}",
                    final_backup_path.display()
                );
                handle_path_operation!(
                    if target.is_directory {
                        fs::remove_dir_all(&final_backup_path).await
                    } else {
                        fs::remove_file(&final_backup_path).await
                    },
                    &final_backup_path,
                    "Failed to remove existing backup"
                );
            }

            println!("📦 Copying to backup location ...");

            // Backup and remove
            if target.is_directory {
                println!("  {} {}", "📦".bold(), "Creating backup structure...".bright_cyan());
                match fs::create_dir_all(&final_backup_path).await {
                    Ok(_) => {
                        println!("  {} {}", "📦".bold(), "Copying contents...".bright_cyan());
                        let options = fs_extra::dir::CopyOptions::new()
                            .overwrite(true)
                            .content_only(false);

                        match fs_extra::dir::copy(&target.source_path, final_backup_path.parent().unwrap(), &options) {
                            Ok(_) => {
                                match fs::remove_dir_all(&target.source_path).await {
                                    Ok(_) => {
                                        println!("  {} {}", "✅".bold(), "Operation completed".green());
                                        success_count += 1;
                                    },
                                    Err(e) => {
                                        println!("  {} {}", "❌".bold(), format!("Removal failed: {}", e).red());
                                        error_count += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("\r{}", format!("⚠️  Failed to copy directory {}: {}", 
                                    target.source_path.display(), e).red());
                                error_count += 1;
                                continue;
                            }
                        }
                    },
                    Err(e) => {
                        println!("  {} {}", "❌".bold(), format!("Failed to create backup directory: {}", e).red());
                        error_count += 1;
                        continue;
                    }
                }
            } else {
                match fs::copy(&target.source_path, &final_backup_path).await {
                    Ok(_) => {
                        match fs::remove_file(&target.source_path).await {
                            Ok(_) => {
                                println!("  {} {}", "✅".bold(), "Operation completed".green());
                                success_count += 1;
                            },
                            Err(e) => {
                                println!("  {} {}", "❌".bold(), format!("Removal failed: {}", e).red());
                                error_count += 1;
                            }
                        }
                    },
                    Err(e) => {
                        println!("  {} {}", "❌".bold(), format!("Backup failed: {}", e).red());
                        error_count += 1;
                    }
                }
            }

            println!(
                "✅ Cleaned {}: {} (Backed up to: {})",
                if target.is_directory {
                    "directory"
                } else {
                    "file"
                },
                target.source_path.display(),
                final_backup_path.display()
            );

            println!("{}", "━".repeat(80).bright_black());
        }

        // Add summary at the end
        println!("\n{}", "━".repeat(80).bright_black());
        println!("📊 Cleanup Summary:");
        println!("  {} Successful: {}", "→".bright_blue(), success_count.to_string().green());
        println!("  {} Failed: {}", "→".bright_blue(), error_count.to_string().red());
        println!("{}", "━".repeat(80).bright_black());

        Ok(())
    }

    #[inline]
    fn is_exception(
        path: &Path,
        is_dir: bool,
        exception_files: &[Regex],
        exception_dirs: &[Regex],
    ) -> bool {
        let path_str = path.to_string_lossy();
        
        if is_dir || path.parent().is_some() {
            for ancestor in path.ancestors() {
                if exception_dirs.iter().any(|re| re.is_match(&ancestor.to_string_lossy())) {
                    return true;
                }
            }
        }

        !is_dir && exception_files.iter().any(|re| re.is_match(&path_str))
    }

    #[inline]
    fn should_clean(
        path: &Path,
        is_dir: bool,
        file_patterns: &[Regex],
        dir_patterns: &[Regex],
    ) -> bool {
        let path_str = path.to_string_lossy();
        if is_dir {
            dir_patterns.iter().any(|re| re.is_match(&path_str))
        } else {
            file_patterns.iter().any(|re| re.is_match(&path_str))
        }
    }
}
