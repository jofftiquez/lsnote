//! Icon lookup for lsnote.
//!
//! Determines the appropriate emoji icon for a file based on:
//! 1. File type (directory, symlink)
//! 2. Special filename (Dockerfile, Makefile, etc.)
//! 3. File extension
//! 4. Executable status

use std::fs::Metadata;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::config::get_config;

/// Get the icon for a file based on its name and metadata.
pub fn get_icon(name: &str, metadata: &Metadata) -> String {
    let config = get_config();
    let icons = &config.icons;

    // Check file type first
    if metadata.is_dir() {
        return icons.directory.clone();
    }
    if metadata.file_type().is_symlink() {
        return icons.symlink.clone();
    }

    let name_lower = name.to_lowercase();

    // Check special filenames first (highest priority)
    if let Some(icon) = icons.filenames.get(&name_lower) {
        return icon.clone();
    }

    // Check extension
    if let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) {
        if let Some(icon) = icons.extensions.get(&ext.to_lowercase()) {
            return icon.clone();
        }
    }

    // Check if executable
    if is_executable(metadata) {
        return icons.executable.clone();
    }

    // Default file icon
    icons.file.clone()
}

/// Check if a file is executable.
pub fn is_executable(metadata: &Metadata) -> bool {
    let mode = metadata.permissions().mode();
    mode & 0o111 != 0
}
