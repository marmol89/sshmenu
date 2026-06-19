use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::Backend;
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use crate::theme;
use crate::ui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub last_seen: Option<i64>,
    #[serde(default)]
    pub ssh_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Name,
    LastSeen,
}

impl Default for SortField {
    fn default() -> Self { SortField::Name }
}

#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub name: String,
    pub host_val: String,
    pub port: String,
    pub user: String,
    pub tags: String,
    pub focus: usize,
    pub editing_index: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Searching,
    Adding,
    Editing,
    Deleting(usize),
    Message(String),
    ThemeSelect,
}

#[derive(Debug, Clone)]
pub struct App {
    pub hosts: Vec<Host>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub mode: Mode,
    pub search: String,
    pub form: FormState,
    pub theme_index: usize,
    pub sort_field: SortField,
    pub sort_reverse: bool,
    pub quitting: bool,
    hosts_path: PathBuf,
    config_path: PathBuf,
}

impl Default for App {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            mode: Mode::default(),
            search: String::new(),
            form: FormState::default(),
            theme_index: 0,
            sort_field: SortField::Name,
            sort_reverse: false,
            quitting: false,
            hosts_path: PathBuf::new(),
            config_path: PathBuf::new(),
        }
    }
}

impl App {
    pub fn load() -> Result<Self> {
        let base = config_dir();
        std::fs::create_dir_all(&base)?;

        let hosts_path = base.join("hosts.json");
        let hosts: Vec<Host> = if hosts_path.exists() {
            let content = std::fs::read_to_string(&hosts_path)
                .context("Failed to read hosts.json")?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::fs::write(&hosts_path, "[]")?;
            Vec::new()
        };

        let config_path = base.join("config.json");
        let theme_index = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            #[derive(Deserialize)]
            struct Config { theme: Option<String> }
            if let Ok(cfg) = serde_json::from_str::<Config>(&content) {
                if let Some(name) = cfg.theme {
                    theme::THEMES.iter().position(|t| t.name == name).unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        let mut app = App {
            hosts_path,
            config_path,
            theme_index,
            ..Default::default()
        };
        app.hosts = hosts;
        app.update_filter();
        Ok(app)
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.hosts)?;
        std::fs::write(&self.hosts_path, content)?;
        Ok(())
    }

    fn save_theme(&self) -> Result<()> {
        let cfg = serde_json::json!({ "theme": theme::THEMES[self.theme_index].name });
        std::fs::write(&self.config_path, serde_json::to_string_pretty(&cfg)?)?;
        Ok(())
    }

    pub fn update_filter(&mut self) {
        let q = self.search.to_lowercase();
        let mut idxs: Vec<usize> = self.hosts.iter().enumerate()
            .filter(|(_, h)| {
                q.is_empty()
                    || h.name.to_lowercase().contains(&q)
                    || h.host.contains(&q)
                    || h.user.to_lowercase().contains(&q)
                    || h.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect();

        idxs.sort_by(|&a, &b| {
            let ha = &self.hosts[a];
            let hb = &self.hosts[b];
            let ord = match self.sort_field {
                SortField::Name => ha.name.to_lowercase().cmp(&hb.name.to_lowercase()),
                SortField::LastSeen => hb.last_seen.unwrap_or(0).cmp(&ha.last_seen.unwrap_or(0)),
            };
            if self.sort_reverse { ord.reverse() } else { ord }
        });

        // Pinned always on top, then sorted
        let (mut pinned, rest): (Vec<_>, Vec<_>) = idxs
            .into_iter()
            .partition(|&i| self.hosts[i].pinned);
        pinned.sort_by(|&a, &b| {
            let ha = &self.hosts[a];
            let hb = &self.hosts[b];
            ha.name.to_lowercase().cmp(&hb.name.to_lowercase())
        });
        pinned.extend(rest);
        self.filtered = pinned;

        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }

    pub fn selected_host(&self) -> Option<&Host> {
        self.filtered.get(self.selected).map(|&i| &self.hosts[i])
    }

    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        loop {
            terminal.draw(|f| ui::render(f, &self))?;

            if self.quitting {
                return Ok(());
            }

            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if self.mode == Mode::Normal && key.code == KeyCode::Enter {
                    if let Some(&idx) = self.filtered.get(self.selected) {
                        let host = self.hosts[idx].clone();
                        self.hosts[idx].last_seen = Some(std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64);
                        self.hosts[idx].ssh_count += 1;
                        let _ = self.save();
                        ssh_connect(&host);
                        terminal.clear()?;
                        continue;
                    }
                }
                if self.handle_key(key)? {
                    return Ok(());
                }
            }
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        Ok(match self.mode.clone() {
            Mode::Normal => { self.handle_normal(key); false }
            Mode::Searching => { self.handle_search(key); false }
            Mode::Adding | Mode::Editing => { self.handle_form(key)?; false }
            Mode::Deleting(idx) => { self.handle_delete(key, idx)?; false }
            Mode::ThemeSelect => { self.handle_theme_select(key)?; false }
            Mode::Message(_) => { self.mode = Mode::Normal; false }
        })
    }

    fn handle_normal(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => { self.quitting = true; }
            KeyCode::Char('a') => {
                self.form = FormState::default();
                self.form.port = String::from("22");
                self.form.user = String::from("root");
                self.mode = Mode::Adding;
            }
            KeyCode::Char('e') => {
                if let Some(&idx) = self.filtered.get(self.selected) {
                    let h = &self.hosts[idx];
                    self.form = FormState {
                        name: h.name.clone(),
                        host_val: h.host.clone(),
                        port: h.port.to_string(),
                        user: h.user.clone(),
                        tags: h.tags.join(", "),
                        focus: 0,
                        editing_index: idx,
                    };
                    self.mode = Mode::Editing;
                }
            }
            KeyCode::Char('d') => {
                if let Some(&idx) = self.filtered.get(self.selected) {
                    self.mode = Mode::Deleting(idx);
                }
            }
            KeyCode::Char('g') => {
                if let Some(host) = self.selected_host() {
                    self.ping_host(host);
                }
            }
            KeyCode::Char('p') => {
                if let Some(&idx) = self.filtered.get(self.selected) {
                    self.hosts[idx].pinned = !self.hosts[idx].pinned;
                    let _ = self.save();
                    self.update_filter();
                }
            }
            KeyCode::Char('c') => {
                if let Some(host) = self.selected_host() {
                    let cmd = host_ssh_cmd(host);
                    let _ = copy_to_clipboard(&cmd);
                    self.mode = Mode::Message(format!("Copied: {}", cmd));
                }
            }
            KeyCode::Char('s') => {
                self.sort_field = match self.sort_field {
                    SortField::Name => SortField::LastSeen,
                    SortField::LastSeen => SortField::Name,
                };
                self.sort_reverse = false;
                self.update_filter();
            }
            KeyCode::Char('S') => {
                self.sort_reverse = !self.sort_reverse;
                self.update_filter();
            }
            KeyCode::Char('/') => {
                self.search.clear();
                self.mode = Mode::Searching;
            }
            KeyCode::Char('t') => {
                self.mode = Mode::ThemeSelect;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_search(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search.clear();
                self.mode = Mode::Normal;
                self.update_filter();
            }
            KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                self.search.pop();
                self.update_filter();
            }
            KeyCode::Char(c) => {
                self.search.push(c);
                self.update_filter();
            }
            _ => {}
        }
    }

    fn handle_form(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            KeyCode::Enter => {
                if self.form.name.is_empty() || self.form.host_val.is_empty() {
                    self.mode = Mode::Message("Name and host are required".into());
                    return Ok(());
                }
                let port: u16 = self.form.port.parse().unwrap_or(22);
                let tags: Vec<String> = self.form.tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let host = Host {
                    name: self.form.name.trim().to_string(),
                    host: self.form.host_val.trim().to_string(),
                    port,
                    user: if self.form.user.is_empty() { "root".into() } else { self.form.user.trim().to_string() },
                    tags,
                    pinned: false,
                    last_seen: None,
                    ssh_count: 0,
                };

                if self.mode == Mode::Adding {
                    self.hosts.push(host);
                } else {
                    let idx = self.form.editing_index;
                    if idx < self.hosts.len() {
                        self.hosts[idx] = host;
                    }
                }
                self.save()?;
                self.update_filter();
                self.mode = Mode::Normal;
            }
            KeyCode::Tab => {
                self.form.focus = (self.form.focus + 1) % 5;
            }
            KeyCode::BackTab => {
                self.form.focus = (self.form.focus + 4) % 5;
            }
            KeyCode::Backspace => {
                let field = match self.form.focus {
                    0 => &mut self.form.name,
                    1 => &mut self.form.host_val,
                    2 => &mut self.form.port,
                    3 => &mut self.form.user,
                    _ => &mut self.form.tags,
                };
                field.pop();
            }
            KeyCode::Char(c) => {
                let field = match self.form.focus {
                    0 => &mut self.form.name,
                    1 => &mut self.form.host_val,
                    2 => &mut self.form.port,
                    3 => &mut self.form.user,
                    _ => &mut self.form.tags,
                };
                field.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_delete(&mut self, key: crossterm::event::KeyEvent, idx: usize) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if idx < self.hosts.len() {
                    self.hosts.remove(idx);
                    self.save()?;
                    self.update_filter();
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn ping_host(&self, host: &Host) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = io::stdout().flush();

        print!("Pinging {} ({})... ", host.name, host.host);
        let _ = io::stdout().flush();

        let status = Command::new("ping")
            .args(["-c", "1", "-W", "3", &host.host])
            .status();

        match status {
            Ok(s) if s.success() => println!("OK"),
            _ => println!("FAILED"),
        }

        print!("Press Enter to continue...");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut String::new());

        let _ = enable_raw_mode();
        let _ = execute!(io::stdout(), EnterAlternateScreen);
    }

    fn handle_theme_select(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.theme_index > 0 {
                    self.theme_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.theme_index + 1 < theme::THEMES.len() {
                    self.theme_index += 1;
                }
            }
            KeyCode::Enter => {
                self.save_theme()?;
                self.mode = Mode::Normal;
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }
}

fn ssh_connect(host: &Host) {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = io::stdout().flush();

    println!("\nConnecting to {}@{}:{} ...\n", host.user, host.host, host.port);

    let status = Command::new("ssh")
        .args(["-p", &host.port.to_string(), &format!("{}@{}", host.user, host.host)])
        .status();

    match status {
        Ok(s) if !s.success() => {
            println!("\nSSH exited with code: {:?}", s.code());
            print!("Press Enter to continue...");
            let _ = io::stdout().flush();
            let _ = io::stdin().read_line(&mut String::new());
        }
        Err(e) => {
            println!("\nFailed to run SSH: {}", e);
            print!("Press Enter to continue...");
            let _ = io::stdout().flush();
            let _ = io::stdin().read_line(&mut String::new());
        }
        _ => {}
    }

    let _ = io::stdout().flush();
    let _ = enable_raw_mode();
    let _ = execute!(io::stdout(), EnterAlternateScreen);
}

fn host_ssh_cmd(host: &Host) -> String {
    format!("ssh -p {} {}@{}", host.port, host.user, host.host)
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::process::{Command, Stdio};
    for prog in ["xclip", "wl-copy", "xsel"] {
        let r = match prog {
            "xclip" => Command::new("xclip").args(["-selection", "clipboard"]).stdin(Stdio::piped()).spawn(),
            "wl-copy" => Command::new("wl-copy").stdin(Stdio::piped()).spawn(),
            "xsel" => Command::new("xsel").args(["-i", "-b"]).stdin(Stdio::piped()).spawn(),
            _ => continue,
        };
        if let Ok(mut child) = r {
            use std::io::Write;
            let _ = child.stdin.as_mut().map(|s| s.write_all(text.as_bytes()));
            let _ = child.wait();
            return Ok(());
        }
    }
    Ok(())
}

fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SSHMENU_DIR") {
        return PathBuf::from(dir);
    }
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    base.join("sshmenu")
}
