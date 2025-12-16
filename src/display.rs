//! Display and output formatting for lsnote.
//!
//! Handles directory listing, tree view, and entry formatting.

use std::collections::HashMap;
use std::fs::{self, Metadata, Permissions};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use colored::Colorize;

use crate::config::{get_config, parse_color};
use crate::git::{format_git_status_ex, get_git_statuses, GitStatus};
use crate::icons::{get_icon, is_executable};
use crate::notes::get_note;

/// Display options for listing.
#[derive(Clone)]
pub struct DisplayOptions {
    pub show_all: bool,
    pub long_format: bool,
    pub show_icons: bool,
    pub human_readable: bool,
    pub show_git: bool,
    #[allow(dead_code)]
    pub tree_view: bool,
}

/// Format a size in bytes to human-readable format.
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.1}T", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

/// Colorize a file name based on its type and git status.
pub fn colorize_name(name: &str, metadata: &Metadata, git_status: Option<&GitStatus>) -> String {
    let config = get_config();
    let colors = &config.colors;

    // Apply git status colors if available
    match git_status {
        Some(GitStatus::Modified) => {
            let color = parse_color(&colors.git_modified);
            if metadata.is_dir() {
                name.color(color).bold().to_string()
            } else {
                name.color(color).to_string()
            }
        }
        Some(GitStatus::Staged) => {
            let color = parse_color(&colors.git_staged);
            if metadata.is_dir() {
                name.color(color).bold().to_string()
            } else {
                name.color(color).to_string()
            }
        }
        Some(GitStatus::Untracked) => {
            let color = parse_color(&colors.git_untracked);
            if metadata.is_dir() {
                name.color(color).bold().to_string()
            } else {
                name.color(color).to_string()
            }
        }
        _ => {
            // Default coloring based on file type
            if metadata.is_dir() {
                name.color(parse_color(&colors.directory))
                    .bold()
                    .to_string()
            } else if metadata.file_type().is_symlink() {
                name.color(parse_color(&colors.symlink)).to_string()
            } else if is_executable(metadata) {
                name.color(parse_color(&colors.executable))
                    .bold()
                    .to_string()
            } else {
                name.color(parse_color(&colors.file)).to_string()
            }
        }
    }
}

/// Format file permissions as a string (e.g., "drwxr-xr-x").
fn format_permissions(permissions: Permissions, metadata: &Metadata) -> String {
    let mode = permissions.mode();

    let file_type = if metadata.file_type().is_symlink() {
        'l'
    } else if metadata.is_dir() {
        'd'
    } else if metadata.file_type().is_block_device() {
        'b'
    } else if metadata.file_type().is_char_device() {
        'c'
    } else {
        '-'
    };

    let user = triplet((mode >> 6) & 0o7);
    let group = triplet((mode >> 3) & 0o7);
    let other = triplet(mode & 0o7);

    format!("{}{}{}{}", file_type, user, group, other)
}

/// Format a permission triplet (rwx).
fn triplet(mode: u32) -> String {
    let r = if mode & 0o4 != 0 { 'r' } else { '-' };
    let w = if mode & 0o2 != 0 { 'w' } else { '-' };
    let x = if mode & 0o1 != 0 { 'x' } else { '-' };
    format!("{}{}{}", r, w, x)
}

/// Get sorted directory entries.
pub fn get_sorted_entries(path: &Path, show_all: bool) -> Vec<PathBuf> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    let mut items: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            if show_all {
                true
            } else {
                !p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
            }
        })
        .collect();

    items.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });

    items
}

/// List a directory's contents.
pub fn list_directory(path: &Path, opts: &DisplayOptions) {
    let output = build_list(path, opts, true);
    print!("{}", output);
}

