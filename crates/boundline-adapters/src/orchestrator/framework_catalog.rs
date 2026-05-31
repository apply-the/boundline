//! Host-owned stage and hook identifiers for the framework-adapter protocol.

use serde::{Deserialize, Serialize};

/// Stable protocol line identifier for the initial framework-adapter contract.
pub const FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1: &str = "framework-adapter-v1";

/// Host-owned lifecycle stages that adapters may claim in the initial slice.
pub const FRAMEWORK_STAGE_CATALOG: [FrameworkStageKey; 4] = [
    FrameworkStageKey::Goal,
    FrameworkStageKey::Plan,
    FrameworkStageKey::Run,
    FrameworkStageKey::Review,
];

/// Host-owned hook identifiers that adapters may subscribe to in the initial slice.
pub const FRAMEWORK_HOOK_CATALOG: [FrameworkHookKey; 2] =
    [FrameworkHookKey::StageCompleted, FrameworkHookKey::StageFailed];

/// Stable lifecycle-stage identifiers accepted by the adapter protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkStageKey {
    /// Initial goal definition and framing stage.
    Goal,
    /// Planning stage that turns the goal into an execution plan.
    Plan,
    /// Execution stage that applies the planned changes.
    Run,
    /// Review stage that verifies the bounded result.
    Review,
}

impl FrameworkStageKey {
    /// Returns the stable serialized identifier for the stage key.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Goal => "goal",
            Self::Plan => "plan",
            Self::Run => "run",
            Self::Review => "review",
        }
    }
}

/// Stable hook identifiers accepted by the adapter protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkHookKey {
    /// Hook fired after a stage finishes successfully.
    StageCompleted,
    /// Hook fired after a stage finishes with failure semantics.
    StageFailed,
}

impl FrameworkHookKey {
    /// Returns the stable serialized identifier for the hook key.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StageCompleted => "stage_completed",
            Self::StageFailed => "stage_failed",
        }
    }
}
