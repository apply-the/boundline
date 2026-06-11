//! Governance hooks for risky commands.
//!
//! Commands classified with high-risk intent or executed in red-zone
//! workspaces trigger governance hooks that can block execution,
//! require approval, or log to an audit channel.

use super::classifier::CommandIntent;
use super::evidence::RiskZone;

/// A governance hook that triggers on matching commands.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernanceHook {
    /// Unique hook identifier.
    pub id: String,
    /// Intents that trigger this hook.
    pub trigger_intents: Vec<CommandIntent>,
    /// Zones that trigger this hook.
    pub trigger_zones: Vec<RiskZone>,
    /// Action to take when triggered.
    pub action: HookAction,
}

/// Action taken by a governance hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookAction {
    /// Block execution entirely.
    Block,
    /// Require operator or system approval before execution.
    RequireApproval,
    /// Log the event but allow execution.
    Log,
}

/// Evaluates all hooks against a given intent and zone. Returns the
/// most restrictive action found (Block > RequireApproval > Log).
pub fn evaluate_hooks(
    hooks: &[GovernanceHook],
    intent: CommandIntent,
    zone: RiskZone,
) -> Option<HookAction> {
    let mut worst: Option<HookAction> = None;

    for hook in hooks {
        if hook.trigger_intents.contains(&intent) && hook.trigger_zones.contains(&zone) {
            worst = match (worst, hook.action) {
                (None, action) => Some(action),
                (Some(HookAction::Block), _) => Some(HookAction::Block),
                (Some(HookAction::RequireApproval), HookAction::Block) => Some(HookAction::Block),
                (Some(HookAction::RequireApproval), _) => Some(HookAction::RequireApproval),
                (Some(HookAction::Log), action) => Some(action),
            };
        }
    }

    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deploy_block_hook() -> GovernanceHook {
        GovernanceHook {
            id: "deploy-gate".into(),
            trigger_intents: vec![CommandIntent::Deploy],
            trigger_zones: vec![RiskZone::Red],
            action: HookAction::Block,
        }
    }

    fn mutate_approval_hook() -> GovernanceHook {
        GovernanceHook {
            id: "mutate-approval".into(),
            trigger_intents: vec![CommandIntent::Mutate],
            trigger_zones: vec![RiskZone::Yellow, RiskZone::Red],
            action: HookAction::RequireApproval,
        }
    }

    #[test]
    fn deploy_in_red_triggers_block() {
        let hooks = vec![deploy_block_hook()];
        let result = evaluate_hooks(&hooks, CommandIntent::Deploy, RiskZone::Red);
        assert_eq!(result, Some(HookAction::Block));
    }

    #[test]
    fn deploy_in_green_passes_through() {
        let hooks = vec![deploy_block_hook()];
        let result = evaluate_hooks(&hooks, CommandIntent::Deploy, RiskZone::Green);
        assert_eq!(result, None);
    }

    #[test]
    fn mutate_in_red_triggers_approval() {
        let hooks = vec![mutate_approval_hook()];
        let result = evaluate_hooks(&hooks, CommandIntent::Mutate, RiskZone::Red);
        assert_eq!(result, Some(HookAction::RequireApproval));
    }

    #[test]
    fn block_takes_priority_over_approval() {
        let hooks = vec![mutate_approval_hook(), deploy_block_hook()];
        let result = evaluate_hooks(&hooks, CommandIntent::Deploy, RiskZone::Red);
        assert_eq!(result, Some(HookAction::Block));
    }

    #[test]
    fn unknown_intent_passes_through() {
        let hooks = vec![deploy_block_hook()];
        let result = evaluate_hooks(&hooks, CommandIntent::Unknown, RiskZone::Red);
        assert_eq!(result, None);
    }

    #[test]
    fn approval_recorded_for_audit() {
        let hooks = vec![mutate_approval_hook()];
        let result = evaluate_hooks(&hooks, CommandIntent::Mutate, RiskZone::Yellow);
        assert_eq!(result, Some(HookAction::RequireApproval));
    }

    #[test]
    fn log_hook_triggers_log_action() {
        let hooks = vec![GovernanceHook {
            id: "audit-log".into(),
            trigger_intents: vec![CommandIntent::Read],
            trigger_zones: vec![RiskZone::Green],
            action: HookAction::Log,
        }];
        let result = evaluate_hooks(&hooks, CommandIntent::Read, RiskZone::Green);
        assert_eq!(result, Some(HookAction::Log));
    }

    #[test]
    fn require_approval_overrides_log() {
        let hooks = vec![
            GovernanceHook {
                id: "audit-log".into(),
                trigger_intents: vec![CommandIntent::Mutate],
                trigger_zones: vec![RiskZone::Yellow],
                action: HookAction::Log,
            },
            mutate_approval_hook(),
        ];
        let result = evaluate_hooks(&hooks, CommandIntent::Mutate, RiskZone::Yellow);
        assert_eq!(result, Some(HookAction::RequireApproval));
    }

    #[test]
    fn block_overrides_require_approval() {
        let hooks = vec![
            mutate_approval_hook(),
            GovernanceHook {
                id: "strict-block".into(),
                trigger_intents: vec![CommandIntent::Mutate],
                trigger_zones: vec![RiskZone::Red],
                action: HookAction::Block,
            },
        ];
        let result = evaluate_hooks(&hooks, CommandIntent::Mutate, RiskZone::Red);
        assert_eq!(result, Some(HookAction::Block));
    }

    #[test]
    fn empty_hooks_returns_none() {
        let result = evaluate_hooks(&[], CommandIntent::Deploy, RiskZone::Red);
        assert_eq!(result, None);
    }

    #[test]
    fn hook_action_serde() {
        let actions = [HookAction::Block, HookAction::RequireApproval, HookAction::Log];
        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let parsed: HookAction = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, action);
        }
    }
}