/// Build a directory listing and return it as a String.
/// If `for_display` is true, includes ANSI colors. If false, plain text for clipboard.
pub fn build_list(path: &Path, opts: &DisplayOptions, for_display: bool) -> String {
    let mut output = String::new();

    if path.is_file() {
        let git_statuses = if opts.show_git {
            get_git_statuses(path.parent().unwrap_or(Path::new(".")))
        } else {
            HashMap::new()
        };
        output.push_str(&build_entry(path, opts, &git_statuses, for_display));
        return output;
    }

    let items = get_sorted_entries(path, opts.show_all);
    if items.is_empty() && !path.is_dir() {
        return format!("Error reading directory: {}\n", path.display());
    }

    let git_statuses = if opts.show_git {
        get_git_statuses(path)
    } else {
        HashMap::new()
    };

    if opts.long_format {
        // Calculate total blocks
        let total: u64 = items
            .iter()
            .filter_map(|p| fs::metadata(p).ok())
            .map(|m| m.blocks())
            .sum();
        output.push_str(&format!("total {}\n", total / 2)); // Convert 512-byte blocks to 1K blocks
    }

    for item in items {
        output.push_str(&build_entry(&item, opts, &git_statuses, for_display));
    }

    output
}

/// Print a tree view of a directory.
pub fn print_tree(path: &Path, opts: &DisplayOptions, prefix: &str, _is_last: bool) {
    let output = build_tree(path, opts, prefix, true);
    print!("{}", output);
}

/// Build a tree view of a directory and return it as a String.
/// If `for_display` is true, includes ANSI colors. If false, plain text for clipboard.
pub fn build_tree(path: &Path, opts: &DisplayOptions, prefix: &str, for_display: bool) -> String {
    let mut output = String::new();
    build_tree_recursive(path, opts, prefix, true, &mut output, for_display);
    output
}

/// Recursive helper for building tree output.
fn build_tree_recursive(
    path: &Path,
    opts: &DisplayOptions,
    prefix: &str,
    is_root: bool,
    output: &mut String,
    for_display: bool,
) {
    let items = get_sorted_entries(path, opts.show_all);

    let git_statuses = if opts.show_git {
        get_git_statuses(path)
    } else {
        HashMap::new()
    };

    // Print current directory name if this is the root call
    if is_root && prefix.is_empty() {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or(".");
        let config = get_config();
        if for_display {
            let colored_name = name.color(parse_color(&config.colors.directory)).bold();
            if opts.show_icons {
                output.push_str(&format!("{} {}\n", config.icons.directory, colored_name));
            } else {
                output.push_str(&format!("{}\n", colored_name));
            }
        } else if opts.show_icons {
            output.push_str(&format!("{} {}\n", config.icons.directory, name));
        } else {
            output.push_str(&format!("{}\n", name));
        }
    }

    for (i, item) in items.iter().enumerate() {
        let is_last_item = i == items.len() - 1;
        let connector = if is_last_item {
            "└── "
        } else {
            "├── "
        };
        let child_prefix = if is_last_item { "    " } else { "│   " };

        let metadata = match fs::symlink_metadata(item) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_name = item.file_name().and_then(|n| n.to_str()).unwrap_or("?");

        // Use absolute path for git status lookup
        let abs_item = item.canonicalize().unwrap_or_else(|_| item.to_path_buf());
        let git_status = git_statuses.get(&abs_item);

        let display_name = if for_display {
            colorize_name(
                file_name,
                &metadata,
                if opts.show_git { git_status } else { None },
            )
        } else {
            file_name.to_string()
        };

        let icon = if opts.show_icons {
            format!("{} ", get_icon(file_name, &metadata))
        } else {
            String::new()
        };

        let git_indicator = if opts.show_git {
            format!("{} ", format_git_status_ex(git_status, for_display))
        } else {
            String::new()
        };

        let note = get_note(item);
        let note_str = if let Some(n) = note {
            if for_display {
                format!("  {}", format!("# {}", n).bright_black())
            } else {
                format!("  # {}", n)
            }
        } else {
            String::new()
        };

        output.push_str(&format!(
            "{}{}{}{}{}{}\n",
            prefix, connector, git_indicator, icon, display_name, note_str
        ));

        // Recurse into directories
        if metadata.is_dir() {
            let new_prefix = format!("{}{}", prefix, child_prefix);
            build_tree_recursive(item, opts, &new_prefix, false, output, for_display);
        }
    }
}

