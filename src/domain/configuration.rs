use std::collections::BTreeMap;
use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Claude,
    Codex,
    Copilot,
    Gemini,
}

impl RuntimeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Gemini => "gemini",
        }
    }
}

impl fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum InitTemplate {
    BugFix,
    Change,
    Delivery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum RouteSlot {
    Planning,
    Implementation,
    Verification,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ConfigShowScope {
    Effective,
    Workspace,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ConfigWriteScope {
    Workspace,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRoute {
    pub runtime: RuntimeKind,
    pub model: String,
}

impl ModelRoute {
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        if self.model.trim().is_empty() {
            return Err(ConfigurationError::MissingModelId);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoutingConfig {
    #[serde(default)]
    pub planning: Option<ModelRoute>,
    #[serde(default)]
    pub implementation: Option<ModelRoute>,
    #[serde(default)]
    pub verification: Option<ModelRoute>,
    #[serde(default)]
    pub review: Option<ModelRoute>,
    #[serde(default)]
    pub reviewer_roles: BTreeMap<String, ModelRoute>,
    #[serde(default)]
    pub adjudication: Option<ModelRoute>,
    #[serde(default)]
    pub assistant_runtimes: Vec<RuntimeKind>,
}

impl RoutingConfig {
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        for route in [
            self.planning.as_ref(),
            self.implementation.as_ref(),
            self.verification.as_ref(),
            self.review.as_ref(),
            self.adjudication.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            route.validate()?;
        }

        for (role, route) in &self.reviewer_roles {
            if role.trim().is_empty() {
                return Err(ConfigurationError::InvalidReviewerRole(
                    "role id cannot be empty".to_string(),
                ));
            }
            route.validate()?;
        }

        Ok(())
    }

    pub fn set_slot(&mut self, slot: RouteSlot, route: ModelRoute) {
        match slot {
            RouteSlot::Planning => self.planning = Some(route),
            RouteSlot::Implementation => self.implementation = Some(route),
            RouteSlot::Verification => self.verification = Some(route),
            RouteSlot::Review => self.review = Some(route),
        }
    }

    pub fn unset_slot(&mut self, slot: RouteSlot) {
        match slot {
            RouteSlot::Planning => self.planning = None,
            RouteSlot::Implementation => self.implementation = None,
            RouteSlot::Verification => self.verification = None,
            RouteSlot::Review => self.review = None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub routing: RoutingConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self { version: default_version(), routing: RoutingConfig::default() }
    }
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    Cli,
    Workspace,
    Global,
    BuiltIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedRoute {
    pub route: ModelRoute,
    pub source: ValueSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveRouting {
    pub planning: SourcedRoute,
    pub implementation: SourcedRoute,
    pub verification: SourcedRoute,
    pub review: SourcedRoute,
    pub adjudication: SourcedRoute,
    pub reviewer_roles: BTreeMap<String, SourcedRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RoutingOverrides {
    pub planning: Option<ModelRoute>,
    pub implementation: Option<ModelRoute>,
    pub verification: Option<ModelRoute>,
    pub review: Option<ModelRoute>,
    pub adjudication: Option<ModelRoute>,
    pub reviewer_roles: BTreeMap<String, ModelRoute>,
}

pub fn resolve_effective_routing(
    cli: &RoutingOverrides,
    workspace: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> EffectiveRouting {
    let defaults = built_in_defaults();

    let planning = resolve_single(
        cli.planning.as_ref(),
        workspace.and_then(|cfg| cfg.planning.as_ref()),
        global.and_then(|cfg| cfg.planning.as_ref()),
        &defaults.planning,
    );
    let implementation = resolve_single(
        cli.implementation.as_ref(),
        workspace.and_then(|cfg| cfg.implementation.as_ref()),
        global.and_then(|cfg| cfg.implementation.as_ref()),
        &defaults.implementation,
    );
    let verification = resolve_single(
        cli.verification.as_ref(),
        workspace.and_then(|cfg| cfg.verification.as_ref()),
        global.and_then(|cfg| cfg.verification.as_ref()),
        &defaults.verification,
    );
    let review = resolve_single(
        cli.review.as_ref(),
        workspace.and_then(|cfg| cfg.review.as_ref()),
        global.and_then(|cfg| cfg.review.as_ref()),
        &defaults.review,
    );
    let adjudication = resolve_single(
        cli.adjudication.as_ref(),
        workspace.and_then(|cfg| cfg.adjudication.as_ref()),
        global.and_then(|cfg| cfg.adjudication.as_ref()),
        &defaults.adjudication,
    );

    let mut reviewer_roles = BTreeMap::new();
    let mut role_ids = BTreeMap::<String, ()>::new();
    for key in cli
        .reviewer_roles
        .keys()
        .chain(workspace.into_iter().flat_map(|cfg| cfg.reviewer_roles.keys()))
        .chain(global.into_iter().flat_map(|cfg| cfg.reviewer_roles.keys()))
    {
        role_ids.insert(key.clone(), ());
    }

    for role_id in role_ids.into_keys() {
        let route = resolve_single(
            cli.reviewer_roles.get(&role_id),
            workspace.and_then(|cfg| cfg.reviewer_roles.get(&role_id)),
            global.and_then(|cfg| cfg.reviewer_roles.get(&role_id)),
            &review.route,
        );
        reviewer_roles.insert(role_id, route);
    }

    EffectiveRouting {
        planning,
        implementation,
        verification,
        review,
        adjudication,
        reviewer_roles,
    }
}

fn resolve_single(
    cli: Option<&ModelRoute>,
    workspace: Option<&ModelRoute>,
    global: Option<&ModelRoute>,
    default: &ModelRoute,
) -> SourcedRoute {
    if let Some(route) = cli {
        return SourcedRoute { route: route.clone(), source: ValueSource::Cli };
    }
    if let Some(route) = workspace {
        return SourcedRoute { route: route.clone(), source: ValueSource::Workspace };
    }
    if let Some(route) = global {
        return SourcedRoute { route: route.clone(), source: ValueSource::Global };
    }
    SourcedRoute { route: default.clone(), source: ValueSource::BuiltIn }
}

struct BuiltInDefaults {
    planning: ModelRoute,
    implementation: ModelRoute,
    verification: ModelRoute,
    review: ModelRoute,
    adjudication: ModelRoute,
}

fn built_in_defaults() -> BuiltInDefaults {
    BuiltInDefaults {
        planning: ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5-codex".to_string() },
        implementation: ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "gpt-5-codex".to_string(),
        },
        verification: ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() },
        review: ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
        adjudication: ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5-codex".to_string() },
    }
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("model id cannot be empty")]
    MissingModelId,
    #[error("invalid reviewer role: {0}")]
    InvalidReviewerRole(String),
}

#[cfg(test)]
mod tests {
    use super::{
        ModelRoute, RoutingConfig, RoutingOverrides, RuntimeKind, ValueSource,
        resolve_effective_routing,
    };

    #[test]
    fn cli_precedence_wins_over_workspace_and_global() {
        let cli = RoutingOverrides {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Gemini,
                model: "gemini-2.5-pro".to_string(),
            }),
            ..RoutingOverrides::default()
        };
        let workspace = RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "sonnet-4".to_string(),
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "gpt-5-codex".to_string(),
            }),
            ..RoutingConfig::default()
        };

        let resolved = resolve_effective_routing(&cli, Some(&workspace), Some(&global));
        assert_eq!(resolved.planning.source, ValueSource::Cli);
        assert_eq!(resolved.planning.route.runtime, RuntimeKind::Gemini);
    }

    #[test]
    fn review_role_falls_back_to_review_default() {
        let cli = RoutingOverrides::default();
        let workspace = RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "sonnet-4".to_string(),
            }),
            ..RoutingConfig::default()
        };
        let global = RoutingConfig::default();

        let resolved = resolve_effective_routing(&cli, Some(&workspace), Some(&global));
        assert!(resolved.reviewer_roles.is_empty());
        assert_eq!(resolved.review.route.runtime, RuntimeKind::Claude);
    }
}
