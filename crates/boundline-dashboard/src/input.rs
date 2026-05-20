#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardInput {
    Refresh,
    Quit,
    FocusNext,
    FocusPrevious,
    Confirm,
    Reject,
    Replan,
    Recover,
    Launch,
    Continue,
    InspectOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardPanelFocus {
    Summary,
    GoalPlan,
    Evidence,
    Findings,
    Checkpoints,
    GovernedReferences,
    Diagnostics,
}

impl DashboardPanelFocus {
    pub const fn next(self) -> Self {
        match self {
            Self::Summary => Self::GoalPlan,
            Self::GoalPlan => Self::Evidence,
            Self::Evidence => Self::Findings,
            Self::Findings => Self::Checkpoints,
            Self::Checkpoints => Self::GovernedReferences,
            Self::GovernedReferences => Self::Diagnostics,
            Self::Diagnostics => Self::Summary,
        }
    }

    pub const fn previous(self) -> Self {
        match self {
            Self::Summary => Self::Diagnostics,
            Self::GoalPlan => Self::Summary,
            Self::Evidence => Self::GoalPlan,
            Self::Findings => Self::Evidence,
            Self::Checkpoints => Self::Findings,
            Self::GovernedReferences => Self::Checkpoints,
            Self::Diagnostics => Self::GovernedReferences,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardInputState {
    pub focus: DashboardPanelFocus,
    pub action_in_progress: bool,
}

impl Default for DashboardInputState {
    fn default() -> Self {
        Self { focus: DashboardPanelFocus::Summary, action_in_progress: false }
    }
}

impl DashboardInputState {
    pub fn apply_navigation(&mut self, input: DashboardInput) {
        match input {
            DashboardInput::FocusNext => {
                self.focus = self.focus.next();
            }
            DashboardInput::FocusPrevious => {
                self.focus = self.focus.previous();
            }
            _ => {}
        }
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
