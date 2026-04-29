#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Focus {
    Browser,
    WorkPane,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorkTab {
    Selection,
    Details,
    Queue,
    Logs,
}

impl WorkTab {
    pub const ALL: [WorkTab; 4] = [
        WorkTab::Selection,
        WorkTab::Details,
        WorkTab::Queue,
        WorkTab::Logs,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WorkTab::Selection => "Selection",
            WorkTab::Details => "Details",
            WorkTab::Queue => "Queue",
            WorkTab::Logs => "Logs",
        }
    }

    pub fn next(self) -> Self {
        match self {
            WorkTab::Selection => WorkTab::Details,
            WorkTab::Details => WorkTab::Queue,
            WorkTab::Queue => WorkTab::Logs,
            WorkTab::Logs => WorkTab::Selection,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            WorkTab::Selection => WorkTab::Logs,
            WorkTab::Details => WorkTab::Selection,
            WorkTab::Queue => WorkTab::Details,
            WorkTab::Logs => WorkTab::Queue,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    QuitRequested,
    CancelDialog,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Enter,
    BackspaceKey,
    FocusNext,
    ToggleSelectCurrent,
    NextTab,
    PreviousTab,
    QueueDownloadSelected,
    QueueDownloadFolder,
    RunScript,
    Refresh,
    OpenFilter,
    ToggleHelp,
    InputChar(char),
    Tick,
}
