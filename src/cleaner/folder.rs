use colored::*;
use fs_extra::dir::{self, CopyOptions};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::Config;

use crate::create_regex;

#[derive(Deserialize)]
pub struct Folder {
    dir: String,
    backup_dir: String,
    dir_patterns: Vec<String>,
    exception_dirs: Vec<String>,
    env: String,
}

impl Folder {
    pub fn new(config: Config) -> Self {
        Folder {
            dir: config.dir,
            backup_dir: config.backup_dir,
            dir_patterns: config.dir_patterns,
            exception_dirs: config.exception_dirs,
            env: config.env,
        }
    }

    pub async fn clean(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = PathBuf::from(&self.dir);
        let backup_dir = PathBuf::from(&self.backup_dir);
        let dir_patterns = create_regex!(&self.dir_patterns);
        let subdir_patterns = create_regex!(&self
            .dir_patterns
            .iter()
            .map(|s| s.replace('$', "/"))
            .collect::<Vec<_>>());
        let exception_dirs = create_regex!(&self.exception_dirs);

        let walker = WalkDir::new(&dir).into_iter().filter_entry(|e| {
            let path = e.path();
            if path.is_dir() {
                let relative_path = path.strip_prefix(&dir).unwrap();
                let is_exception = self.is_in_exception_dir(relative_path, &exception_dirs);
                let is_subdir_of_target = subdir_patterns
                    .iter()
                    .any(|p| p.is_match(relative_path.to_str().unwrap()));

                // If the directory is an exception or to be cleaned, skip it and its descendants
                !(is_exception || is_subdir_of_target)
            } else {
                false
            }
        });

        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&dir).unwrap();

            // println!("{} Checking: {}", "[SEARCH]".cyan(), path.display());

            let is_to_clean = dir_patterns
                .iter()
                .any(|p| p.is_match(relative_path.to_str().unwrap()));

            if is_to_clean {
                // If the directory is to be cleaned, clean it
                let dest_path = backup_dir.join(relative_path.parent().unwrap());
                println!(
                    "{} Found target folder: {}",
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

        // If the destination directory exists, delete it
        if current_dest_path.exists() {
            match tokio::fs::remove_dir_all(&current_dest_path).await {
                Ok(_) => {
                    if self.env == "development" {
                        println!(
                            "└──{} Successfully deleted destination directory.",
                            "[SYSTEM-DEBUG]".green()
                        );
                    }
                }
                Err(e) => {
                    if self.env == "development" {
                        eprintln!(
                            "└──{} Failed to delete destination directory: {}",
                            "[SYSTEM-DEBUG]".red(),
                            e
                        );
                    }
                }
            }
        }

        match tokio::fs::create_dir_all(&current_dest_path).await {
            Ok(_) => {
                if self.env == "development" {
                    println!(
                        "└──{} Successfully created destination directory.",
                        "[SYSTEM-DEBUG]".green()
                    );
                }
            }
            Err(e) => {
                if self.env == "development" {
                    eprintln!(
                        "└──{} Failed to create destination directory: {}",
                        "[SYSTEM-DEBUG]".red(),
                        e
                    );
                }
            }
        }

        // Use fs_extra::dir::copy instead of fs::copy
        let mut options = CopyOptions::new();
        options.overwrite = true;
        match dir::copy(current_path, current_dest_path, &options) {
            Ok(_) => {
                if self.env == "development" {
                    println!(
                        "└──{} Successfully copied directory.",
                        "[SYSTEM-DEBUG]".green()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "└──{} Failed to copy directory: {}",
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
                "└──{} Removing directory: {}",
                "[SYSTEM]".red(),
                current_path.display()
            );
            match tokio::fs::remove_dir_all(&current_path).await {
                Ok(_) => println!("└──{} Successfully removed directory.", "[SYSTEM]".green()),
                Err(e) => eprintln!("└──{} Failed to remove directory: {}", "[SYSTEM]".red(), e),
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
