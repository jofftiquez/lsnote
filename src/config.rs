//! Configuration management for lsn.
//!
//! Loads user configuration from `~/.lsn/config` using a simple key=value format.

use colored::Color;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::get_data_dir;

const CONFIG_FILE: &str = "config";

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Returns a reference to the global configuration.
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| Config::load().unwrap_or_default())
}

/// Main configuration structure.
#[derive(Debug, Clone)]
pub struct Config {
    pub icons: IconsConfig,
    pub colors: ColorsConfig,
    pub git: GitConfig,
}

/// Icon configuration for different file types.
#[derive(Debug, Clone)]
pub struct IconsConfig {
    pub directory: String,
    pub symlink: String,
    pub file: String,
    pub executable: String,
    pub extensions: HashMap<String, String>,
    pub filenames: HashMap<String, String>,
}

/// Color configuration for output.
#[derive(Debug, Clone)]
pub struct ColorsConfig {
    pub directory: String,
    pub symlink: String,
    pub executable: String,
    pub file: String,
    pub git_modified: String,
    pub git_staged: String,
    pub git_untracked: String,
}

/// Git status symbol configuration.
#[derive(Debug, Clone)]
pub struct GitConfig {
    pub modified: String,
    pub staged: String,
    pub untracked: String,
    pub ignored: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            icons: IconsConfig::default(),
            colors: ColorsConfig::default(),
            git: GitConfig::default(),
        }
    }
}

