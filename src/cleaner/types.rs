use colored::*;
use humansize::{format_size, BINARY};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CleanTarget {
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub is_directory: bool,
}

#[derive(Debug)]
pub struct CleanSummary {
    pub total_items: usize,
    pub total_size: u64,
    pub source_total_size: u64,
    pub items: Vec<CleanTarget>,
}

impl CleanSummary {
    fn format_size(size: u64) -> String {
        format_size(size, BINARY)
    }

    pub fn display_summary(&self) {
        if self.total_items == 0 {
            println!("\n{}", "⚠️ No items to clean.".bright_yellow());
            return;
        }

        println!("\n{}:", "📋 Items to clean".bright_blue().bold());
        for item in &self.items {
            let icon = if item.is_directory { "📁" } else { "📄" };
            let item_type = if item.is_directory { "Directory" } else { "File" };
            println!(
                "  {} {} {}: {}",
                "→".bright_black(),
                icon,
                item_type.bright_magenta(),
                item.source_path.display().to_string().bright_white()
            );
        }

        let reduction_percentage = if self.source_total_size > 0 {
            (self.total_size as f64 / self.source_total_size as f64) * 100.0
        } else {
            0.0
        };

        println!("\n{}:", "📊 Clean summary".bright_blue().bold());
        println!(
            "  {} Total items: {}",
            "→".bright_black(),
            self.total_items.to_string().bright_white()
        );
        println!(
            "  {} Source size: {}",
            "→".bright_black(),
            Self::format_size(self.source_total_size).bright_white()
        );
        println!(
            "  {} Items size: {}",
            "→".bright_black(),
            Self::format_size(self.total_size).bright_white()
        );
        println!(
            "  {} After cleanup: {}",
            "→".bright_black(),
            Self::format_size(self.source_total_size.saturating_sub(self.total_size)).bright_white()
        );
        println!(
            "  {} Reduction: {}",
            "→".bright_black(),
            format!("{:.1}%", reduction_percentage).bright_white()
        );
    }
}
