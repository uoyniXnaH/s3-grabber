#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub key: String,
    pub status: JobStatus,
}

#[derive(Debug, Default)]
pub struct DownloadManager {
    pub jobs: Vec<DownloadJob>,
}