/// Build a single directory entry as a String.
fn build_entry(
    path: &Path,
    opts: &DisplayOptions,
    git_statuses: &HashMap<PathBuf, GitStatus>,
    for_display: bool,
) -> String {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");

    let note = get_note(path);
    // Use absolute path for git status lookup
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let git_status = git_statuses.get(&abs_path);

    if opts.long_format {
        build_long_format(
            path,
            &metadata,
            file_name,
            note,
            opts,
            git_status,
            for_display,
        )
    } else {
        build_short_format(file_name, &metadata, note, opts, git_status, for_display)
    }
}

/// Build an entry in long format as a String.
fn build_long_format(
    path: &Path,
    metadata: &Metadata,
    name: &str,
    note: Option<String>,
    opts: &DisplayOptions,
    git_status: Option<&GitStatus>,
    for_display: bool,
) -> String {
    let mut output = String::new();

    let mode = format_permissions(metadata.permissions(), metadata);
    let nlink = metadata.nlink();
    let uid = metadata.uid();
    let gid = metadata.gid();

    let user = users::get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string());

    let group = users::get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string());

    let size = metadata.len();
    let size_str = if opts.human_readable {
        format!("{:>6}", format_size(size))
    } else {
        format!("{:>8}", size)
    };

    let modified: DateTime<Local> = DateTime::from(
        metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
    );
    let date_str = modified.format("%b %e %H:%M").to_string();

    let display_name = if for_display {
        colorize_name(
            name,
            metadata,
            if opts.show_git { git_status } else { None },
        )
    } else {
        name.to_string()
    };

    let icon_prefix = if opts.show_icons {
        format!("{} ", get_icon(name, metadata))
    } else {
        String::new()
    };

    let git_indicator = if opts.show_git {
        format!("{} ", format_git_status_ex(git_status, for_display))
    } else {
        String::new()
    };

    // Handle symlinks
    let link_target = if metadata.file_type().is_symlink() {
        fs::read_link(path)
            .ok()
            .map(|t| format!(" -> {}", t.display()))
            .unwrap_or_default()
    } else {
        String::new()
    };

    output.push_str(&format!(
        "{} {:>2} {:<8} {:<8} {} {} {}{}{}{}",
        mode,
        nlink,
        user,
        group,
        size_str,
        date_str,
        git_indicator,
        icon_prefix,
        display_name,
        link_target
    ));

    if let Some(n) = note {
        if for_display {
            output.push_str(&format!("  {}", format!("# {}", n).bright_black()));
        } else {
            output.push_str(&format!("  # {}", n));
        }
    }
    output.push('\n');

    output
}

/// Build an entry in short format as a String.
fn build_short_format(
    name: &str,
    metadata: &Metadata,
    note: Option<String>,
    opts: &DisplayOptions,
    git_status: Option<&GitStatus>,
    for_display: bool,
) -> String {
    let mut output = String::new();

    if opts.show_git {
        output.push_str(&format!(
            "{} ",
            format_git_status_ex(git_status, for_display)
        ));
    }
    if opts.show_icons {
        output.push_str(&format!("{} ", get_icon(name, metadata)));
    }

    let display_name = if for_display {
        colorize_name(
            name,
            metadata,
            if opts.show_git { git_status } else { None },
        )
    } else {
        name.to_string()
    };
    output.push_str(&display_name);

    if let Some(n) = note {
        if for_display {
            output.push_str(&format!("  {}", format!("# {}", n).bright_black()));
        } else {
            output.push_str(&format!("  # {}", n));
        }
    }
    output.push('\n');

    output
}
