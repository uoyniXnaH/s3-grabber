use std::collections::BTreeSet;

use crate::action::{Action, Focus, WorkTab};

#[derive(Debug, Clone)]
pub struct SessionState {
    pub profile: String,
    pub region: String,
    pub bucket: String,
    pub path: String,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct BrowserItem {
    pub is_dir: bool,
    pub name: String,
    pub size: Option<u64>,
    pub modified: String,
}

#[derive(Debug, Clone)]
pub struct BrowserState {
    pub items: Vec<BrowserItem>,
    pub cursor: usize,
    pub selected: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct QueueState {
    pub done_files: u64,
    pub total_files: u64,
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub speed_mbps: f64,
    pub eta: String,
}

#[derive(Debug, Clone)]
pub struct ScriptState {
    pub command: String,
    pub last_result: String,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub focus: Focus,
    pub tab: WorkTab,
    pub show_help: bool,
    pub confirm_quit: bool,
}

#[derive(Debug, Clone)]
pub struct App {
    pub running: bool,
    pub session: SessionState,
    pub browser: BrowserState,
    pub queue: QueueState,
    pub script: ScriptState,
    pub ui: UiState,
    pub logs: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let items = vec![
            BrowserItem {
                is_dir: true,
                name: "logs/".to_string(),
                size: None,
                modified: "-".to_string(),
            },
            BrowserItem {
                is_dir: true,
                name: "reports/".to_string(),
                size: None,
                modified: "-".to_string(),
            },
            BrowserItem {
                is_dir: false,
                name: "README.txt".to_string(),
                size: Some(2_048),
                modified: "2026-03-20 10:00".to_string(),
            },
        ];

        Self {
            running: true,
            session: SessionState {
                profile: "default".to_string(),
                region: "ap-northeast-1".to_string(),
                bucket: "my-bucket".to_string(),
                path: "/".to_string(),
                mode: "Browse".to_string(),
            },
            browser: BrowserState {
                items,
                cursor: 0,
                selected: BTreeSet::new(),
            },
            queue: QueueState {
                done_files: 0,
                total_files: 0,
                done_bytes: 0,
                total_bytes: 0,
                speed_mbps: 0.0,
                eta: "--:--".to_string(),
            },
            script: ScriptState {
                command: "./post-process.sh".to_string(),
                last_result: "idle".to_string(),
            },
            ui: UiState {
                focus: Focus::Browser,
                tab: WorkTab::Selection,
                show_help: false,
                confirm_quit: false,
            },
            logs: vec!["INFO app initialized".to_string()],
        }
    }

    pub fn update(&mut self, action: Action) {
        if self.ui.show_help {
            match action {
                Action::ToggleHelp | Action::CancelDialog => self.ui.show_help = false,
                _ => {}
            }
            return;
        }

        if self.ui.confirm_quit {
            match action {
                Action::ConfirmQuit => self.running = false,
                Action::CancelDialog => self.ui.confirm_quit = false,
                _ => {}
            }
            return;
        }

        match action {
            Action::QuitRequested => {
                if self.queue.total_files > self.queue.done_files {
                    self.ui.confirm_quit = true;
                } else {
                    self.running = false;
                }
            }
            Action::MoveUp => {
                self.browser.cursor = self.browser.cursor.saturating_sub(1);
            }
            Action::MoveDown => {
                if self.browser.cursor + 1 < self.browser.items.len() {
                    self.browser.cursor += 1;
                }
            }
            Action::FocusNext => {
                self.ui.focus = match self.ui.focus {
                    Focus::Browser => Focus::WorkPane,
                    Focus::WorkPane => Focus::Browser,
                };
            }
            Action::ToggleSelectCurrent => {
                if let Some(item) = self.browser.items.get(self.browser.cursor) {
                    if self.browser.selected.contains(&item.name) {
                        self.browser.selected.remove(&item.name);
                    } else {
                        self.browser.selected.insert(item.name.clone());
                    }
                }
            }
            Action::SelectAllVisible => {
                self.browser
                    .selected
                    .extend(self.browser.items.iter().map(|i| i.name.clone()));
            }
            Action::ClearSelection => {
                self.browser.selected.clear();
            }
            Action::NextTab => {
                self.ui.tab = self.ui.tab.next();
            }
            Action::PreviousTab => {
                self.ui.tab = self.ui.tab.previous();
            }
            Action::OpenPreview => {
                self.ui.tab = WorkTab::Preview;
            }
            Action::OpenLogsTab => {
                self.ui.tab = WorkTab::Logs;
            }
            Action::QueueDownloadSelected => {
                let selected_count = self.browser.selected.len() as u64;
                if selected_count > 0 {
                    self.queue.total_files = selected_count;
                    self.queue.total_bytes = selected_count * 1024 * 1024;
                    self.session.mode = "Download".to_string();
                    self.logs.push(format!(
                        "INFO queued {} selected item(s) for download",
                        selected_count
                    ));
                }
            }
            Action::QueueDownloadFolder => {
                self.queue.total_files = self.browser.items.len() as u64;
                self.queue.total_bytes = self.queue.total_files * 1024 * 1024;
                self.session.mode = "Download".to_string();
                self.logs.push(format!(
                    "INFO queued current folder with {} item(s)",
                    self.queue.total_files
                ));
            }
            Action::RunScript => {
                self.session.mode = "Script".to_string();
                self.script.last_result = "last run: success".to_string();
                self.logs
                    .push("INFO post-process script completed".to_string());
            }
            Action::Refresh => {
                self.logs.push("INFO refreshed S3 listing".to_string());
            }
            Action::OpenFilter => {
                self.logs
                    .push("INFO filter input not yet implemented".to_string());
            }
            Action::ToggleHelp => {
                self.ui.show_help = true;
            }
            Action::Tick => {
                if self.queue.done_files < self.queue.total_files {
                    self.queue.done_files += 1;
                    self.queue.done_bytes =
                        (self.queue.done_files * 1024 * 1024).min(self.queue.total_bytes);
                    self.queue.speed_mbps = 12.5;
                    if self.queue.done_files == self.queue.total_files {
                        self.session.mode = "Browse".to_string();
                        self.logs.push("INFO download queue completed".to_string());
                    }
                }
            }
            Action::MoveLeft
            | Action::MoveRight
            | Action::Enter
            | Action::GoParent
            | Action::ConfirmQuit
            | Action::CancelDialog => {}
        }
    }

    pub fn selected_count(&self) -> usize {
        self.browser.selected.len()
    }
}
