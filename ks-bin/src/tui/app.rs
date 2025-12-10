use anyhow::Result;
use kitchn_lib::config::Cookbook;
use ratatui::widgets::ListState;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub struct App {
    pub items: Vec<MenuItem>,
    pub state: ListState,
    pub running: bool,
    pub cookbook: Cookbook,
    pub last_tick: Instant,
    pub sys: System,
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
    Quit,
}

impl App {
    pub fn new(cookbook: Cookbook) -> App {
        let mut app = App {
            items: vec![
                MenuItem {
                    label: "Toggle Kitchnsink".to_string(),
                    action: MenuAction::ToggleService,
                    description: "Start or Stop the kitchnsink bar".to_string(),
                },
                MenuItem {
                    label: "Quit".to_string(),
                    action: MenuAction::Quit,
                    description: "Exit the manager".to_string(),
                },
            ],
            state: ListState::default(),
            running: false,
            cookbook,
            last_tick: Instant::now(),
            sys: System::new_with_specifics(
                RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
            ),
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
        // Update process list
        // sysinfo 0.32+: refresh_processes(ProcessesToUpdate, bool)
        self.sys.refresh_processes(ProcessesToUpdate::All, true);

        // Simple check: Look for "kitchnsink" process
        let my_pid = std::process::id();
        self.running = self.sys.processes().values().any(|p| {
            let name = p.name();
            // Check for both binary names and potential truncations
            (name == "kitchnsink" || name == "ks-bin")
                && p.pid().as_u32() != my_pid
                // CMD check: cmd() returns &[String]
                // Filter out the manager itself
                && !p.cmd().iter().any(|arg| arg == "manage" || arg == "m")
        });
    }

    pub fn execute_selected(&mut self) -> Result<bool> {
        let selected = self.state.selected().unwrap_or(0);
        let action = self.items[selected].action;

        match action {
            MenuAction::ToggleService => {
                if self.running {
                    // Kill process
                    // Find and kill external kitchnsink
                    let my_pid = std::process::id();
                    for process in self.sys.processes().values() {
                        let name = process.name();
                        if (name == "kitchnsink" || name == "ks-bin")
                            && process.pid().as_u32() != my_pid
                            && !process
                                .cmd()
                                .iter()
                                .any(|arg| arg == "manage" || arg == "m")
                        {
                            process.kill();
                        }
                    }
                } else {
                    // Start process
                    let self_exe = std::env::current_exe()?;
                    Command::new(self_exe)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;
                }
                // Wait a bit for propogation
                std::thread::sleep(Duration::from_millis(200));
                self.check_process();
                Ok(false) // Don't quit
            }
            MenuAction::Quit => Ok(true), // Quit
        }
    }
}
