//! Git integration for lsn.
//!
//! Provides git status detection and formatting for files and directories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::Colorize;

use crate::config::{get_config, parse_color};

/// Git status for a file or directory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitStatus {
    Modified,
    Staged,
    Untracked,
    Ignored,
    Clean,
}

/// Get priority for git status (higher = more important).
fn git_status_priority(status: &GitStatus) -> u8 {
    match status {
        GitStatus::Modified => 3,
        GitStatus::Staged => 2,
        GitStatus::Untracked => 1,
        GitStatus::Ignored => 0,
        GitStatus::Clean => 0,
    }
}

/// Get git statuses for all files in a directory.
///
/// Returns a map from absolute paths to their git status.
/// Status is propagated to parent directories with the highest priority status.
pub fn get_git_statuses(dir: &Path) -> HashMap<PathBuf, GitStatus> {
    let mut statuses = HashMap::new();

    // Get absolute path of directory
    let abs_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    // Check if we're in a git repo
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(&abs_dir)
        .output();

    if output.is_err() || !output.unwrap().status.success() {
        return statuses;
    }

    // Get the git root directory
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&abs_dir)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| PathBuf::from(s.trim()))
        .unwrap_or_else(|| abs_dir.clone());

    // Get status for all files (paths are relative to git root)
    if let Ok(output) = Command::new("git")
        .args(["status", "--porcelain", "-uall"])
        .current_dir(&git_root)
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.len() < 4 {
                    continue;
                }
                let status_chars: Vec<char> = line.chars().take(2).collect();
                let file_path = line[3..].trim();
                // Build absolute path from git root
                let path = git_root.join(file_path);

                let status = match (status_chars[0], status_chars[1]) {
                    ('?', '?') => GitStatus::Untracked,
                    ('!', '!') => GitStatus::Ignored,
                    (_, 'M') | (_, 'D') | (_, 'A') => GitStatus::Modified,
                    ('M', _) | ('A', _) | ('D', _) | ('R', _) | ('C', _) => GitStatus::Staged,
                    _ => GitStatus::Clean,
                };

                statuses.insert(path.clone(), status);

                // Propagate status to parent directories
                let mut parent = path.parent();
                while let Some(p) = parent {
                    // Stop at or above git root
                    if p < git_root {
                        break;
                    }

                    let current_status = statuses.get(p);
                    let should_update = match current_status {
                        None => true,
                        Some(existing) => git_status_priority(&status) > git_status_priority(existing),
                    };

                    if should_update {
                        statuses.insert(p.to_path_buf(), status);
                    }

                    parent = p.parent();
                }
            }
        }
    }

    statuses
}

/// Format a git status as a colored symbol.
pub fn format_git_status(status: Option<&GitStatus>) -> String {
    let config = get_config();
    let git = &config.git;
    let colors = &config.colors;

    match status {
        Some(GitStatus::Modified) => git.modified.color(parse_color(&colors.git_modified)).to_string(),
        Some(GitStatus::Staged) => git.staged.color(parse_color(&colors.git_staged)).to_string(),
        Some(GitStatus::Untracked) => git.untracked.color(parse_color(&colors.git_untracked)).to_string(),
        Some(GitStatus::Ignored) => git.ignored.bright_black().to_string(),
        Some(GitStatus::Clean) | None => " ".to_string(),
    }
}
