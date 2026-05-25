use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::configuration::{
    EffectiveRouting, RouteSlot, RuntimeKind, SourcedRoute, SourcedRuntimeCapabilityProfile,
    SourcedSlotEffortPolicy, ValueSource,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoutingDecisionProjection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effective_routing: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assistant_bindings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slot_effort_policies: Vec<String>,
}

impl RoutingDecisionProjection {
    pub fn from_effective_routing(routing: &EffectiveRouting) -> Self {
        Self::from_effective_state(routing, &BTreeMap::new(), &BTreeMap::new())
    }

    pub fn from_effective_state(
        routing: &EffectiveRouting,
        runtime_capabilities: &BTreeMap<RuntimeKind, SourcedRuntimeCapabilityProfile>,
        slot_effort_policies: &BTreeMap<RouteSlot, SourcedSlotEffortPolicy>,
    ) -> Self {
        let mut effective_routing = Vec::new();
        let mut assistant_bindings = Vec::new();
        let mut runtime_capabilities_projection = Vec::new();
        let mut slot_effort_policies_projection = Vec::new();

        push_projection_entry(
            "planning",
            &routing.planning,
            &mut effective_routing,
            &mut assistant_bindings,
        );
        push_projection_entry(
            "implementation",
            &routing.implementation,
            &mut effective_routing,
            &mut assistant_bindings,
        );
        push_projection_entry(
            "verification",
            &routing.verification,
            &mut effective_routing,
            &mut assistant_bindings,
        );
        push_projection_entry(
            "review",
            &routing.review,
            &mut effective_routing,
            &mut assistant_bindings,
        );
        push_projection_entry(
            "adjudication",
            &routing.adjudication,
            &mut effective_routing,
            &mut assistant_bindings,
        );

        for (role, route) in &routing.reviewer_roles {
            push_projection_entry(
                &format!("reviewer:{role}"),
                route,
                &mut effective_routing,
                &mut assistant_bindings,
            );
        }

        for (runtime, profile) in runtime_capabilities {
            runtime_capabilities_projection.push(format!(
                "{}={} [{}]",
                runtime.as_str(),
                profile.profile.summary_text(),
                value_source_text(profile.source)
            ));
        }

        for (slot, policy) in slot_effort_policies {
            slot_effort_policies_projection.push(format!(
                "{}={} [{}]",
                slot.as_str(),
                policy.policy.summary_text(),
                value_source_text(policy.source)
            ));
        }

        Self {
            effective_routing,
            assistant_bindings,
            runtime_capabilities: runtime_capabilities_projection,
            slot_effort_policies: slot_effort_policies_projection,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.effective_routing.is_empty()
            && self.assistant_bindings.is_empty()
            && self.runtime_capabilities.is_empty()
            && self.slot_effort_policies.is_empty()
    }

    pub fn projection_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.effective_routing.is_empty() {
            lines.push(format!("effective_routing: {}", self.effective_routing.join(", ")));
        }
        if !self.assistant_bindings.is_empty() {
            lines.push(format!("assistant_bindings: {}", self.assistant_bindings.join(", ")));
        }
        if !self.runtime_capabilities.is_empty() {
            lines.push(format!("runtime_capabilities: {}", self.runtime_capabilities.join(", ")));
        }
        if !self.slot_effort_policies.is_empty() {
            lines.push(format!("slot_effort_policies: {}", self.slot_effort_policies.join(", ")));
        }
        lines
    }

    pub fn from_event_payload(payload: &Value) -> Option<Self> {
        payload
            .get("input")
            .and_then(Self::from_task_input)
            .or_else(|| payload.get("routing_projection").and_then(Self::from_value))
    }

    pub fn from_task_input(input: &Value) -> Option<Self> {
        input.get("routing_projection").and_then(Self::from_value)
    }

    pub fn from_value(value: &Value) -> Option<Self> {
        let effective_routing =
            value.get("effective_routing").map(string_array).unwrap_or_default();
        let assistant_bindings =
            value.get("assistant_bindings").map(string_array).unwrap_or_default();
        let runtime_capabilities =
            value.get("runtime_capabilities").map(string_array).unwrap_or_default();
        let slot_effort_policies =
            value.get("slot_effort_policies").map(string_array).unwrap_or_default();

        let projection = Self {
            effective_routing,
            assistant_bindings,
            runtime_capabilities,
            slot_effort_policies,
        };
        (!projection.is_empty()).then_some(projection)
    }
}

fn push_projection_entry(
    slot: &str,
    route: &SourcedRoute,
    effective_routing: &mut Vec<String>,
    assistant_bindings: &mut Vec<String>,
) {
    effective_routing.push(format!(
        "{slot}={}/{} [{}]",
        route.route.runtime.as_str(),
        route.route.model,
        value_source_text(route.source)
    ));
    assistant_bindings.push(format!("{slot}={}", assistant_binding_label(route.route.runtime)));
}

