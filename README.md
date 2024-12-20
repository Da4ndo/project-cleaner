<div align="center">

![Forks](https://img.shields.io/github/forks/Da4ndo/project-cleaner?label=Forks&color=lime&logo=githubactions&logoColor=lime)
![Stars](https://img.shields.io/github/stars/Da4ndo/project-cleaner?label=Stars&color=yellow&logo=reverbnation&logoColor=yellow)
![License](https://img.shields.io/github/license/Da4ndo/project-cleaner?label=License&color=808080&logo=gitbook&logoColor=808080)
![Issues](https://img.shields.io/github/issues/Da4ndo/project-cleaner?label=Issues&color=red&logo=ifixit&logoColor=red)

# 🧹 ProjectCleaner

<p align="center">
  <strong>A blazingly fast Rust utility for cleaning up project build files</strong>
</p>

<p align="center">
  <em>Efficiently manage and clean your project's build artifacts with customizable patterns and safety features</em>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#%EF%B8%8F-getting-started">Getting Started</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a>
</p>

</div>

## ✨ Overview

ProjectCleaner is a high-performance Rust utility designed to streamline your development workflow by efficiently managing build files and other unnecessary artifacts. Built with safety and customization in mind, it offers powerful pattern matching capabilities while ensuring your important files remain protected.

Made with ❤️ by Da4ndo. If you find this project helpful, consider giving it a star ⭐️!

## 🚀 Features

### 📋 Pattern Management
- **Customizable Patterns** - Define flexible file and directory patterns
- **Regex Support** - Powerful regex-based pattern matching
- **Exception Handling** - Protect specific files/directories from deletion

### 🛡️ Safety Features
- **Safe Deletion** - Carefully validates files before removal
- **Smart Detection** - Intelligently identifies common build artifacts
- **Preview Mode** - Review changes before applying

### 💾 Backup System
- **Multiple Modes**
  - Direct deletion (no backup)
  - Simple one-time backups
  - Versioned date-based backups
- **Restore Capabilities**
  - Restore from latest backup
  - Select specific versions
  - Preview restoration changes

## 🛠️ Getting Started

### 🔧 Installation

#### 📦 Arch Linux

You can install ProjectCleaner by running the following command: 
```bash
yay -S project-cleaner
```

#### 🐧 Debian/Ubuntu

For Debian/Ubuntu systems, download the latest release package and install manually:

1. Download the latest release from [GitHub Releases](https://github.com/Da4ndo/project-cleaner/releases/latest)
2. Extract the downloaded package, navigate to the directory, and install:
```bash
cp project-cleaner /usr/bin/project-cleaner
mkdir -p /etc/project-cleaner
cp clean.config.json /etc/project-cleaner/clean.config.json
```


#### 🔨 Build from Source

You can install ProjectCleaner by running the following command: 
```bash
git clone https://github.com/Da4ndo/project-cleaner
cd project-cleaner

cargo build --release

cp target/release/project-cleaner /usr/bin/project-cleane
mkdir -p /etc/project-cleaner
cp clean.config.json /etc/project-cleaner/clean.config.json
```

### ⚙️ Configuration

The default configuration file is located at `/etc/project-cleaner/clean.config.json`. You can customize the cleaning patterns and behavior by editing this file:

```bash
sudo nano /etc/project-cleaner/clean.config.json
```

Example configuration:
```json
{
  "dir": "/projects",
  "target": {
    "files_patterns": ["\\.pyc$", "\\.o$", "\\.a$"],
    "dirs_patterns": ["target$", "node_modules$", "build$", "dist$"]
  },
  "exclude": {
    "files_patterns": ["README\\.md$", "LICENSE$"],
    "dirs_patterns": ["project-cleaner$", "\\.git$", "\\.vscode$", "test$"]
  },

  "backup": {
    "enabled": true,
    "dir": "~/.backup",
    "versioning": false
  }
}
```

#### Configuration Options:
- **dir**: The root directory to scan for cleanup
- **target**: Define patterns for files/directories to clean
  - `files_patterns`: Regex patterns for files to clean (e.g. ".pyc$", ".o$")
  - `dirs_patterns`: Regex patterns for directories to clean (e.g. "target$", "node_modules$")
- **exclude**: Define patterns for files/directories to protect
  - `files_patterns`: Regex patterns for files to exclude (e.g. "README.md$")
  - `dirs_patterns`: Regex patterns for directories to exclude (e.g. ".git$")
- **backup**: Backup settings
  - `enabled`: Enable/disable backup functionality (true/false)
  - `dir`: Directory path for storing backups (e.g. "~/.backup")
  - `versioning`: Enable/disable versioned backups (true/false)

## 📖 Documentation

Project Cleaner is a high-performance tool designed for efficient project cleanup. Key features include:

### ⚡️ Performance
- **Parallel Processing** - Leverages Tokio's multi-threading for maximum speed
- **Minimal Memory Usage** - Efficient resource utilization

### 🎯 Precision
- **Intelligent Scanning** - Advanced pattern matching algorithms
- **Granular Control** - Fine-tuned cleanup rules
- **Validation Checks** - Multiple safety verification steps

### 🔄 Workflow Integration
- **CI/CD Ready** - Seamless integration with build pipelines
- **Configuration Profiles** - Save and reuse cleanup settings

## 🤝 Contributing

We welcome contributions to make Project Cleaner even better! Here's how you can help:

- **🐛 Bug Reports** - Help identify and fix issues
- **✨ Feature Requests** - Share ideas for improvements
- **📝 Documentation** - Enhance guides and examples
- **💻 Code** - Submit pull requests and improvements