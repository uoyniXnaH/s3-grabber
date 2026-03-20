# S3 Grabber TUI UI Specification (V1)

## 1. Goal
Define the V1 terminal UI for Linux and Windows CLI environments.

The app must let users:
- Browse AWS S3 folders and objects.
- Select objects or full folders for download.
- Track download progress.
- Run a custom post-process script.
- Preview text-based objects.

## 2. Screen Layout
The screen is divided into three major sections.

### 2.1 Top Status Bar (3 rows)
Shows current session and navigation context:
- `Profile`
- `Region`
- `Bucket`
- `Path` (current prefix)
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
- `Preview`
- `Queue`
- `Logs`

Tab responsibilities:
- `Selection`: selected objects/folders, total file count/bytes, target download path.
- `Preview`: read-only text preview and object metadata.
- `Queue`: pending/running/completed/failed download jobs with progress.
- `Logs`: timestamped operational logs, warnings, errors, script output summaries.

### 2.3 Bottom Bar (3 rows)
Three fixed lines:
- Line 1: key hints (`h help`, navigation, selection, download actions).
- Line 2: transfer progress summary (`files`, `bytes`, `speed`, `ETA`).
- Line 3: user/session/script status (script name, last result).

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
- `p`: open preview for current object.
- `l`: jump directly to `Logs` tab.

### 3.4 Operations
- `d`: queue download for current selection.
- `D`: queue download for full current folder prefix.
- `s`: run post-process script (configured mode).
- `r`: refresh list.
- `/`: open filter/search input.

### 3.5 Help and Exit
- `h`: open/close Help panel.
- `?`: optional alias for help.
- `Esc`: close help/dialog overlays.
- `q`: quit (with confirmation if queue has active jobs).

## 4. Help Panel Spec
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

## 5. State Model
Core UI/application state components:
- `SessionState`: profile, region, bucket, current prefix, connectivity.
- `BrowserState`: item list, cursor index, filter text, pagination token.
- `SelectionState`: selected object keys and selected folder prefixes.
- `PreviewState`: key, loading status, text buffer, truncation, preview errors.
- `QueueState`: jobs and aggregate progress.
- `ScriptState`: command config, execution mode, last exit code and stderr summary.
- `UiState`: active pane focus, active tab, open modal, notifications.
- `ConfigState`: defaults for download dir, concurrency, retries, preview limit.

## 6. Functional Rules
- Left pane is dedicated to browsing/selecting S3 items.
- Right pane displays details/workflow state only.
- Long operations must not block keyboard navigation.
- Preview supports text-like content only.
- Unsupported/binary content shows metadata-only preview message.
- Confirm before quitting with running jobs.

## 7. MVP Constraints
- Text preview size cap (default: `1 MiB`).
- Download retries with backoff (default: `3` retries).
- Keyboard-only operation must be complete.
- Color should assist readability but not encode critical meaning alone.
- Minimum supported terminal size: `100x30`.

## 8. Acceptance Criteria (UI V1)
- User can browse prefixes/objects and navigate fully by keyboard.
- User can select multiple objects and full folder prefixes.
- User can queue downloads and observe per-job + aggregate progress.
- User can preview supported text content or see clear unsupported message.
- User can open Help (`h`) and understand all major key actions.
- User can jump to logs with `l` and troubleshoot failed operations.

## 9. Next Implementation Step
Scaffold ratatui app structure aligned to this spec:
- `main.rs`, `app.rs`, `action.rs`, `event.rs`, `ui.rs`
- `components/` (`browser`, `selection`, `preview`, `queue`, `status_bar`, `help_modal`)
- service modules for `s3`, `download`, `script`, `logging`, `config`
