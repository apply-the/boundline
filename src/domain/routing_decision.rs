use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::configuration::{EffectiveRouting, RuntimeKind, SourcedRoute, ValueSource};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoutingDecisionProjection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effective_routing: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assistant_bindings: Vec<String>,
}

impl RoutingDecisionProjection {
    pub fn from_effective_routing(routing: &EffectiveRouting) -> Self {
        let mut effective_routing = Vec::new();
        let mut assistant_bindings = Vec::new();

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

        Self { effective_routing, assistant_bindings }
    }

    pub fn is_empty(&self) -> bool {
        self.effective_routing.is_empty() && self.assistant_bindings.is_empty()
    }

    pub fn projection_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.effective_routing.is_empty() {
            lines.push(format!("effective_routing: {}", self.effective_routing.join(", ")));
        }
        if !self.assistant_bindings.is_empty() {
            lines.push(format!("assistant_bindings: {}", self.assistant_bindings.join(", ")));
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

        let projection = Self { effective_routing, assistant_bindings };
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
    use serde_json::json;

    use crate::domain::configuration::{EffectiveRouting, RuntimeKind, SourcedRoute, ValueSource};

    use super::RoutingDecisionProjection;

    #[test]
    fn parses_projection_from_task_input() {
        let projection = RoutingDecisionProjection::from_task_input(&json!({
            "routing_projection": {
                "effective_routing": ["planning=codex/gpt-5-codex [workspace]"],
                "assistant_bindings": ["planning=codex"]
            }
        }))
        .unwrap();

        assert_eq!(
            projection.effective_routing,
            vec!["planning=codex/gpt-5-codex [workspace]".to_string()]
        );
        assert_eq!(projection.assistant_bindings, vec!["planning=codex".to_string()]);
    }

    #[test]
    fn builds_projection_from_effective_routing() {
        let routing = EffectiveRouting {
            planning: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "gpt-5-codex".to_string(),
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
                    model: "gpt-5.4".to_string(),
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
            adjudication: SourcedRoute {
                route: crate::domain::configuration::ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "gpt-5-codex".to_string(),
                },
                source: ValueSource::Cli,
            },
            reviewer_roles: Default::default(),
        };

        let projection = RoutingDecisionProjection::from_effective_routing(&routing);

        assert!(
            projection
                .effective_routing
                .contains(&"planning=codex/gpt-5-codex [workspace]".to_string())
        );
        assert!(
            projection
                .effective_routing
                .contains(&"implementation=gemini/gemini-2.5-pro [built-in]".to_string())
        );
        assert!(projection.assistant_bindings.contains(&"implementation=gemini-cli".to_string()));
    }
}
