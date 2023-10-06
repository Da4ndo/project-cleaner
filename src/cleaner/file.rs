use colored::*;
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::Config;

use crate::create_regex;

#[derive(Deserialize)]
pub struct File {
    dir: String,
    backup_dir: String,
    file_patterns: Vec<String>,
    exception_files: Vec<String>,
    exception_folders: Vec<String>,
    env: String,
}

impl File {
    pub fn new(config: Config) -> Self {
        File {
            dir: config.dir,
            backup_dir: config.backup_dir,
            file_patterns: config.file_patterns,
            exception_files: config.exception_files,
            exception_folders: config.exception_dirs,
            env: config.env,
        }
    }

    pub async fn clean(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = PathBuf::from(&self.dir);
        let backup_dir = PathBuf::from(&self.backup_dir);
        let file_patterns = create_regex!(&self.file_patterns);
        let exception_files = create_regex!(&self.exception_files);
        let exception_dirs = create_regex!(&self.exception_folders);

        let walker = WalkDir::new(&dir).into_iter().filter_entry(|e| {
            let path = e.path();
            if path.is_dir() {
                let relative_path = path.strip_prefix(&dir).unwrap();
                let is_exception = self.is_in_exception_dir(relative_path, &exception_dirs);

                // If the directory is an exception or to be cleaned, skip it and its descendants  
                !(is_exception)
            } else {
                let relative_path = path.strip_prefix(&dir).unwrap();
                let is_exception = self.is_in_exception_dir(relative_path, &exception_files);
                
                !(is_exception)
            }
        });

        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&dir).unwrap();

            // Skip if it's a directory
            if path.is_dir() {
                continue;
            }

            // println!("{} Checking: {}", "[SEARCH]".cyan(), path.display());

            let is_to_clean = file_patterns
                .iter()
                .any(|p| p.is_match(relative_path.to_str().unwrap()));

            if is_to_clean {
                // If the file is to be cleaned, clean it
                let dest_path = backup_dir.join(relative_path);
                println!(
                    "{} Found target file: {}",
                    "[SEARCH]".green(),
                    path.display()
                );
                self._clean(path, &dest_path).await?;
            }
        }

        Ok(())
    }

    async fn _clean(
        &self,
        path: &Path,
        dest_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_path = path;
        let current_dest_path = dest_path;

        println!(
            "└──{} Copying from {} to {}",
            "[BACKUP]".yellow(),
            current_path.display(),
            current_dest_path.display()
        );

        // If the destination file exists, delete it
        if current_dest_path.exists() {
            match tokio::fs::remove_file(&current_dest_path).await {
                Ok(_) => {
                    if self.env == "development" {
                        println!(
                            "└──{} Successfully deleted destination file.",
                            "[SYSTEM-DEBUG]".green()
                        );
                    }
                }
                Err(e) => {
                    if self.env == "development" {
                        eprintln!(
                            "└──{} Failed to delete destination file: {}",
                            "[SYSTEM-DEBUG]".red(),
                            e
                        );
                    }
                }
            }
        }

        // Use fs::copy instead of fs_extra::dir::copy
        match tokio::fs::copy(current_path, current_dest_path).await {
            Ok(_) => {
                if self.env == "development" {
                    println!(
                        "└──{} Successfully copied file.",
                        "[SYSTEM-DEBUG]".green()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "└──{} Failed to copy file: {}",
                    "[SYSTEM-DEBUG]".red(),
                    e
                );
                println!(
                    "└──{} Skipping deletion due to copy failure.",
                    "[SYSTEM-DEBUG]".yellow()
                );
                return Ok(());
            }
        }

        if current_dest_path.exists() {
            println!(
                "└──{} Removing file: {}",
                "[SYSTEM]".red(),
                current_path.display()
            );
            match tokio::fs::remove_file(&current_path).await {
                Ok(_) => println!("└──{} Successfully removed file.", "[SYSTEM]".green()),
                Err(e) => eprintln!("└──{} Failed to remove file: {}", "[SYSTEM]".red(), e),
            }
        }

        Ok(())
    }

    fn is_in_exception_dir(&self, path: &Path, exception_dirs: &[Regex]) -> bool {
        let mut current_path = path;
        loop {
            if exception_dirs
                .iter()
                .any(|p| p.is_match(current_path.to_str().unwrap()))
            {
                return true;
            }
            if let Some(parent_path) = current_path.parent() {
                current_path = parent_path;
            } else {
                break;
            }
        }
        false
    }
}
