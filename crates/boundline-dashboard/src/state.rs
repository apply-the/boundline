use boundline_core::domain::dashboard::DashboardSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCapabilities {
    pub color: bool,
    pub interactive: bool,
}

impl TerminalCapabilities {
    pub const fn detect(no_color: bool) -> Self {
        Self { color: !no_color, interactive: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DashboardAppState {
    pub snapshot: Option<DashboardSnapshot>,
    pub action_in_progress: bool,
}

impl DashboardAppState {
    pub fn replace_snapshot(&mut self, snapshot: DashboardSnapshot) {
        self.snapshot = Some(snapshot);
    }

    pub fn begin_action(&mut self) -> bool {
        if self.action_in_progress {
            false
        } else {
            self.action_in_progress = true;
            true
        }
    }

    pub fn finish_action(&mut self) {
        self.action_in_progress = false;
    }
}