impl Default for IconsConfig {
    fn default() -> Self {
        let mut extensions = HashMap::new();
        // Rust
        extensions.insert("rs".into(), "🦀".into());
        // Config files
        extensions.insert("toml".into(), "⚙️".into());
        extensions.insert("cfg".into(), "⚙️".into());
        extensions.insert("conf".into(), "⚙️".into());
        extensions.insert("ini".into(), "⚙️".into());
        extensions.insert("config".into(), "⚙️".into());
        // Markdown
        extensions.insert("md".into(), "📝".into());
        extensions.insert("markdown".into(), "📝".into());
        // Data formats
        extensions.insert("json".into(), "📋".into());
        extensions.insert("yaml".into(), "📋".into());
        extensions.insert("yml".into(), "📋".into());
        // JavaScript
        extensions.insert("js".into(), "🟨".into());
        extensions.insert("mjs".into(), "🟨".into());
        extensions.insert("cjs".into(), "🟨".into());
        extensions.insert("jsx".into(), "🟨".into());
        // TypeScript
        extensions.insert("ts".into(), "🔷".into());
        extensions.insert("tsx".into(), "🔷".into());
        extensions.insert("mts".into(), "🔷".into());
        extensions.insert("cts".into(), "🔷".into());
        // Python
        extensions.insert("py".into(), "🐍".into());
        extensions.insert("pyi".into(), "🐍".into());
        extensions.insert("pyc".into(), "🐍".into());
        // Go
        extensions.insert("go".into(), "🐹".into());
        // Lock files
        extensions.insert("lock".into(), "🔒".into());
        // Shell
        extensions.insert("sh".into(), "💻".into());
        extensions.insert("bash".into(), "💻".into());
        extensions.insert("zsh".into(), "💻".into());
        extensions.insert("fish".into(), "💻".into());
        // Images
        extensions.insert("png".into(), "🖼️".into());
        extensions.insert("jpg".into(), "🖼️".into());
        extensions.insert("jpeg".into(), "🖼️".into());
        extensions.insert("gif".into(), "🖼️".into());
        extensions.insert("svg".into(), "🖼️".into());
        extensions.insert("ico".into(), "🖼️".into());
        extensions.insert("webp".into(), "🖼️".into());
        // Video
        extensions.insert("mp4".into(), "🎬".into());
        extensions.insert("mkv".into(), "🎬".into());
        extensions.insert("avi".into(), "🎬".into());
        extensions.insert("mov".into(), "🎬".into());
        extensions.insert("webm".into(), "🎬".into());
        // Audio
        extensions.insert("mp3".into(), "🎵".into());
        extensions.insert("wav".into(), "🎵".into());
        extensions.insert("flac".into(), "🎵".into());
        extensions.insert("ogg".into(), "🎵".into());
        extensions.insert("m4a".into(), "🎵".into());
        // Archives
        extensions.insert("zip".into(), "📦".into());
        extensions.insert("tar".into(), "📦".into());
        extensions.insert("gz".into(), "📦".into());
        extensions.insert("bz2".into(), "📦".into());
        extensions.insert("xz".into(), "📦".into());
        extensions.insert("7z".into(), "📦".into());
        extensions.insert("rar".into(), "📦".into());
        // Web
        extensions.insert("html".into(), "🌐".into());
        extensions.insert("htm".into(), "🌐".into());
        extensions.insert("css".into(), "🎨".into());
        extensions.insert("scss".into(), "🎨".into());
        extensions.insert("sass".into(), "🎨".into());
        extensions.insert("less".into(), "🎨".into());
        // Java
        extensions.insert("java".into(), "☕".into());
        extensions.insert("jar".into(), "☕".into());
        extensions.insert("class".into(), "☕".into());
        // C/C++
        extensions.insert("c".into(), "🔧".into());
        extensions.insert("h".into(), "🔧".into());
        extensions.insert("cpp".into(), "🔧".into());
        extensions.insert("cc".into(), "🔧".into());
        extensions.insert("cxx".into(), "🔧".into());
        extensions.insert("hpp".into(), "🔧".into());
        extensions.insert("hxx".into(), "🔧".into());
        // Database
        extensions.insert("sql".into(), "🗄️".into());
        extensions.insert("db".into(), "🗄️".into());
        extensions.insert("sqlite".into(), "🗄️".into());
        extensions.insert("sqlite3".into(), "🗄️".into());

        let mut filenames = HashMap::new();
        filenames.insert("cargo.toml".into(), "🦀".into());
        filenames.insert("cargo.lock".into(), "🦀".into());
        filenames.insert("makefile".into(), "🔨".into());
        filenames.insert("gnumakefile".into(), "🔨".into());
        filenames.insert("dockerfile".into(), "🐳".into());
        filenames.insert("containerfile".into(), "🐳".into());
        filenames.insert("license".into(), "📜".into());
        filenames.insert("license.md".into(), "📜".into());
        filenames.insert("license.txt".into(), "📜".into());
        filenames.insert("readme".into(), "📖".into());
        filenames.insert("readme.md".into(), "📖".into());
        filenames.insert("readme.txt".into(), "📖".into());
        filenames.insert(".gitignore".into(), "🙈".into());
        filenames.insert(".gitattributes".into(), "🙈".into());
        filenames.insert(".gitmodules".into(), "🙈".into());
        filenames.insert(".git".into(), "📦".into());
        filenames.insert(".env".into(), "⚙️".into());
        filenames.insert(".envrc".into(), "⚙️".into());

        Self {
            directory: "📁".into(),
            symlink: "🔗".into(),
            file: "📄".into(),
            executable: "📄".into(),
            extensions,
            filenames,
        }
    }
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            directory: "blue".into(),
            symlink: "cyan".into(),
            executable: "green".into(),
            file: "white".into(),
            git_modified: "red".into(),
            git_staged: "green".into(),
            git_untracked: "yellow".into(),
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            modified: "●".into(),
            staged: "◐".into(),
            untracked: "?".into(),
            ignored: "◌".into(),
        }
    }
}

impl Config {
    /// Load configuration from ~/.lsn/config.
    pub fn load() -> Result<Self, String> {
        let config_path = get_config_path()?;
        let mut config = Self::default();

        if !config_path.exists() {
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                config.set(key.trim(), value.trim());
            }
        }

