# S3 Grabber TUI UI Specification (V1)

## 1. Goal
Define the V1 terminal UI for Linux and Windows CLI environments.

The app must let users:
- Browse AWS S3 folders and objects.
- Select objects or full folders for download.
- Track download progress.
- Run a custom post-process script.
- Inspect object metadata before download.

## 2. Screen Layout
The screen is divided into three major sections.

### 2.1 Top Status Bar (3 rows)
Shows current session and navigation context:
- `Profile` (empty profile displayed as `default-chain`)
- `Region`
- `Bucket`
- `Path` (current prefix)
- `Target` (effective target only: `aws-profile:<name>` / `endpoint:<url>` / `default-chain`)
- `Mode` (`Browse | Download | Script | Error`)

### 2.2 Main Area
Split into two panes:
- Left pane (`35%`): `S3 Browser`
- Right pane (`65%`): `Work Pane`

#### Left Pane: S3 Browser
Purpose: navigation and selection only.

Columns:
- `Type` (`DIR` or `OBJ`)
- `Name`
- `Size` (for objects)
- `Modified`

Features:
- Cursor movement.
- Enter/open folder behavior.
- Multi-selection.
- Filter/search.

#### Right Pane: Work Pane Tabs
Tabs:
- `Selection`
- `Details`
- `Queue`
- `Logs`

Tab responsibilities:
- `Selection`: selected objects/folders, total file count/bytes, target download path.
- `Details`: metadata-only view for current browser item (object key, size, modified time, local download path).
- `Queue`: pending/running/completed/failed download jobs with progress.
- `Logs`: timestamped operational logs, warnings, errors, script output summaries.

### 2.3 Bottom Bar (3 rows)
Three fixed lines:
- Line 1: key hints (`h help`, `c connection`, navigation, selection, download actions).
- Line 2: transfer progress summary (`files`, `bytes`, `speed`, `ETA`).
- Line 3: profile/session/script status including effective `Target` (script name, last result).

## 3. Keymap
Cross-platform safe keys only.

### 3.1 Navigation
- `Up/Down`: move cursor.
- `Left/Right`: collapse/expand prefix level or move between contexts.
- `Enter`: open directory or inspect object.
- `Backspace`: go to parent prefix.
- `f`: switch focus between left and right panes.

### 3.2 Selection
- `Space`: add/remove current item (toggle).
- `a`: select all visible items.
- `x`: clear all selected items.

### 3.3 Tabs and Views
- `Tab`: next tab in right pane.
- `Shift+Tab`: previous tab in right pane.

### 3.4 Operations
- `d`: queue download for current selection.
- `D`: queue download for full current folder prefix.
- `s`: run post-process script (configured mode).
- `S`: open Script Picker modal.
- `r`: refresh list.
- `/`: open filter/search input.
- `c`: open Connection Settings modal.

### 3.5 Help and Exit
- `h`: open/close Help panel.
- `?`: optional alias for help.
- `Esc`: close help/dialog overlays.
- `q`: quit (with confirmation if queue has active jobs).
- `y`: confirm quit when quit confirmation dialog is open.

## 4. Connection Settings Modal
Runtime-editable S3 connection context popup.

Behavior:
- Triggered by `c`.
- Centered modal popup over existing UI.
- Prefills current active values.
- `Tab`/`Shift+Tab` move between fields (`Up`/`Down` also supported).
- Typing edits current field; `Backspace` deletes one character.
- `Enter` applies and reconnects context.
- `Esc` closes modal without applying.

Fields:
- `Profile (optional)` (empty = default AWS credential chain / EC2 role path)
- `Region`
- `Bucket` (required)
- `Prefix`
- `Endpoint URL (optional)` (empty = standard AWS S3 endpoint)

Validation:
- Bucket is required before apply.
- Inline error is shown in modal if validation fails.
- If both `profile` and `endpoint-url` are set, a warning is shown and `profile` takes precedence.
- If `endpoint-url` is selected as effective target and is invalid/unreachable, apply fails with no fallback.

