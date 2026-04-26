#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScriptMode {
    PerFile,
    PostBatch,
}

impl ScriptMode {
    pub fn label(self) -> &'static str {
        match self {
            ScriptMode::PerFile => "per-file",
            ScriptMode::PostBatch => "post-batch",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            ScriptMode::PerFile => ScriptMode::PostBatch,
            ScriptMode::PostBatch => ScriptMode::PerFile,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptRunner {
    pub command: String,
    pub mode: ScriptMode,
}

impl ScriptRunner {
    pub fn new(command: impl Into<String>, mode: ScriptMode) -> Self {
        Self {
            command: command.into(),
            mode,
        }
    }
}
