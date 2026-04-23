use std::collections::BTreeSet;
use std::path::PathBuf;

use s3_grabber::action::Action;
use s3_grabber::app::{App, BrowserItem, BrowserItemKind, QueueJob, QueueJobStatus};

#[test]
fn selected_counts_split_objects_and_prefixes() {
    let mut app = App::new();
    app.browser.selected = BTreeSet::from([
        "logs/".to_string(),
        "reports/".to_string(),
        "README.txt".to_string(),
    ]);

    assert_eq!(app.selected_count(), 3);
    assert_eq!(app.selected_prefix_count(), 2);
    assert_eq!(app.selected_object_count(), 1);
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
fn queue_selected_direct_objects_builds_stable_queue_and_summary() {
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

    app.update(Action::QueueDownloadSelected);

    let queued_keys = app
        .queue
        .jobs
        .iter()
        .map(|job| job.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(queued_keys, vec!["a.txt", "z.txt"]);
    assert_eq!(app.queue.total_files, 2);
    assert_eq!(app.queue.total_bytes, 110);
    assert!(app
        .queue
        .summary
        .contains("selected 2 objects + 0 prefixes"));
    assert!(app.queue.summary.contains("=> 2 queued"));
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
