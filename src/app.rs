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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConnectionField {
    Profile,
    Region,
    Bucket,
    Prefix,
}

impl ConnectionField {
    fn next(self) -> Self {
        match self {
            Self::Profile => Self::Region,
            Self::Region => Self::Bucket,
            Self::Bucket => Self::Prefix,
            Self::Prefix => Self::Profile,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Profile => Self::Prefix,
            Self::Region => Self::Profile,
            Self::Bucket => Self::Region,
            Self::Prefix => Self::Bucket,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Profile => "Profile (optional)",
            Self::Region => "Region",
            Self::Bucket => "Bucket",
            Self::Prefix => "Prefix",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionDraft {
    pub profile: String,
    pub region: String,
    pub bucket: String,
    pub prefix: String,
    pub active_field: ConnectionField,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub focus: Focus,
    pub tab: WorkTab,
    pub show_help: bool,
    pub confirm_quit: bool,
    pub show_connection_settings: bool,
    pub connection_draft: ConnectionDraft,
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

        let session = SessionState {
            profile: "".to_string(),
            region: "ap-northeast-1".to_string(),
            bucket: "my-bucket".to_string(),
            path: "/".to_string(),
            mode: "Browse".to_string(),
        };

        Self {
            running: true,
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
                show_connection_settings: false,
                connection_draft: ConnectionDraft {
                    profile: session.profile.clone(),
                    region: session.region.clone(),
                    bucket: session.bucket.clone(),
                    prefix: session.path.clone(),
                    active_field: ConnectionField::Profile,
                    error: None,
                },
            },
            logs: vec!["INFO app initialized".to_string()],
            session,
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
                Action::InputChar('y') | Action::InputChar('Y') => self.running = false,
                Action::CancelDialog => self.ui.confirm_quit = false,
                _ => {}
            }
            return;
        }

        if self.ui.show_connection_settings {
            self.update_connection_modal(action);
            return;
        }

        match action {
            Action::QuitRequested | Action::InputChar('q') => {
                if self.queue.total_files > self.queue.done_files {
                    self.ui.confirm_quit = true;
                } else {
                    self.running = false;
                }
            }
            Action::InputChar('h') | Action::InputChar('?') | Action::ToggleHelp => {
                self.ui.show_help = true;
            }
            Action::InputChar('c') | Action::InputChar('C') => {
                self.open_connection_settings();
            }
            Action::MoveUp => {
                self.browser.cursor = self.browser.cursor.saturating_sub(1);
            }
            Action::MoveDown => {
                if self.browser.cursor + 1 < self.browser.items.len() {
                    self.browser.cursor += 1;
                }
            }
            Action::BackspaceKey => {
                self.logs
                    .push("INFO parent prefix navigation not yet implemented".to_string());
            }
            Action::FocusNext | Action::InputChar('f') => {
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
            Action::InputChar('a') => {
                self.browser
                    .selected
                    .extend(self.browser.items.iter().map(|i| i.name.clone()));
            }
            Action::InputChar('x') => {
                self.browser.selected.clear();
            }
            Action::NextTab => {
                self.ui.tab = self.ui.tab.next();
            }
            Action::PreviousTab => {
                self.ui.tab = self.ui.tab.previous();
            }
            Action::OpenPreview | Action::InputChar('p') => {
                self.ui.tab = WorkTab::Preview;
            }
            Action::OpenLogsTab | Action::InputChar('l') => {
                self.ui.tab = WorkTab::Logs;
            }
            Action::QueueDownloadSelected | Action::InputChar('d') => {
                self.queue_download_selected();
            }
            Action::QueueDownloadFolder | Action::InputChar('D') => {
                self.queue_download_folder();
            }
            Action::RunScript | Action::InputChar('s') => {
                self.session.mode = "Script".to_string();
                self.script.last_result = "last run: success".to_string();
                self.logs
                    .push("INFO post-process script completed".to_string());
            }
            Action::Refresh | Action::InputChar('r') => {
                self.logs.push("INFO refreshed S3 listing".to_string());
            }
            Action::OpenFilter | Action::InputChar('/') => {
                self.logs
                    .push("INFO filter input not yet implemented".to_string());
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
            | Action::CancelDialog
            | Action::InputChar(_) => {}
        }
    }

    pub fn selected_count(&self) -> usize {
        self.browser.selected.len()
    }

    pub fn display_profile(&self) -> &str {
        if self.session.profile.trim().is_empty() {
            "default-chain"
        } else {
            self.session.profile.as_str()
        }
    }

    fn open_connection_settings(&mut self) {
        self.ui.connection_draft.profile = self.session.profile.clone();
        self.ui.connection_draft.region = self.session.region.clone();
        self.ui.connection_draft.bucket = self.session.bucket.clone();
        self.ui.connection_draft.prefix = self.session.path.clone();
        self.ui.connection_draft.active_field = ConnectionField::Profile;
        self.ui.connection_draft.error = None;
        self.ui.show_connection_settings = true;
    }

    fn update_connection_modal(&mut self, action: Action) {
        match action {
            Action::CancelDialog => {
                self.ui.show_connection_settings = false;
            }
            Action::NextTab | Action::MoveDown => {
                self.ui.connection_draft.active_field =
                    self.ui.connection_draft.active_field.next();
            }
            Action::PreviousTab | Action::MoveUp => {
                self.ui.connection_draft.active_field =
                    self.ui.connection_draft.active_field.previous();
            }
            Action::Enter => {
                self.apply_connection_settings();
            }
            Action::BackspaceKey => {
                let target = self.active_connection_field_mut();
                target.pop();
            }
            Action::InputChar(ch) => {
                if !ch.is_control() {
                    let target = self.active_connection_field_mut();
                    target.push(ch);
                    self.ui.connection_draft.error = None;
                }
            }
            _ => {}
        }
    }

    fn active_connection_field_mut(&mut self) -> &mut String {
        match self.ui.connection_draft.active_field {
            ConnectionField::Profile => &mut self.ui.connection_draft.profile,
            ConnectionField::Region => &mut self.ui.connection_draft.region,
            ConnectionField::Bucket => &mut self.ui.connection_draft.bucket,
            ConnectionField::Prefix => &mut self.ui.connection_draft.prefix,
        }
    }

    fn apply_connection_settings(&mut self) {
        if self.ui.connection_draft.bucket.trim().is_empty() {
            self.ui.connection_draft.error = Some("Bucket is required".to_string());
            return;
        }

        self.session.profile = self.ui.connection_draft.profile.trim().to_string();
        self.session.region = self.ui.connection_draft.region.trim().to_string();
        self.session.bucket = self.ui.connection_draft.bucket.trim().to_string();
        self.session.path = normalize_prefix(self.ui.connection_draft.prefix.trim());
        self.session.mode = "Browse".to_string();

        self.browser.selected.clear();
        self.browser.cursor = 0;
        self.queue.done_files = 0;
        self.queue.total_files = 0;
        self.queue.done_bytes = 0;
        self.queue.total_bytes = 0;

        self.logs.push(format!(
            "INFO connection updated profile={} region={} bucket={} prefix={}",
            if self.session.profile.is_empty() {
                "default-chain"
            } else {
                self.session.profile.as_str()
            },
            self.session.region,
            self.session.bucket,
            self.session.path
        ));

        self.ui.connection_draft.error = None;
        self.ui.show_connection_settings = false;
    }

    fn queue_download_selected(&mut self) {
        let selected_count = self.browser.selected.len() as u64;
        if selected_count > 0 {
            self.queue.total_files = selected_count;
            self.queue.total_bytes = selected_count * 1024 * 1024;
            self.queue.done_files = 0;
            self.queue.done_bytes = 0;
            self.session.mode = "Download".to_string();
            self.logs.push(format!(
                "INFO queued {} selected item(s) for download",
                selected_count
            ));
        }
    }

    fn queue_download_folder(&mut self) {
        self.queue.total_files = self.browser.items.len() as u64;
        self.queue.total_bytes = self.queue.total_files * 1024 * 1024;
        self.queue.done_files = 0;
        self.queue.done_bytes = 0;
        self.session.mode = "Download".to_string();
        self.logs.push(format!(
            "INFO queued current folder with {} item(s)",
            self.queue.total_files
        ));
    }
}

fn normalize_prefix(prefix: &str) -> String {
    if prefix.is_empty() {
        return "/".to_string();
    }

    if prefix.starts_with('/') {
        prefix.to_string()
    } else {
        format!("/{prefix}")
    }
}
