//! lsnote - ls with notes
//!
//! A modern `ls` replacement with file notes, emoji icons, and git integration.

mod config;
mod display;
mod git;
mod icons;
mod notes;

use std::fs;
use std::path::{Path, PathBuf};

use arboard::Clipboard;
use clap::Parser;

use config::init_config;
use display::{build_tree, list_directory, print_tree, DisplayOptions};
use notes::{get_note, remove_note, set_note};

#[derive(Parser, Debug)]
#[command(name = "lsnote")]
#[command(about = "ls with notes - list directory contents with file notes")]
#[command(version)]
struct Args {
    /// Directory or file to list
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Show all files including hidden
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Use long listing format
    #[arg(short = 'l', long = "long")]
    long: bool,

    /// Set note for a file
    #[arg(short = 's', long = "set", value_names = ["FILE", "NOTE"], num_args = 2)]
    set: Option<Vec<String>>,

    /// Get note for a file
    #[arg(short = 'g', long = "get")]
    get: Option<PathBuf>,

    /// Remove note from a file
    #[arg(short = 'r', long = "remove")]
    remove: Option<PathBuf>,

    /// Human-readable sizes (e.g., 1K, 234M, 2G)
    #[arg(short = 'H', long = "human-readable")]
    human_readable: bool,

    /// Disable git status indicators
    #[arg(long = "no-git")]
    no_git: bool,

    /// Tree view - show directory structure
    #[arg(short = 't', long = "tree")]
    tree: bool,

    /// Disable icons
    #[arg(long = "no-icons")]
    no_icons: bool,

    /// Generate default config file at ~/.lsnote/config
    #[arg(long = "init-config")]
    init_config: bool,

    /// Copy output to clipboard (use with -t for tree, -l for long format, etc.)
    #[arg(short = 'c', long = "copy")]
    copy: bool,
}

fn main() {
    let args = Args::parse();

    // Handle init-config
    if args.init_config {
        match init_config() {
            Ok(path) => println!("Config file created at: {}", path.display()),
            Err(e) => eprintln!("Error creating config: {}", e),
        }
        return;
    }

    // Handle set note
    if let Some(set_args) = &args.set {
        let file = &set_args[0];
        let note = &set_args[1];
        let path = Path::new(file);

        match set_note(path, note) {
            Ok(_) => println!("Note set for '{}'", file),
            Err(e) => eprintln!("Error setting note: {}", e),
        }
        return;
    }

    // Handle get note
    if let Some(file) = &args.get {
        match get_note(file) {
            Some(note) => println!("{}", note),
            None => println!("No note set for '{}'", file.display()),
        }
        return;
    }

    // Handle remove note
    if let Some(file) = &args.remove {
        match remove_note(file) {
            Ok(_) => println!("Note removed from '{}'", file.display()),
            Err(e) => eprintln!("Error removing note: {}", e),
        }
        return;
    }

    // List directory
    let show_icons = !args.no_icons;
    let opts = DisplayOptions {
        show_all: args.all,
        long_format: args.long,
        show_icons,
        human_readable: args.human_readable,
        show_git: !args.no_git,
        tree_view: args.tree,
    };

    if args.copy {
        // Build output with colors for display, without colors for clipboard
        let (display_output, clipboard_output) = if args.tree {
            (
                build_tree(&args.path, &opts, "", true),
                build_tree(&args.path, &opts, "", false),
            )
        } else {
            (
                display::build_list(&args.path, &opts, true),
                display::build_list(&args.path, &opts, false),
            )
        };

        // Print the colored version
        print!("{}", display_output);

        // Copy plain text version to clipboard
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&clipboard_output) {
                    eprintln!("Failed to copy to clipboard: {}", e);
                } else {
                    eprintln!("Copied to clipboard!");
                }
            }
            Err(e) => eprintln!("Failed to access clipboard: {}", e),
        }
    } else if args.tree {
        print_tree(&args.path, &opts, "", true);
    } else {
        list_directory(&args.path, &opts);
    }
}

/// Get the data directory: ~/.lsnote/
pub fn get_data_dir() -> Result<PathBuf, String> {
    let data_dir = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".lsnote");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }

    Ok(data_dir)
}
