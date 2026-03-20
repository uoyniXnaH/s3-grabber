#[derive(Debug, Clone)]
pub struct S3ObjectSummary {
    pub key: String,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug, Default)]
pub struct S3Client;

impl S3Client {
    pub fn new() -> Self {
        Self
    }
}
