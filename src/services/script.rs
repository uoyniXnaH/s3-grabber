#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ScriptMode {
    PerFile,
    PostBatch,
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
