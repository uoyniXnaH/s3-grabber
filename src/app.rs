use std::collections::BTreeSet;

use crate::action::{Action, Focus, WorkTab};
use crate::services::s3::{
    list_objects_sync, resolve_target, validate_endpoint_url, S3ConnectParams, S3ListResult,
    S3Target, TargetWarning,
};

#[derive(Debug, Clone)]
pub struct SessionState {
    pub profile: String,
    pub region: String,
    pub bucket: String,
    pub path: String,
    pub endpoint_url: String,
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
    pub warning: Option<String>,
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
    EndpointUrl,
}

impl ConnectionField {
    fn next(self) -> Self {
        match self {
            Self::Profile => Self::Region,
            Self::Region => Self::Bucket,
            Self::Bucket => Self::Prefix,
            Self::Prefix => Self::EndpointUrl,
            Self::EndpointUrl => Self::Profile,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Profile => Self::EndpointUrl,
            Self::Region => Self::Profile,
            Self::Bucket => Self::Region,
            Self::Prefix => Self::Bucket,
            Self::EndpointUrl => Self::Prefix,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Profile => "Profile (optional)",
            Self::Region => "Region",
            Self::Bucket => "Bucket",
            Self::Prefix => "Prefix",
            Self::EndpointUrl => "Endpoint URL",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionDraft {
    pub profile: String,
    pub region: String,
    pub bucket: String,
    pub prefix: String,
    pub endpoint_url: String,
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
            endpoint_url: "".to_string(),
            mode: "Browse".to_string(),
        };

        Self {
            running: true,
            browser: BrowserState {
                items,
                cursor: 0,
                selected: BTreeSet::new(),
                warning: None,
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
                    endpoint_url: session.endpoint_url.clone(),
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
                self.reload_current_listing();
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

    pub fn display_effective_target(&self) -> String {
        let (target, _) = resolve_target(&self.session.profile, &self.session.endpoint_url);
        match target {
            S3Target::Profile { profile } => format!("aws-profile:{profile}"),
            S3Target::Endpoint { endpoint_url } => format!("endpoint:{endpoint_url}"),
            S3Target::DefaultChain => "default-chain".to_string(),
        }
    }

    pub fn connection_modal_warning(&self) -> Option<&'static str> {
        let (_, warning) = resolve_target(
            &self.ui.connection_draft.profile,
            &self.ui.connection_draft.endpoint_url,
        );
        match warning {
            Some(TargetWarning::ProfileOverridesEndpoint) => {
                Some("Profile is set, endpoint-url will be ignored.")
            }
            None => None,
        }
    }

    fn open_connection_settings(&mut self) {
        self.ui.connection_draft.profile = self.session.profile.clone();
        self.ui.connection_draft.region = self.session.region.clone();
        self.ui.connection_draft.bucket = self.session.bucket.clone();
        self.ui.connection_draft.prefix = self.session.path.clone();
        self.ui.connection_draft.endpoint_url = self.session.endpoint_url.clone();
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
            ConnectionField::EndpointUrl => &mut self.ui.connection_draft.endpoint_url,
        }
    }

    fn apply_connection_settings(&mut self) {
        if self.ui.connection_draft.bucket.trim().is_empty() {
            self.ui.connection_draft.error = Some("Bucket is required".to_string());
            return;
        }

        let draft_profile = self.ui.connection_draft.profile.trim().to_string();
        let draft_region = self.ui.connection_draft.region.trim().to_string();
        let draft_bucket = self.ui.connection_draft.bucket.trim().to_string();
        let draft_prefix = normalize_prefix(self.ui.connection_draft.prefix.trim());
        let draft_endpoint = self.ui.connection_draft.endpoint_url.trim().to_string();
        let (target, warning) = resolve_target(&draft_profile, &draft_endpoint);

        if let S3Target::Endpoint { endpoint_url } = &target {
            if let Err(err) = validate_endpoint_url(endpoint_url) {
                let message = format!("Invalid endpoint-url: {err}");
                self.ui.connection_draft.error = Some(message.clone());
                self.browser.warning = Some(message.clone());
                self.logs.push(format!(
                    "WARN connection rejected due to endpoint-url: {err}"
                ));
                return;
            }
        }

        let params = S3ConnectParams {
            profile: draft_profile.clone(),
            region: draft_region.clone(),
            bucket: draft_bucket.clone(),
            prefix: draft_prefix.clone(),
            endpoint_url: draft_endpoint.clone(),
            max_keys: 200,
        };

        let listing = match list_objects_sync(&params) {
            Ok(listing) => listing,
            Err(err) => {
                let message = "Connection failed. See Logs tab for details.".to_string();
                self.ui.connection_draft.error = Some(message.clone());
                self.browser.warning = Some(message);
                self.push_warn_multiline("connection failed", &err);
                return;
            }
        };

        self.session.profile = draft_profile;
        self.session.region = draft_region;
        self.session.bucket = draft_bucket;
        self.session.path = draft_prefix;
        self.session.endpoint_url = draft_endpoint;
        self.session.mode = "Browse".to_string();
        self.browser.warning = None;

        self.browser.items = self.build_browser_items(&listing);
        self.browser.selected.clear();
        self.browser.cursor = 0;
        self.queue.done_files = 0;
        self.queue.total_files = 0;
        self.queue.done_bytes = 0;
        self.queue.total_bytes = 0;

        let target_msg = match target {
            S3Target::Profile { profile } => format!("aws-profile:{profile}"),
            S3Target::Endpoint { endpoint_url } => format!("endpoint:{endpoint_url}"),
            S3Target::DefaultChain => "default-chain".to_string(),
        };
        self.logs.push(format!(
            "INFO connection updated target={} region={} bucket={} prefix={}",
            target_msg, self.session.region, self.session.bucket, self.session.path
        ));

        if matches!(warning, Some(TargetWarning::ProfileOverridesEndpoint)) {
            self.logs
                .push("WARN endpoint-url ignored because profile takes precedence".to_string());
        }

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

    fn reload_current_listing(&mut self) {
        let params = S3ConnectParams {
            profile: self.session.profile.clone(),
            region: self.session.region.clone(),
            bucket: self.session.bucket.clone(),
            prefix: self.session.path.clone(),
            endpoint_url: self.session.endpoint_url.clone(),
            max_keys: 200,
        };

        match list_objects_sync(&params) {
            Ok(listing) => {
                self.browser.items = self.build_browser_items(&listing);
                self.browser.cursor = 0;
                self.browser.warning = None;
                self.logs.push(format!(
                    "INFO refreshed S3 listing target={}",
                    self.display_effective_target()
                ));
            }
            Err(err) => {
                self.browser.warning =
                    Some("S3 refresh failed. See Logs tab for details.".to_string());
                self.push_warn_multiline("S3 refresh failed", &err);
            }
        }
    }

    fn build_browser_items(&self, listing: &S3ListResult) -> Vec<BrowserItem> {
        let base_prefix = api_prefix_from_path(&self.session.path);
        let mut items = Vec::new();

        for prefix in &listing.prefixes {
            items.push(BrowserItem {
                is_dir: true,
                name: display_key(prefix, base_prefix.as_deref()),
                size: None,
                modified: "-".to_string(),
            });
        }

        for object in &listing.objects {
            items.push(BrowserItem {
                is_dir: false,
                name: display_key(&object.key, base_prefix.as_deref()),
                size: Some(object.size),
                modified: object.modified.clone(),
            });
        }

        items
    }

    fn push_warn_multiline(&mut self, headline: &str, detail: &str) {
        self.logs.push(format!("WARN {headline}"));
        for line in detail.lines() {
            self.logs.push(format!("WARN {line}"));
        }
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

fn api_prefix_from_path(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        None
    } else if trimmed.ends_with('/') {
        Some(trimmed.to_string())
    } else {
        Some(format!("{trimmed}/"))
    }
}

fn display_key(key: &str, base_prefix: Option<&str>) -> String {
    match base_prefix {
        Some(base) => key
            .strip_prefix(base)
            .map_or_else(|| key.to_string(), ToString::to_string),
        None => key.to_string(),
    }
}