fn value_source_text(source: ValueSource) -> &'static str {
    match source {
        ValueSource::Cli => "cli",
        ValueSource::Workspace => "workspace",
        ValueSource::Cluster => "cluster",
        ValueSource::Global => "global",
        ValueSource::BuiltIn => "built-in",
    }
}

fn assistant_binding_label(runtime: RuntimeKind) -> &'static str {
    match runtime {
        RuntimeKind::Gemini => "gemini-cli",
        _ => runtime.as_str(),
    }
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use crate::domain::configuration::{
        CapabilityState, EffectiveRouting, EffortFallbackPolicy, EffortLevel, RouteSlot,
        RuntimeCapabilityProfile, RuntimeKind, SlotEffortPolicy, SourcedRoute,
        SourcedRuntimeCapabilityProfile, SourcedSlotEffortPolicy, ValueSource,
    };

    use super::RoutingDecisionProjection;

    #[test]
    fn parses_projection_from_task_input() {
        let projection = RoutingDecisionProjection::from_task_input(&json!({
            "routing_projection": {
                "effective_routing": ["planning=codex/o4-mini [workspace]"],
                "assistant_bindings": ["planning=codex"],
                "runtime_capabilities": ["codex=continuation=supported [workspace]"],
                "slot_effort_policies": ["implementation=level=high, fallback=preserve [global]"]
            }
        }))
        .unwrap();

        assert_eq!(
            projection.effective_routing,
            vec!["planning=codex/o4-mini [workspace]".to_string()]
        );
        assert_eq!(projection.assistant_bindings, vec!["planning=codex".to_string()]);
        assert_eq!(
            projection.runtime_capabilities,
            vec!["codex=continuation=supported [workspace]".to_string()]
        );
        assert_eq!(
            projection.slot_effort_policies,
            vec!["implementation=level=high, fallback=preserve [global]".to_string()]
        );
    }

    #[test]
    fn builds_projection_from_effective_state() {
        let routing = EffectiveRouting {
            planning: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "o4-mini".to_string(),
                },
                source: ValueSource::Workspace,
            },
            implementation: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Gemini,
                    model: "gemini-2.5-pro".to_string(),
                },
                source: ValueSource::BuiltIn,
            },
            verification: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-4o".to_string(),
                },
                source: ValueSource::Global,
            },
            review: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Claude,
                    model: "sonnet-4".to_string(),
                },
                source: ValueSource::Cluster,
            },
            chat: None,
            adjudication: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "o4-mini".to_string(),
                },
                source: ValueSource::Cli,
            },
            reviewer_roles: Default::default(),
        };

        let runtime_capabilities = BTreeMap::from([
            (
                RuntimeKind::Claude,
                SourcedRuntimeCapabilityProfile {
                    profile: RuntimeCapabilityProfile {
                        continuation: CapabilityState::Unsupported,
                        resume: CapabilityState::Unsupported,
                        validation: CapabilityState::Supported,
                        handoff_target: CapabilityState::Unsupported,
                        escalation_context: CapabilityState::Supported,
                        notes: None,
                    },
                    source: ValueSource::Workspace,
                },
            ),
            (
                RuntimeKind::Codex,
                SourcedRuntimeCapabilityProfile {
                    profile: RuntimeCapabilityProfile {
                        continuation: CapabilityState::Supported,
                        resume: CapabilityState::Supported,
                        validation: CapabilityState::Supported,
                        handoff_target: CapabilityState::Supported,
                        escalation_context: CapabilityState::Supported,
                        notes: Some("preferred handoff target".to_string()),
                    },
                    source: ValueSource::Global,
                },
            ),
        ]);
        let slot_effort_policies = BTreeMap::from([(
            RouteSlot::Implementation,
            SourcedSlotEffortPolicy {
                policy: SlotEffortPolicy {
                    level: EffortLevel::High,
                    fallback: EffortFallbackPolicy::Preserve,
                    rationale: Some("keep implementation reviews thorough".to_string()),
                },
                source: ValueSource::Cluster,
            },
        )]);

        let projection = RoutingDecisionProjection::from_effective_state(
            &routing,
            &runtime_capabilities,
            &slot_effort_policies,
        );

        assert!(
            projection
                .effective_routing
                .contains(&"planning=codex/o4-mini [workspace]".to_string())
        );
        assert!(
            projection
                .effective_routing
                .contains(&"implementation=gemini/gemini-2.5-pro [built-in]".to_string())
        );
        assert!(projection.assistant_bindings.contains(&"implementation=gemini-cli".to_string()));
        assert!(projection.runtime_capabilities.iter().any(|line| {
            line.contains("claude=continuation=unsupported") && line.ends_with("[workspace]")
        }));
        assert!(projection.slot_effort_policies.iter().any(|line| {
            line.contains("implementation=level=high, fallback=preserve")
                && line.ends_with("[cluster]")
        }));
    }
}
