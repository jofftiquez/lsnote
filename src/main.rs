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

// Hidden argument for clipboard daemon mode
const CLIPBOARD_DAEMON_ARG: &str = "--__clipboard_daemon__";

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

    /// Use short listing format (disable long format)
    #[arg(short = 'S', long = "short")]
    short: bool,

    /// Use long listing format (default, kept for compatibility with -la)
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

    /// Show raw byte sizes instead of human-readable (e.g., 1K, 234M)
    #[arg(short = 'B', long = "bytes")]
    bytes: bool,

    /// Disable git status indicators
    #[arg(long = "no-git")]
    no_git: bool,

    /// Tree view - show directory structure
    #[arg(short = 't', long = "tree")]
    tree: bool,

    /// Disable icons
    #[arg(long = "no-icons")]
    no_icons: bool,

    /// Hide column headers in long format
    #[arg(long = "no-header")]
    no_header: bool,

    /// Generate default config file at ~/.lsnote/config
    #[arg(long = "init-config")]
    init_config: bool,

    /// Copy output to clipboard (use with -t for tree, -l for long format, etc.)
    #[arg(short = 'c', long = "copy")]
    copy: bool,
}

fn main() {
    // Check for hidden clipboard daemon mode
    let raw_args: Vec<String> = std::env::args().collect();
    if raw_args.len() >= 2 && raw_args[1] == CLIPBOARD_DAEMON_ARG {
        run_clipboard_daemon();
        return;
    }

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
    // -l overrides -S if both specified; long format is default
    let long_format = args.long || !args.short;
    let opts = DisplayOptions {
        show_all: args.all,
        long_format,
        show_icons,
        human_readable: !args.bytes,
        show_git: !args.no_git,
        tree_view: args.tree,
        show_header: !args.no_header,
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
        copy_to_clipboard(&clipboard_output);
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

/// Copy text to clipboard, handling Wayland's clipboard persistence issue
fn copy_to_clipboard(text: &str) {
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();

    if is_wayland {
        // On Wayland, spawn a daemon process that sets and holds the clipboard
        use std::io::Write;
        use std::process::{Command, Stdio};

        let exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to get executable path: {}", e);
                return;
            }
        };

        match Command::new(exe)
            .arg(CLIPBOARD_DAEMON_ARG)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    if stdin.write_all(text.as_bytes()).is_ok() {
                        drop(stdin); // Close stdin to signal end of input
                        eprintln!("Copied to clipboard!");
                    } else {
                        eprintln!("Failed to send data to clipboard daemon");
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to spawn clipboard daemon: {}", e);
            }
        }
    } else {
        // Non-Wayland: use arboard directly
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(text) {
                    eprintln!("Failed to copy to clipboard: {}", e);
                } else {
                    eprintln!("Copied to clipboard!");
                }
            }
            Err(e) => eprintln!("Failed to access clipboard: {}", e),
        }
    }
}

/// Run as a clipboard daemon - reads text from stdin, sets clipboard, holds for 60s
fn run_clipboard_daemon() {
    use std::io::Read;

    // Read the text from stdin
    let mut text = String::new();
    if std::io::stdin().read_to_string(&mut text).is_err() {
        std::process::exit(1);
    }

    // Detach from controlling terminal
    unsafe {
        libc::setsid();
    }

    // Set the clipboard and hold it
    if let Ok(mut clipboard) = Clipboard::new() {
        if clipboard.set_text(&text).is_ok() {
            // Keep process alive to serve clipboard requests (Wayland requirement)
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
}