        Ok(config)
    }

    fn set(&mut self, key: &str, value: &str) {
        match key {
            "icon.directory" => self.icons.directory = value.into(),
            "icon.symlink" => self.icons.symlink = value.into(),
            "icon.file" => self.icons.file = value.into(),
            "icon.executable" => self.icons.executable = value.into(),
            "color.directory" => self.colors.directory = value.into(),
            "color.symlink" => self.colors.symlink = value.into(),
            "color.executable" => self.colors.executable = value.into(),
            "color.file" => self.colors.file = value.into(),
            "color.git_modified" => self.colors.git_modified = value.into(),
            "color.git_staged" => self.colors.git_staged = value.into(),
            "color.git_untracked" => self.colors.git_untracked = value.into(),
            "git.modified" => self.git.modified = value.into(),
            "git.staged" => self.git.staged = value.into(),
            "git.untracked" => self.git.untracked = value.into(),
            "git.ignored" => self.git.ignored = value.into(),
            _ if key.starts_with("icon.ext.") => {
                let ext = &key[9..];
                self.icons.extensions.insert(ext.into(), value.into());
            }
            _ if key.starts_with("icon.name.") => {
                let name = &key[10..];
                self.icons.filenames.insert(name.to_lowercase(), value.into());
            }
            _ => {}
        }
    }

    /// Generate default configuration file content.
    pub fn generate_default() -> String {
        [
            "# lsn configuration file",
            "# Lines starting with # are comments",
            "# Format: key = value",
            "",
            "# Icons for file types",
            "icon.directory = 📁",
            "icon.symlink = 🔗",
            "icon.file = 📄",
            "icon.executable = 📄",
            "",
            "# Colors (black, red, green, yellow, blue, magenta, cyan, white)",
            "# Also: bright_black, bright_red, bright_green, etc.",
            "color.directory = blue",
            "color.symlink = cyan",
            "color.executable = green",
            "color.file = white",
            "color.git_modified = red",
            "color.git_staged = green",
            "color.git_untracked = yellow",
            "",
            "# Git status symbols",
            "git.modified = ●",
            "git.staged = ◐",
            "git.untracked = ?",
            "git.ignored = ◌",
            "",
            "# Extension icons (icon.ext.<extension> = <icon>)",
            "# icon.ext.rs = 🦀",
            "# icon.ext.py = 🐍",
            "# icon.ext.js = 🟨",
            "# icon.ext.ts = 🔷",
            "# icon.ext.go = 🐹",
            "",
            "# Filename icons (icon.name.<filename> = <icon>)",
            "# icon.name.Dockerfile = 🐳",
            "# icon.name.Makefile = 🔨",
            "# icon.name.README.md = 📖",
        ]
        .join("\n")
    }
}

fn get_config_path() -> Result<PathBuf, String> {
    Ok(get_data_dir()?.join(CONFIG_FILE))
}

/// Initialize a new config file with defaults.
pub fn init_config() -> Result<PathBuf, String> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        return Err(format!(
            "Config file already exists at: {}\nDelete it first if you want to regenerate.",
            config_path.display()
        ));
    }

    fs::write(&config_path, Config::generate_default())
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(config_path)
}

/// Parse a color name string into a Color enum.
pub fn parse_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" | "purple" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "bright_black" | "brightblack" => Color::BrightBlack,
        "bright_red" | "brightred" => Color::BrightRed,
        "bright_green" | "brightgreen" => Color::BrightGreen,
        "bright_yellow" | "brightyellow" => Color::BrightYellow,
        "bright_blue" | "brightblue" => Color::BrightBlue,
        "bright_magenta" | "brightmagenta" => Color::BrightMagenta,
        "bright_cyan" | "brightcyan" => Color::BrightCyan,
        "bright_white" | "brightwhite" => Color::BrightWhite,
        _ => Color::White,
    }
}
