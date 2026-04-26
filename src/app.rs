use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::action::{Action, Focus, WorkTab};
use crate::services::config::AppConfig;
use crate::services::s3::{
    download_object_to_path_sync, list_all_objects_sync, list_objects_sync, resolve_target,
    validate_endpoint_url, S3ConnectParams, S3ListResult, S3ObjectSummary, S3Target, TargetWarning,
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
    pub kind: BrowserItemKind,
    pub is_dir: bool,
    pub key: String,
    pub name: String,
    pub size: Option<u64>,
    pub modified: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BrowserItemKind {
    Parent,
    Dir,
    Obj,
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
    pub summary: String,
    pub started_at: Option<Instant>,
    pub jobs: Vec<QueueJob>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QueueJobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct QueueJob {
    pub key: String,
    pub local_path: PathBuf,
    pub size: u64,
    pub status: QueueJobStatus,
    pub attempts: u8,
    pub error: Option<String>,
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
    pub max_retries: u8,
    pub download_concurrency: usize,
    pub download_root: PathBuf,
}

impl App {
    pub fn new() -> Self {
        let items = vec![
            BrowserItem {
                kind: BrowserItemKind::Dir,
                is_dir: true,
                key: "logs/".to_string(),
                name: "logs/".to_string(),
                size: None,
                modified: "-".to_string(),
            },
            BrowserItem {
                kind: BrowserItemKind::Dir,
                is_dir: true,
                key: "reports/".to_string(),
                name: "reports/".to_string(),
                size: None,
                modified: "-".to_string(),
            },
            BrowserItem {
                kind: BrowserItemKind::Obj,
                is_dir: false,
                key: "README.txt".to_string(),
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
                summary: "No queue prepared. Use d (selected) or D (folder).".to_string(),
                started_at: None,
                jobs: Vec::new(),
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
            max_retries: AppConfig::default().max_retries,
            download_concurrency: AppConfig::default().concurrency.max(1),
            download_root: PathBuf::from(AppConfig::default().download_dir),
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
                self.go_parent_prefix();
            }
            Action::FocusNext | Action::InputChar('f') => {
                self.ui.focus = match self.ui.focus {
                    Focus::Browser => Focus::WorkPane,
                    Focus::WorkPane => Focus::Browser,
                };
            }
            Action::ToggleSelectCurrent => {
                if let Some(item) = self.browser.items.get(self.browser.cursor) {
                    if item_is_selectable(item) {
                        if self.browser.selected.contains(&item.key) {
                            self.browser.selected.remove(&item.key);
                        } else {
                            self.browser.selected.insert(item.key.clone());
                        }
                    }
                }
            }
            Action::InputChar('a') => {
                self.browser.selected.extend(
                    self.browser
                        .items
                        .iter()
                        .filter(|item| item_is_selectable(item))
                        .map(|item| item.key.clone()),
                );
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
                self.process_queue_tick();
            }
            Action::MoveLeft | Action::MoveRight | Action::CancelDialog | Action::InputChar(_) => {}
            Action::Enter => self.handle_enter(),
        }
    }

    pub fn selected_count(&self) -> usize {
        self.browser.selected.len()
    }

    pub fn selected_prefix_count(&self) -> usize {
        self.browser
            .selected
            .iter()
            .filter(|key| key.ends_with('/'))
            .count()
    }

    pub fn selected_object_count(&self) -> usize {
        self.browser
            .selected
            .iter()
            .filter(|key| !key.ends_with('/'))
            .count()
    }

    pub fn queue_status_counts(&self) -> (usize, usize, usize, usize) {
        let pending = self
            .queue
            .jobs
            .iter()
            .filter(|job| job.status == QueueJobStatus::Pending)
            .count();
        let running = self
            .queue
            .jobs
            .iter()
            .filter(|job| job.status == QueueJobStatus::Running)
            .count();
        let done = self
            .queue
            .jobs
            .iter()
            .filter(|job| job.status == QueueJobStatus::Done)
            .count();
        let failed = self
            .queue
            .jobs
            .iter()
            .filter(|job| job.status == QueueJobStatus::Failed)
            .count();

        (pending, running, done, failed)
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
        let connection_target_changed =
            self.session.profile != draft_profile || self.session.endpoint_url != draft_endpoint;
        let effective_prefix = if connection_target_changed {
            "/".to_string()
        } else {
            draft_prefix
        };

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
            prefix: effective_prefix.clone(),
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
        self.session.path = effective_prefix;
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
        self.queue.speed_mbps = 0.0;
        self.queue.eta = "--:--".to_string();
        self.queue.started_at = None;
        self.queue.summary = "No queue prepared. Use d (selected) or D (folder).".to_string();

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
        if connection_target_changed {
            self.logs
                .push("INFO profile/endpoint changed; browser reset to root prefix".to_string());
        }

        self.ui.connection_draft.error = None;
        self.ui.show_connection_settings = false;
    }

    fn queue_download_selected(&mut self) {
        if self.browser.selected.is_empty() {
            self.browser.warning =
                Some("No selection to queue. Select objects or folders first.".to_string());
            self.logs
                .push("WARN queue requested with empty selection".to_string());
            return;
        }

        match self.resolve_selected_objects() {
            Ok(resolution) => {
                self.prepare_download_queue(resolution.objects);
                self.queue.summary = format!(
                    "Plan: selected {} objects + {} prefixes -> {} effective prefixes + {} direct objects (skipped {} overlap prefixes, {} covered objects) => {} queued",
                    resolution.stats.selected_object_count,
                    resolution.stats.selected_prefix_count,
                    resolution.stats.effective_prefix_count,
                    resolution.stats.effective_direct_object_count,
                    resolution.stats.overlapping_prefixes_skipped,
                    resolution.stats.covered_objects_skipped,
                    self.queue.total_files
                );
                self.logs.push(format!(
                    "INFO queue summary selected(objects={}, prefixes={}) effective(prefixes={}, direct_objects={}) skipped(overlap_prefixes={}, covered_objects={}) final_objects={}",
                    resolution.stats.selected_object_count,
                    resolution.stats.selected_prefix_count,
                    resolution.stats.effective_prefix_count,
                    resolution.stats.effective_direct_object_count,
                    resolution.stats.overlapping_prefixes_skipped,
                    resolution.stats.covered_objects_skipped,
                    self.queue.total_files
                ));
            }
            Err(err) => {
                self.browser.warning =
                    Some("Failed to resolve selected objects. See Logs tab.".to_string());
                self.push_warn_multiline("resolve selected objects failed", &err);
            }
        }
    }

    fn process_queue_tick(&mut self) {
        if self.queue.jobs.is_empty() {
            return;
        }

        if self.queue.started_at.is_none() {
            self.queue.started_at = Some(Instant::now());
        }

        let running = self
            .queue
            .jobs
            .iter()
            .filter(|job| job.status == QueueJobStatus::Running)
            .count();
        let slots = self.download_concurrency.saturating_sub(running);
        if slots == 0 {
            self.update_transfer_metrics();
            return;
        }

        let pending_indices = self
            .queue
            .jobs
            .iter()
            .enumerate()
            .filter(|(_, job)| job.status == QueueJobStatus::Pending)
            .map(|(index, _)| index)
            .take(slots)
            .collect::<Vec<_>>();

        if pending_indices.is_empty() {
            self.update_transfer_metrics();
            if self
                .queue
                .jobs
                .iter()
                .all(|job| matches!(job.status, QueueJobStatus::Done | QueueJobStatus::Failed))
                && self.session.mode == "Download"
            {
                self.session.mode = "Browse".to_string();
                self.logs.push("INFO download queue completed".to_string());
            }
            return;
        }

        let mut batch = Vec::with_capacity(pending_indices.len());
        for index in pending_indices {
            let key = self.queue.jobs[index].key.clone();
            let path = self.queue.jobs[index].local_path.clone();
            let attempt = self.queue.jobs[index].attempts.saturating_add(1);
            self.queue.jobs[index].attempts = attempt;
            self.queue.jobs[index].status = QueueJobStatus::Running;
            self.logs.push(format!(
                "INFO downloading key={} attempt={}/{}",
                key, attempt, self.max_retries
            ));
            batch.push((index, key, path));
        }

        let params = S3ConnectParams {
            profile: self.session.profile.clone(),
            region: self.session.region.clone(),
            bucket: self.session.bucket.clone(),
            prefix: self.session.path.clone(),
            endpoint_url: self.session.endpoint_url.clone(),
            max_keys: 200,
        };

        let mut results = Vec::with_capacity(batch.len());
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(batch.len());
            for (index, key, path) in batch {
                let params = params.clone();
                handles.push(scope.spawn(move || {
                    let result = std::panic::catch_unwind(|| {
                        download_object_to_path_sync(&params, &key, &path)
                    })
                    .unwrap_or_else(|_| Err("download worker panicked".to_string()));
                    (index, key, path, result)
                }));
            }

            for handle in handles {
                if let Ok(result) = handle.join() {
                    results.push(result);
                }
            }
        });

        for (index, key, path, result) in results {
            match result {
                Ok(download) => {
                    self.queue.jobs[index].status = QueueJobStatus::Done;
                    self.queue.jobs[index].error = None;
                    self.queue.done_files += 1;
                    self.queue.done_bytes += download.bytes_written;
                    self.logs.push(format!(
                        "INFO download complete key={} bytes={} path={}",
                        key,
                        download.bytes_written,
                        path.display()
                    ));
                }
                Err(err) => {
                    let attempts = self.queue.jobs[index].attempts;
                    if attempts < self.max_retries {
                        self.queue.jobs[index].status = QueueJobStatus::Pending;
                        self.queue.jobs[index].error = Some(err.clone());
                        self.logs.push(format!(
                            "WARN download failed key={} attempt={}/{} retrying",
                            key, attempts, self.max_retries
                        ));
                        self.push_warn_multiline("download attempt failed", &err);
                    } else {
                        self.queue.jobs[index].status = QueueJobStatus::Failed;
                        self.queue.jobs[index].error = Some(err.clone());
                        self.queue.done_files += 1;
                        self.logs.push(format!(
                            "WARN download failed permanently key={} attempts={}",
                            key, attempts
                        ));
                        self.push_warn_multiline("download failed permanently", &err);
                    }
                }
            }
        }

        self.update_transfer_metrics();
        if self
            .queue
            .jobs
            .iter()
            .all(|job| matches!(job.status, QueueJobStatus::Done | QueueJobStatus::Failed))
            && self.session.mode == "Download"
        {
            self.queue.eta = "00:00".to_string();
            self.session.mode = "Browse".to_string();
            self.logs.push("INFO download queue completed".to_string());
        }
    }

    fn queue_download_folder(&mut self) {
        let params = S3ConnectParams {
            profile: self.session.profile.clone(),
            region: self.session.region.clone(),
            bucket: self.session.bucket.clone(),
            prefix: self.session.path.clone(),
            endpoint_url: self.session.endpoint_url.clone(),
            max_keys: 1000,
        };
        match list_all_objects_sync(&params) {
            Ok(objects) => {
                self.prepare_download_queue(objects);
                self.queue.summary = format!(
                    "Plan: full folder prefix {} => {} queued object(s)",
                    self.session.path, self.queue.total_files
                );
                self.logs.push(format!(
                    "INFO queued folder prefix {} with {} object(s)",
                    self.session.path, self.queue.total_files
                ));
            }
            Err(err) => {
                self.browser.warning =
                    Some("Failed to queue folder download. See Logs tab.".to_string());
                self.push_warn_multiline("queue folder download failed", &err);
            }
        }
    }

    fn resolve_selected_objects(&self) -> Result<SelectionResolution, String> {
        let plan = SelectionPlan::from_selected(&self.browser.selected);
        let mut by_key = BTreeMap::<String, S3ObjectSummary>::new();

        for prefix in &plan.effective_prefixes {
            let params = S3ConnectParams {
                profile: self.session.profile.clone(),
                region: self.session.region.clone(),
                bucket: self.session.bucket.clone(),
                prefix: normalize_prefix(prefix),
                endpoint_url: self.session.endpoint_url.clone(),
                max_keys: 1000,
            };
            let expanded = list_all_objects_sync(&params)?;
            for obj in expanded {
                by_key.entry(obj.key.clone()).or_insert(obj);
            }
        }

        for key in &plan.effective_object_keys {
            let known_size = self
                .browser
                .items
                .iter()
                .find(|item| item.key == *key)
                .and_then(|item| item.size)
                .unwrap_or(0);

            by_key
                .entry(key.clone())
                .or_insert_with(|| S3ObjectSummary {
                    key: key.clone(),
                    size: known_size,
                    modified: "-".to_string(),
                });
        }

        Ok(SelectionResolution {
            objects: by_key.into_values().collect(),
            stats: SelectionResolutionStats {
                selected_prefix_count: plan.selected_prefix_count,
                selected_object_count: plan.selected_object_count,
                effective_prefix_count: plan.effective_prefixes.len(),
                effective_direct_object_count: plan.effective_object_keys.len(),
                overlapping_prefixes_skipped: plan.overlapping_prefixes_skipped,
                covered_objects_skipped: plan.covered_objects_skipped,
            },
        })
    }

    fn prepare_download_queue(&mut self, objects: Vec<S3ObjectSummary>) {
        let mut deduped = BTreeMap::<String, S3ObjectSummary>::new();
        for obj in objects {
            deduped
                .entry(obj.key.clone())
                .and_modify(|existing| {
                    if obj.size > existing.size {
                        existing.size = obj.size;
                    }
                    if existing.modified == "-" && obj.modified != "-" {
                        existing.modified = obj.modified.clone();
                    }
                })
                .or_insert(obj);
        }

        self.queue.jobs = deduped
            .into_values()
            .into_iter()
            .map(|obj| QueueJob {
                local_path: local_destination_for_key(&self.download_root, &obj.key),
                key: obj.key,
                size: obj.size,
                status: QueueJobStatus::Pending,
                attempts: 0,
                error: None,
            })
            .collect();

        self.queue.total_files = self.queue.jobs.len() as u64;
        self.queue.total_bytes = self.queue.jobs.iter().map(|job| job.size).sum();
        self.queue.done_files = 0;
        self.queue.done_bytes = 0;
        self.queue.speed_mbps = 0.0;
        self.queue.eta = if self.queue.total_files > 0 {
            "calculating".to_string()
        } else {
            "--:--".to_string()
        };
        self.queue.started_at = if self.queue.total_files > 0 {
            Some(Instant::now())
        } else {
            None
        };
        self.session.mode = if self.queue.total_files > 0 {
            "Download".to_string()
        } else {
            "Browse".to_string()
        };
    }

    fn update_transfer_metrics(&mut self) {
        let Some(started_at) = self.queue.started_at else {
            self.queue.speed_mbps = 0.0;
            self.queue.eta = "--:--".to_string();
            return;
        };

        let elapsed = started_at.elapsed().as_secs_f64();
        if elapsed <= f64::EPSILON {
            self.queue.speed_mbps = 0.0;
            self.queue.eta = "calculating".to_string();
            return;
        }

        let bytes_per_second = self.queue.done_bytes as f64 / elapsed;
        self.queue.speed_mbps = bytes_per_second / (1024.0 * 1024.0);
        let remaining_bytes = self
            .queue
            .jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    QueueJobStatus::Pending | QueueJobStatus::Running
                )
            })
            .map(|job| job.size)
            .sum::<u64>();

        self.queue.eta = if remaining_bytes == 0 {
            "00:00".to_string()
        } else if bytes_per_second <= f64::EPSILON {
            "calculating".to_string()
        } else {
            format_eta((remaining_bytes as f64 / bytes_per_second).ceil() as u64)
        };
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
                self.browser.selected.clear();
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

        if self.session.path != "/" {
            items.push(BrowserItem {
                kind: BrowserItemKind::Parent,
                is_dir: true,
                key: parent_prefix(&self.session.path),
                name: "[..]".to_string(),
                size: None,
                modified: "-".to_string(),
            });
        }

        for prefix in &listing.prefixes {
            items.push(BrowserItem {
                kind: BrowserItemKind::Dir,
                is_dir: true,
                key: prefix.clone(),
                name: display_key(prefix, base_prefix.as_deref()),
                size: None,
                modified: "-".to_string(),
            });
        }

        for object in &listing.objects {
            items.push(BrowserItem {
                kind: BrowserItemKind::Obj,
                is_dir: false,
                key: object.key.clone(),
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

    fn handle_enter(&mut self) {
        let Some(item) = self.browser.items.get(self.browser.cursor) else {
            return;
        };
        match item.kind {
            BrowserItemKind::Parent => self.navigate_to_prefix(item.key.clone(), "parent"),
            BrowserItemKind::Dir => {
                self.navigate_to_prefix(normalize_prefix(&item.key), "open-dir")
            }
            BrowserItemKind::Obj => {
                self.ui.tab = WorkTab::Preview;
                self.logs
                    .push(format!("INFO open object preview key={}", item.key));
            }
        }
    }

    fn go_parent_prefix(&mut self) {
        if self.session.path == "/" {
            self.logs.push("INFO already at root prefix".to_string());
            return;
        }
        self.navigate_to_prefix(parent_prefix(&self.session.path), "parent");
    }

    fn navigate_to_prefix(&mut self, prefix: String, reason: &str) {
        let params = S3ConnectParams {
            profile: self.session.profile.clone(),
            region: self.session.region.clone(),
            bucket: self.session.bucket.clone(),
            prefix: prefix.clone(),
            endpoint_url: self.session.endpoint_url.clone(),
            max_keys: 200,
        };

        match list_objects_sync(&params) {
            Ok(listing) => {
                self.session.path = normalize_prefix(&prefix);
                self.browser.items = self.build_browser_items(&listing);
                self.browser.cursor = 0;
                self.browser.warning = None;
                self.browser.selected.clear();
                self.logs.push(format!(
                    "INFO navigated reason={} prefix={} target={}",
                    reason,
                    self.session.path,
                    self.display_effective_target()
                ));
            }
            Err(err) => {
                self.browser.warning =
                    Some("S3 navigation failed. See Logs tab for details.".to_string());
                self.push_warn_multiline("S3 navigation failed", &err);
            }
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

fn parent_prefix(path: &str) -> String {
    let normalized = normalize_prefix(path);
    if normalized == "/" {
        return normalized;
    }
    let trimmed = normalized.trim_end_matches('/');
    let parent = trimmed.rsplit_once('/').map(|(head, _)| head).unwrap_or("");
    if parent.is_empty() {
        "/".to_string()
    } else {
        format!("{parent}/")
    }
}

fn item_is_selectable(item: &BrowserItem) -> bool {
    !matches!(item.kind, BrowserItemKind::Parent)
}

fn local_destination_for_key(root: &Path, key: &str) -> PathBuf {
    let sanitized = key.replace('/', std::path::MAIN_SEPARATOR.to_string().as_str());
    root.join(sanitized)
}

fn format_eta(total_seconds: u64) -> String {
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[derive(Debug)]
struct SelectionResolution {
    objects: Vec<S3ObjectSummary>,
    stats: SelectionResolutionStats,
}

#[derive(Debug)]
struct SelectionResolutionStats {
    selected_prefix_count: usize,
    selected_object_count: usize,
    effective_prefix_count: usize,
    effective_direct_object_count: usize,
    overlapping_prefixes_skipped: usize,
    covered_objects_skipped: usize,
}

#[derive(Debug)]
struct SelectionPlan {
    selected_prefix_count: usize,
    selected_object_count: usize,
    effective_prefixes: Vec<String>,
    effective_object_keys: Vec<String>,
    overlapping_prefixes_skipped: usize,
    covered_objects_skipped: usize,
}

impl SelectionPlan {
    fn from_selected(selected: &BTreeSet<String>) -> Self {
        let mut prefixes = selected
            .iter()
            .filter(|key| key.ends_with('/'))
            .cloned()
            .collect::<Vec<_>>();
        prefixes.sort();
        let selected_prefix_count = prefixes.len();

        let mut effective_prefixes = Vec::<String>::new();
        let mut overlapping_prefixes_skipped = 0usize;
        for prefix in prefixes {
            if effective_prefixes
                .iter()
                .any(|existing| prefix.starts_with(existing))
            {
                overlapping_prefixes_skipped += 1;
            } else {
                effective_prefixes.push(prefix);
            }
        }

        let mut object_keys = selected
            .iter()
            .filter(|key| !key.ends_with('/'))
            .cloned()
            .collect::<Vec<_>>();
        object_keys.sort();
        let selected_object_count = object_keys.len();

        let mut effective_object_keys = Vec::new();
        let mut covered_objects_skipped = 0usize;
        for key in object_keys {
            if effective_prefixes
                .iter()
                .any(|prefix| key.starts_with(prefix))
            {
                covered_objects_skipped += 1;
            } else {
                effective_object_keys.push(key);
            }
        }

        Self {
            selected_prefix_count,
            selected_object_count,
            effective_prefixes,
            effective_object_keys,
            overlapping_prefixes_skipped,
            covered_objects_skipped,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_plan_skips_overlapping_prefixes_and_covered_objects() {
        let selected = BTreeSet::from([
            "logs/".to_string(),
            "logs/archive/".to_string(),
            "logs/2026-04-01.log".to_string(),
            "reports/".to_string(),
            "reports/summary.csv".to_string(),
            "README.txt".to_string(),
        ]);

        let plan = SelectionPlan::from_selected(&selected);
        assert_eq!(plan.selected_prefix_count, 3);
        assert_eq!(plan.selected_object_count, 3);
        assert_eq!(plan.effective_prefixes, vec!["logs/", "reports/"]);
        assert_eq!(plan.effective_object_keys, vec!["README.txt"]);
        assert_eq!(plan.overlapping_prefixes_skipped, 1);
        assert_eq!(plan.covered_objects_skipped, 2);
    }

    #[test]
    fn resolve_selected_objects_uses_direct_selection_without_prefix_queries() {
        let mut app = App::new();
        app.browser.items = vec![
            BrowserItem {
                kind: BrowserItemKind::Obj,
                is_dir: false,
                key: "z.txt".to_string(),
                name: "z.txt".to_string(),
                size: Some(99),
                modified: "-".to_string(),
            },
            BrowserItem {
                kind: BrowserItemKind::Obj,
                is_dir: false,
                key: "a.txt".to_string(),
                name: "a.txt".to_string(),
                size: Some(11),
                modified: "-".to_string(),
            },
        ];
        app.browser.selected = BTreeSet::from(["z.txt".to_string(), "a.txt".to_string()]);

        let resolution = app.resolve_selected_objects().expect("resolution succeeds");

        let keys = resolution
            .objects
            .iter()
            .map(|obj| obj.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(keys, vec!["a.txt", "z.txt"]);
        assert_eq!(resolution.stats.selected_prefix_count, 0);
        assert_eq!(resolution.stats.selected_object_count, 2);
        assert_eq!(resolution.stats.effective_prefix_count, 0);
        assert_eq!(resolution.stats.effective_direct_object_count, 2);
    }

    #[test]
    fn prepare_download_queue_dedups_by_key_and_uses_stable_sort() {
        let mut app = App::new();
        app.prepare_download_queue(vec![
            S3ObjectSummary {
                key: "b/file.txt".to_string(),
                size: 1,
                modified: "-".to_string(),
            },
            S3ObjectSummary {
                key: "a/file.txt".to_string(),
                size: 10,
                modified: "-".to_string(),
            },
            S3ObjectSummary {
                key: "a/file.txt".to_string(),
                size: 20,
                modified: "mtime".to_string(),
            },
        ]);

        let keys = app
            .queue
            .jobs
            .iter()
            .map(|job| job.key.as_str())
            .collect::<Vec<_>>();
        assert_eq!(keys, vec!["a/file.txt", "b/file.txt"]);
        assert_eq!(app.queue.total_files, 2);
        assert_eq!(app.queue.total_bytes, 21);
    }

    #[test]
    fn queue_status_counts_reports_each_bucket() {
        let mut app = App::new();
        app.queue.jobs = vec![
            QueueJob {
                key: "a".to_string(),
                local_path: PathBuf::from("a"),
                size: 1,
                status: QueueJobStatus::Pending,
                attempts: 0,
                error: None,
            },
            QueueJob {
                key: "b".to_string(),
                local_path: PathBuf::from("b"),
                size: 1,
                status: QueueJobStatus::Running,
                attempts: 1,
                error: None,
            },
            QueueJob {
                key: "c".to_string(),
                local_path: PathBuf::from("c"),
                size: 1,
                status: QueueJobStatus::Done,
                attempts: 1,
                error: None,
            },
            QueueJob {
                key: "d".to_string(),
                local_path: PathBuf::from("d"),
                size: 1,
                status: QueueJobStatus::Failed,
                attempts: 2,
                error: Some("err".to_string()),
            },
        ];

        assert_eq!(app.queue_status_counts(), (1, 1, 1, 1));
    }

    #[test]
    fn queue_selected_with_empty_selection_warns_and_does_not_queue() {
        let mut app = App::new();
        app.browser.selected.clear();

        app.update(Action::QueueDownloadSelected);

        assert_eq!(app.queue.total_files, 0);
        assert!(app
            .browser
            .warning
            .as_deref()
            .is_some_and(|w| w.contains("No selection to queue")));
    }

    #[test]
    fn prepare_download_queue_sets_started_state() {
        let mut app = App::new();
        app.prepare_download_queue(vec![S3ObjectSummary {
            key: "a/file.txt".to_string(),
            size: 10,
            modified: "-".to_string(),
        }]);

        assert!(app.queue.started_at.is_some());
        assert_eq!(app.queue.eta, "calculating");
        assert_eq!(app.session.mode, "Download");
    }

    #[test]
    fn format_eta_formats_mm_ss_and_hh_mm_ss() {
        assert_eq!(format_eta(5), "00:05");
        assert_eq!(format_eta(125), "02:05");
        assert_eq!(format_eta(3_726), "01:02:06");
    }
}
