//! Note storage for lsn.
//!
//! Notes are stored in `~/.lsn/notes` with the format:
//! `/full/path/to/file: note text`

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::get_data_dir;

const NOTES_FILE: &str = "notes";

/// Get the notes file path: ~/.lsn/notes
fn get_notes_path() -> Result<PathBuf, String> {
    Ok(get_data_dir()?.join(NOTES_FILE))
}

/// Load all notes from the notes file.
pub fn load_notes() -> HashMap<String, String> {
    let mut map = HashMap::new();

    let notes_path = match get_notes_path() {
        Ok(p) => p,
        Err(_) => return map,
    };

    if let Ok(file) = fs::File::open(notes_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if let Some((path, note)) = line.split_once(": ") {
                map.insert(path.to_string(), note.to_string());
            }
        }
    }

    map
}

/// Save all notes to the notes file.
pub fn save_notes(notes: &HashMap<String, String>) -> Result<(), String> {
    let notes_path = get_notes_path()?;
    let mut file = fs::File::create(&notes_path).map_err(|e| e.to_string())?;

    let mut entries: Vec<_> = notes.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    for (path, note) in entries {
        writeln!(file, "{}: {}", path, note).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Canonicalize a path to an absolute path string.
fn canonicalize_path(path: &Path) -> Result<String, String> {
    path.canonicalize()
        .map_err(|e| format!("Could not resolve path: {}", e))
        .map(|p| p.to_string_lossy().to_string())
}

/// Set a note for a file.
pub fn set_note(path: &Path, note: &str) -> Result<(), String> {
    let canonical = canonicalize_path(path)?;
    let mut notes = load_notes();
    notes.insert(canonical, note.to_string());
    save_notes(&notes)
}

/// Get the note for a file, if one exists.
pub fn get_note(path: &Path) -> Option<String> {
    let canonical = canonicalize_path(path).ok()?;
    let notes = load_notes();
    notes.get(&canonical).cloned()
}

/// Remove the note from a file.
pub fn remove_note(path: &Path) -> Result<(), String> {
    let canonical = canonicalize_path(path)?;
    let mut notes = load_notes();

    if notes.remove(&canonical).is_some() {
        save_notes(&notes)
    } else {
        Err("No note found".to_string())
    }
}
