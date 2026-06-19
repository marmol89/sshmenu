# sshmenu

```
┌───────────────────────────────┐ ┌───────────────────────────────┐
│ sshmenu                       │ │ Details                       │
├───────────────────────────────┤ ├───────────────────────────────┤
│ Name   │ SSH                  │ │ prod-server-01                │
├───────────────────────────────┤ │                               │
│ * prod-1 │ root@10.0.0.1:22   │ │ Basic                         │
│   dev-2 │ forge@10.0.0.2:22   │ │   Host:     10.0.0.1          │
│   stage │ user@10.0.0.3:2222  │ │   User:     root              │
├───────────────────────────────┤ │   Port:     22                │
│ 3 hosts | name↑               │ │   Tags:     [prod, web]       │
└───────────────────────────────┘ │   Last SSH: 2h ago            │
                                  │   Connects: 42                │
                                  └───────────────────────────────┘
  [a] add  [e] edit  [d] del  [g] ping  [p] pin  [c] copy  [s] sort  [t] theme  [q] quit
```

A terminal-based, interactive SSH connection manager for your terminal.
Fast, keyboard-driven, and pretty. Built with Rust + ratatui.
Inspired by [lazyssh](https://github.com/adembc/lazyssh) and the spirit of `lazydocker`/`k9s`.

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.74%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue?style=for-the-badge)](LICENSE)
[![Release](https://img.shields.io/github/v/release/marmol89/sshmenu?style=for-the-badge)](https://github.com/marmol89/sshmenu/releases)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macos-success?style=for-the-badge)](#installation)
[![No deps](https://img.shields.io/badge/install-no%20rust%20needed-brightgreen?style=for-the-badge)](#installation)

</div>

## Why sshmenu?

Stop memorizing IPs, ports, and usernames. Stop typing the same `ssh -p 2222 user@host` over and over.
Open one TUI, find your server, hit Enter. Done.

- ⚡ **Instant** — single static binary, no dependencies, starts in milliseconds
- 🎨 **Pretty** — 5 built-in themes, side-by-side details panel, clean column layout
- 🧠 **Smart** — tracks when you last connected, sorts by recency, fuzzy search across name/host/tags/user
- 🪶 **Lightweight** — ~1.3 MB compiled, zero runtime deps
- ⌨️ **Keyboard-first** — Vim-style navigation, every action one keystroke away

## Features

<table>
<tr><td>📜</td><td><b>Host management</b></td><td>Add, edit, delete hosts from the UI. Stored as JSON, easy to version-control.</td></tr>
<tr><td>🔍</td><td><b>Fuzzy search</b></td><td>Press <code>/</code> and search by name, host, user, or any tag.</td></tr>
<tr><td>📌</td><td><b>Pin favorites</b></td><td>Pinned hosts always sit at the top of the list.</td></tr>
<tr><td>🏓</td><td><b>Ping before connecting</b></td><td>Verify reachability with <code>g</code> before opening an SSH session.</td></tr>
<tr><td>📋</td><td><b>Copy SSH command</b></td><td>One keystroke (<code>c</code>) copies the full <code>ssh</code> command to your clipboard.</td></tr>
<tr><td>📊</td><td><b>Sort</b></td><td>Toggle sort by name or last connection time (<code>s</code>), reverse order (<code>S</code>).</td></tr>
<tr><td>⏱</td><td><b>Connection history</b></td><td>Each host remembers when you last connected and how many times.</td></tr>
<tr><td>🪟</td><td><b>Details panel</b></td><td>Side-by-side panel with everything you need to know about the selected host.</td></tr>
<tr><td>🎨</td><td><b>5 themes</b></td><td>default, dracula, nord, monokai, light — switch live and pick what fits.</td></tr>
<tr><td>🛡</td><td><b>Safe by default</b></td><td>Confirmation prompt before deleting, atomic config writes, no external SSH agent magic.</td></tr>
</table>

## Installation

### 🚀 Quick install (no Rust required)

```bash
curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```

This downloads the latest release for your platform and installs it to `~/.local/bin/sshmenu`.
Works on Linux and macOS, both `x86_64` and `aarch64`.

<details>
<summary><b>Custom install location</b></summary>

```bash
SSHMENU_INSTALL_DIR=/usr/local/bin curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```
</details>

<details>
<summary><b>Install a specific version</b></summary>

```bash
SSHMENU_VERSION=0.1.0 curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```
</details>

<details>
<summary><b>Update to latest</b></summary>

Just re-run the install script. It overwrites the existing binary.

```bash
curl -fsSL https://raw.githubusercontent.com/marmol89/sshmenu/main/install.sh | bash
```
</details>

<details>
<summary><b>Uninstall</b></summary>

```bash
rm ~/.local/bin/sshmenu
```
</details>

### 🔧 Build from source

For hackers and contributors. Requires the [Rust toolchain](https://rustup.rs/) (1.74+).

```bash
git clone https://github.com/marmol89/sshmenu.git
cd sshmenu
make install
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

That's it. Use arrow keys (or `j`/`k`) to navigate, `Enter` to connect.

### Keybindings

#### Navigation

| Key | Action |
|:---:|--------|
| `↑` / `↓` | Move selection |
| `j` / `k` | Move selection (vim-style) |
| `Enter` | **Connect to selected host** |
| `q` / `Esc` | Quit |

#### Host management

| Key | Action |
|:---:|--------|
| `a` | Add new host |
| `e` | Edit selected host |
| `d` | Delete selected host (with confirmation) |
| `/` | Search/filter |

#### Power features

| Key | Action |
|:---:|--------|
| `g` | Ping selected host |
| `p` | Pin / unpin selected host |
| `c` | Copy SSH command to clipboard |
| `s` | Toggle sort field (name ↔ last seen) |
| `S` | Reverse sort order |
| `t` | Open theme selector |

## Configuration

Everything lives in `~/.config/sshmenu/`:

```
~/.config/sshmenu/
├── hosts.json    # your saved hosts
└── config.json   # UI settings (theme, sort, etc.)
```

Both files are plain JSON — feel free to edit them directly or version-control them in a dotfiles repo.

### Example `hosts.json`

```json
[
  {
    "name": "prod-server-01",
    "host": "10.0.0.1",
    "port": 22,
    "user": "root",
    "tags": ["prod", "web"],
    "pinned": true,
    "last_seen": 1718800000,
    "ssh_count": 42
  },
  {
    "name": "dev-laptop",
    "host": "192.168.1.50",
    "port": 2222,
    "user": "forge",
    "tags": ["dev", "personal"]
  }
]
```

> 💡 `pinned`, `last_seen`, and `ssh_count` are auto-managed by sshmenu — you don't need to set them by hand.

### Custom config directory

```bash
SSHMENU_DIR=/path/to/dir sshmenu
```

## Themes

Press `t` to open the theme selector. Each theme name in the selector renders in its **own colors**, so you can preview before committing.

| Theme | Vibe |
|-------|------|
| `default` | Classic cyan/blue on dark — safe and readable |
| `dracula` | Purple accents on dark grey — popular and easy on the eyes |
| `nord` | Arctic blue tones — calm and focused |
| `monokai` | Green/orange syntax colors — bold and energetic |
| `light` | Bright theme for daylight / well-lit rooms |

Your theme choice is saved to `~/.config/sshmenu/config.json` and restored on next launch.

## How it works

sshmenu is a thin TUI wrapper around your regular `ssh` command. It doesn't manage keys, doesn't touch your `~/.ssh/config`, doesn't inject any custom SSH logic.

When you hit `Enter`, it:
1. Exits the alternate screen
2. Runs `ssh -p <port> <user>@<host>` as a child process
3. Returns you to the TUI when the SSH session ends

Your existing SSH config, keys, agents, and `~/.ssh/known_hosts` all work exactly as if you'd typed the command yourself.

## Contributing

Issues, PRs, and theme requests are welcome. If you build a cool new theme, send a PR — I'll happily merge it.

```bash
# Dev workflow
make build       # debug build
make release     # optimized build
make run         # build and run
make clean       # clean artifacts
```

## License

[Apache-2.0](LICENSE)

---

<div align="center">
<sub>Built with 🦀 by <a href="https://github.com/marmol89">marmol89</a></sub>
</div>