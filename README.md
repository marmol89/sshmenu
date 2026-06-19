# sshmenu

A terminal-based, interactive SSH connection manager for your terminal.
Built with Rust + ratatui. Inspired by [lazyssh](https://github.com/adembc/lazyssh).

![sshmenu screenshot](docs/screenshot.png)

## Features

- 📜 Configurable list of SSH hosts stored in `~/.config/sshmenu/hosts.json`
- ➕ Add, ✏ edit, 🗑 delete hosts from the UI
- 🔍 Fuzzy search by name, host, user, or tags
- 📌 Pin your favorite hosts to the top
- 🏓 Ping a host before connecting (`g`)
- 📋 Copy the `ssh` command to your clipboard (`c`)
- 🔢 Sort by name or last connection time (`s`/`S`)
- 🎨 5 built-in color themes (default, dracula, nord, monokai, light)
- ⏱ Tracks `Last SSH` time and total connection count per host
- 🪟 Side-by-side **Details** panel with full host info
- ⌨️ Pure keyboard-driven — no mouse required

## Installation

### Option 1: Prebuilt binary (no Rust required)

```bash
curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```

This downloads the latest release for your platform (Linux/macOS, x86_64/aarch64) and installs it to `~/.local/bin/sshmenu`.

To install to a custom location:
```bash
SSHMENU_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```

To install a specific version:
```bash
SSHMENU_VERSION=0.1.0 curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```

### Option 2: Build from source

Requirements: [Rust toolchain](https://rustup.rs/) (1.74+)

```bash
git clone https://github.com/marmol89/sshmenu.git
cd sshmenu
make install         # builds release and installs to ~/.local/bin
```

Or manually:
```bash
cargo build --release
install -m 0755 target/release/sshmenu ~/.local/bin/sshmenu
```

## Usage

```bash
sshmenu
```

### Keybindings

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate hosts |
| `Enter` | Connect to selected host |
| `a` | Add new host |
| `e` | Edit selected host |
| `d` | Delete selected host |
| `/` | Search |
| `g` | Ping selected host |
| `p` | Pin/unpin selected host |
| `c` | Copy SSH command to clipboard |
| `s` | Toggle sort field (name ↔ last seen) |
| `S` | Reverse sort order |
| `t` | Open theme selector |
| `q` or `Esc` | Quit |

### Config

- Hosts: `~/.config/sshmenu/hosts.json`
- Settings: `~/.config/sshmenu/config.json`

Override location with `SSHMENU_DIR=/path/to/dir` environment variable.

### Adding a host

Press `a` to open the form. Fill in:
- **Name** — Display name (required)
- **Host** — Hostname or IP (required)
- **Port** — SSH port (defaults to 22)
- **User** — SSH user (defaults to `root`)
- **Tags** — Comma-separated tags for filtering

Press `Tab` to move between fields, `Enter` to save, `Esc` to cancel.

## Themes

Press `t` to open the theme selector. Each theme renders its own preview in the list.

Built-in themes:
- **default** — cyan/blue classic
- **dracula** — purple accents on dark grey
- **nord** — arctic blue tones
- **monokai** — green/orange syntax colors
- **light** — bright theme for daylight

Theme selection persists in `~/.config/sshmenu/config.json`.

## License

Apache-2.0