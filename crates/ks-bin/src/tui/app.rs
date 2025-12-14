use anyhow::Result;
use ks_lib::config::SinkConfig;
use ratatui::widgets::ListState;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub struct App {
    pub items: Vec<MenuItem>,
    pub state: ListState,
    pub running: bool,
    pub sys: System,
    pub config: SinkConfig,
    pub last_tick: Instant,
}

pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    #[allow(dead_code)]
    pub description: String,
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    ToggleService,
    EditConfig,
    Quit,
}

impl App {
    pub fn new(config: SinkConfig) -> App {
        let items = vec![
            MenuItem {
                label: "Toggle Kitchnsink".to_string(),
                action: MenuAction::ToggleService,
                description: "Start or Stop the kitchnsink bar".to_string(),
            },
            MenuItem {
                label: "Edit Config".to_string(),
                action: MenuAction::EditConfig,
                description: "Open sink.toml in editor".to_string(),
            },
            MenuItem {
                label: "Quit".to_string(),
                action: MenuAction::Quit,
                description: "Exit the manager".to_string(),
            },
        ];
        let mut app = Self {
            items,
            state: ListState::default(),
            running: false,
            sys: System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            ),
            config,
            last_tick: Instant::now(),
        };
        app.state.select(Some(0));
        app.check_process();
        app
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn check_process(&mut self) {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        let my_pid = std::process::id();

        let found = self.sys.processes().values().find(|p| {
            let name = p.name();
            // Filter zombie processes (status 4 seems to be Zombie in some versions, but let's use the update)
            // sysinfo 0.37 usages: p.status() returns ProcessStatus
            let status = p.status();
            // Filter out Zombies (essential because we are the parent and might not have reaped them)
            if matches!(status, sysinfo::ProcessStatus::Zombie) {
                return false;
            }

            (name == "kitchnsink" || name == "ks-bin")
                && p.pid().as_u32() != my_pid
                && !p
                    .cmd()
                    .iter()
                    .any(|arg| arg == "manage" || arg == "m" || arg == "internal-watch")
        });

        self.running = found.is_some();
    }

    pub fn execute_selected(&mut self) -> Result<bool> {
        let selected = self.state.selected().unwrap_or(0);
        let action = self.items[selected].action;
        let self_exe = std::env::current_exe()?;

        match action {
            MenuAction::ToggleService => {
                if self.running {
                    // Use CLI stop command
                    Command::new(&self_exe)
                        .arg("--stop")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?
                        .wait()?;
                } else {
                    // Use CLI start command
                    Command::new(&self_exe)
                        .arg("--start")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;
                }
                // Increase wait time to allow pid file creation/deletion
                std::thread::sleep(Duration::from_millis(500));
                self.check_process();
                Ok(false)
            }
            MenuAction::EditConfig => {
                // 1. Resolve Editor
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

                // 2. Resolve Config Path
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let config_path = std::path::PathBuf::from(home)
                    .join(".config")
                    .join("kitchnsink")
                    .join("sink.toml");

                // 3. Detect Terminal
                let terminal = std::env::var("TERMINAL").ok().or_else(|| {
                    let terminals = ["rio", "alacritty", "kitty", "gnome-terminal", "xterm"];
                    for term in terminals {
                        if which::which(term).is_ok() {
                            return Some(term.to_string());
                        }
                    }
                    None
                });

                // 4. Spawn
                if let Some(term) = terminal {
                    Command::new(term)
                        .arg("-e")
                        .arg(&editor)
                        .arg(config_path)
                        .spawn()?;
                }

                Ok(false)
            }
            MenuAction::Quit => Ok(true),
        }
    }
}
