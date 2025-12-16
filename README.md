# lsnote

A modern `ls` replacement with file notes, emoji icons, and git integration.

## Features

- **File notes** - Add notes to any file or directory
- **Emoji icons** - Visual file type indicators (no nerd fonts needed)
- **Git integration** - Status indicators and colored filenames
- **Tree view** - Recursive directory display
- **Human-readable sizes** - 1.2K, 3.4M, 2.1G
- **Customizable** - Configure icons, colors, and symbols

## Installation

```bash
cargo install lsnote
```

Or build manually:

```bash
cargo build --release
cp target/release/lsnote ~/.local/bin/
```

## Usage

```bash
lsnote                     # List current directory
lsnote -l                  # Long listing format
lsnote -a                  # Show hidden files
lsnote -t                  # Tree view
lsnote -H                  # Human-readable sizes
lsnote --no-git            # Disable git status
lsnote --no-icons          # Disable icons
```

### Notes

```bash
lsnote -s FILE "note"      # Set note
lsnote -g FILE             # Get note
lsnote -r FILE             # Remove note
```

Notes are stored in `~/.lsn/notes`.

## Configuration

Generate a config file:

```bash
lsnote --init-config
```

Edit `~/.lsn/config`:

```
# Icons
icon.directory = 📁
icon.symlink = 🔗
icon.file = 📄

# Extension icons
icon.ext.rs = 🦀
icon.ext.py = 🐍
icon.ext.go = 🐹

# Filename icons
icon.name.Dockerfile = 🐳
icon.name.Makefile = 🔨

# Colors (black, red, green, yellow, blue, magenta, cyan, white)
color.directory = blue
color.git_modified = red
color.git_staged = green
color.git_untracked = yellow

# Git symbols
git.modified = ●
git.staged = ◐
git.untracked = ?
```

## Git Status

Git integration is enabled by default. Files and directories are colored by status:

| Status | Symbol | Color |
|--------|--------|-------|
| Modified | ● | Red |
| Staged | ◐ | Green |
| Untracked | ? | Yellow |

Directories inherit the highest-priority status from their contents.

## License

MIT

Made with <3 by [@jofftiquez](https://github.com/jofftiquez)
