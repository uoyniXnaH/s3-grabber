#[derive(Debug, Clone)]
pub struct AppConfig {
    pub download_dir: String,
    pub preview_size_limit_bytes: usize,
    pub max_retries: u8,
    pub concurrency: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            download_dir: "./downloads".to_string(),
            preview_size_limit_bytes: 1_048_576,
            max_retries: 3,
            concurrency: 4,
        }
    }
}