Apply result:
- On success:
 - Session context updates (`profile`, `region`, `bucket`, `path`, `endpoint_url`).
 - Browser list is fetched from S3 (`ListObjectsV2`) immediately.
 - Selection and in-memory queue counters are reset.
 - Connection change is logged in `Logs` tab.
- On failure:
 - Existing active session context remains unchanged.
 - A warning is shown in the left pane (`S3 Browser`).
 - Detailed multiline diagnostics are written to `Logs` tab.

## 5. Help Panel Spec
A beginner-friendly key reference popup.

Behavior:
- Triggered by `h` (or `?`).
- Centered modal popup over existing UI.
- Non-destructive: opening/closing does not modify selection or jobs.
- While open, normal action keys are ignored except close keys (`h`, `?`, `Esc`).

Content groups:
- Navigation
- Selection
- Tabs/Views
- Download/Script
- System

Footer requirement:
- Always show `h help` hint in bottom key-hint line.

## 6. Script Picker Modal
Runtime script selection popup using a local scripts directory.

Behavior:
- Triggered by `S`.
- Centered modal popup over existing UI.
- Lists scripts found under `./scripts` (default directory).
- `Up`/`Down` move cursor in script list.
- `m` toggles script mode (`per-file` / `post-batch`).
- `r` rescans script directory.
- `Enter` selects highlighted script.
- `Esc` closes modal without changing current selection.

## 7. State Model
Core UI/application state components:
- `SessionState`: profile, region, bucket, current prefix, endpoint_url, connectivity.
- `BrowserState`: item list, cursor index, selected set, warning message.
- `SelectionState`: selected object keys and selected folder prefixes.
- `DetailsState`: current item metadata projection for right-pane details tab.
- `QueueState`: jobs and aggregate progress.
- `ScriptState`: script directory, available scripts, selected script, execution mode, last exit code and stderr summary.
- `UiState`: active pane focus, active tab, modal flags (`help`, `confirm_quit`, `connection_settings`, `script_picker`), notifications.
- `ConnectionDraft`: editable connection fields (`profile`, `region`, `bucket`, `prefix`, `endpoint_url`) and validation error.
- `ConfigState`: defaults for download dir, concurrency, retries.

## 8. Functional Rules
- Left pane is dedicated to browsing/selecting S3 items.
- Right pane displays details/workflow state only.
- Long operations must not block keyboard navigation.
- Details tab is metadata-only and does not fetch/render object body content.
- Confirm before quitting with running jobs.
- Connection settings are changeable at runtime via `c` modal.
- Script selection is changeable at runtime via `S` modal.
- Target precedence: `profile` > `endpoint-url` > default chain.
- If `profile` and `endpoint-url` are both set, endpoint is ignored (with warning).
- If effective `endpoint-url` connection fails, app must not fall back to default chain.
- `r` refresh performs a real S3 listing and reports failures in left-pane warning + logs.

## 9. MVP Constraints
- Download retries with backoff (default: `3` retries).
- Keyboard-only operation must be complete.
- Color should assist readability but not encode critical meaning alone.
- Minimum supported terminal size: `100x30`.

## 10. Acceptance Criteria (UI V1)
- User can browse prefixes/objects and navigate fully by keyboard.
- User can select multiple objects and full folder prefixes.
- User can queue downloads and observe per-job + aggregate progress.
- User can open Details (`p`) and inspect metadata for the currently highlighted item.
- User can open Help (`h`) and understand all major key actions.
- User can jump to logs with `l` and troubleshoot failed operations.
- User can open Connection Settings (`c`), edit profile/region/bucket/prefix, and apply changes.
- User can open Script Picker (`S`), select a script from `./scripts`, and toggle execution mode.
- User can optionally set `endpoint-url` for local S3 mock endpoints and apply changes.
- User can see effective target in top/bottom status (`profile` target hides ignored endpoint as active target).
- Invalid or unreachable endpoint-url shows warning in S3 Browser and does not auto-fallback.
