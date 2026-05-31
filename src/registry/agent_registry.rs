//! Runtime adapter registries for step agents and known framework-adapter profiles.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use thiserror::Error;

use crate::adapters::agent::{AgentAdapter, SharedAgentAdapter};
use crate::domain::configuration::{
    KnownAdapterProfileDefinition, KnownAdapterProfileFieldDefault,
};
use crate::domain::framework_adapter::FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1;

const SPECKIT_ADAPTER_ID: &str = "speckit";
const SPECKIT_DISPLAY_NAME: &str = "Speckit";
const SPECKIT_REGISTRATION_ALIAS: &str = "speckit";
const SPECKIT_DEFAULT_COMMAND: &str = "boundline-adapter-speckit";
const SPECKIT_ADAPTER_REPO_REF: &str = "../boundline-adapter-speckit";
const SPECKIT_TEMPLATE_REPO_REF: &str = "../boundline-framework-template";
const TEMPLATE_REPO_FIELD_KEY: &str = "template_repo";
const ADAPTER_REPO_FIELD_KEY: &str = "adapter_repo";

#[derive(Default, Clone)]
pub struct AgentRegistry {
    agents: HashMap<String, SharedAgentAdapter>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A>(&mut self, name: impl Into<String>, adapter: A) -> Result<(), RegistryError>
    where
        A: AgentAdapter + 'static,
    {
        self.register_shared(name, Arc::new(adapter))
    }

    pub fn register_shared(
        &mut self,
        name: impl Into<String>,
        adapter: SharedAgentAdapter,
    ) -> Result<(), RegistryError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(RegistryError::EmptyName);
        }

        self.agents.insert(name, adapter);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<SharedAgentAdapter> {
        self.agents.get(name).cloned()
    }
}

/// Host-owned registry of known framework-adapter profiles.
#[derive(Debug, Clone, Default)]
pub struct FrameworkAdapterProfileRegistry {
    profiles: BTreeMap<String, KnownAdapterProfileDefinition>,
    aliases: BTreeMap<String, String>,
    discovery_names: BTreeMap<String, String>,
}

impl FrameworkAdapterProfileRegistry {
    /// Returns an empty known-profile registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the host-shipped known-profile registry for the current release line.
    pub fn boundline_known_profiles() -> Result<Self, FrameworkAdapterRegistryError> {
        let mut registry = Self::new();
        registry.register_known_profile(speckit_known_profile())?;
        Ok(registry)
    }

    /// Registers one known framework-adapter profile.
    pub fn register_known_profile(
        &mut self,
        profile: KnownAdapterProfileDefinition,
    ) -> Result<(), FrameworkAdapterRegistryError> {
        validate_known_profile(&profile)?;

        if self.profiles.contains_key(&profile.adapter_id) {
            return Err(FrameworkAdapterRegistryError::DuplicateAdapterId(
                profile.adapter_id.clone(),
            ));
        }

        if self.aliases.contains_key(&profile.registration_alias) {
            return Err(FrameworkAdapterRegistryError::DuplicateRegistrationAlias(
                profile.registration_alias.clone(),
            ));
        }

        for discovery_name in &profile.discovery_names {
            if self.discovery_names.contains_key(discovery_name) {
                return Err(FrameworkAdapterRegistryError::DuplicateDiscoveryName(
                    discovery_name.clone(),
                ));
            }
        }

        for discovery_name in &profile.discovery_names {
            self.discovery_names.insert(discovery_name.clone(), profile.adapter_id.clone());
        }
        self.aliases.insert(profile.registration_alias.clone(), profile.adapter_id.clone());
        self.profiles.insert(profile.adapter_id.clone(), profile);
        Ok(())
    }

    /// Resolves a known profile by adapter ID.
    pub fn get_profile(&self, adapter_id: &str) -> Option<&KnownAdapterProfileDefinition> {
        self.profiles.get(adapter_id)
    }

    /// Resolves a known profile by adapter ID or registration alias.
    pub fn resolve_profile(&self, request: &str) -> Option<&KnownAdapterProfileDefinition> {
        self.get_profile(request).or_else(|| {
            self.aliases.get(request).and_then(|adapter_id| self.get_profile(adapter_id))
        })
    }

    /// Resolves a known profile from a discovered executable name.
    pub fn resolve_discovery_name(
        &self,
        executable_name: &str,
    ) -> Option<&KnownAdapterProfileDefinition> {
        self.discovery_names
            .get(executable_name)
            .and_then(|adapter_id| self.get_profile(adapter_id))
    }

    /// Returns all registered known profiles in deterministic order.
    pub fn profiles(&self) -> impl Iterator<Item = &KnownAdapterProfileDefinition> {
        self.profiles.values()
    }
}

/// Returns the host-shipped known profile definition for Speckit.
pub fn speckit_known_profile() -> KnownAdapterProfileDefinition {
    KnownAdapterProfileDefinition {
        adapter_id: SPECKIT_ADAPTER_ID.to_string(),
        display_name: SPECKIT_DISPLAY_NAME.to_string(),
        default_command: SPECKIT_DEFAULT_COMMAND.to_string(),
        registration_alias: SPECKIT_REGISTRATION_ALIAS.to_string(),
        adapter_repo_ref: SPECKIT_ADAPTER_REPO_REF.to_string(),
        template_repo_ref: SPECKIT_TEMPLATE_REPO_REF.to_string(),
        compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
        discovery_names: vec![SPECKIT_DEFAULT_COMMAND.to_string()],
        prefilled_fields: vec![
            KnownAdapterProfileFieldDefault {
                field_key: TEMPLATE_REPO_FIELD_KEY.to_string(),
                value_text: SPECKIT_TEMPLATE_REPO_REF.to_string(),
            },
            KnownAdapterProfileFieldDefault {
                field_key: ADAPTER_REPO_FIELD_KEY.to_string(),
                value_text: SPECKIT_ADAPTER_REPO_REF.to_string(),
            },
        ],
    }
}

fn validate_known_profile(
    profile: &KnownAdapterProfileDefinition,
) -> Result<(), FrameworkAdapterRegistryError> {
    if profile.adapter_id.trim().is_empty() {
        return Err(FrameworkAdapterRegistryError::EmptyAdapterId);
    }

    if profile.display_name.trim().is_empty() {
        return Err(FrameworkAdapterRegistryError::EmptyDisplayName(profile.adapter_id.clone()));
    }

    if profile.registration_alias.trim().is_empty() {
        return Err(FrameworkAdapterRegistryError::EmptyRegistrationAlias(
            profile.adapter_id.clone(),
        ));
    }

    if profile.default_command.trim().is_empty() {
        return Err(FrameworkAdapterRegistryError::EmptyDefaultCommand(profile.adapter_id.clone()));
    }

    if profile.compatibility_line.trim().is_empty() {
        return Err(FrameworkAdapterRegistryError::EmptyCompatibilityLine(
            profile.adapter_id.clone(),
        ));
    }

    for discovery_name in &profile.discovery_names {
        if discovery_name.trim().is_empty() {
            return Err(FrameworkAdapterRegistryError::EmptyDiscoveryName(
                profile.adapter_id.clone(),
            ));
        }
    }

    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistryError {
    #[error("registry names must not be empty")]
    EmptyName,
}

/// Errors returned while building or mutating the known framework-adapter registry.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FrameworkAdapterRegistryError {
    #[error("known framework-adapter profiles must not have an empty adapter_id")]
    EmptyAdapterId,
    #[error("known framework-adapter profile {0} must not have an empty display_name")]
    EmptyDisplayName(String),
    #[error("known framework-adapter profile {0} must not have an empty registration_alias")]
    EmptyRegistrationAlias(String),
    #[error("known framework-adapter profile {0} must not have an empty default_command")]
    EmptyDefaultCommand(String),
    #[error("known framework-adapter profile {0} must not have an empty compatibility_line")]
    EmptyCompatibilityLine(String),
    #[error("known framework-adapter profile {0} must not contain blank discovery names")]
    EmptyDiscoveryName(String),
    #[error("known framework-adapter profile {0} is already registered")]
    DuplicateAdapterId(String),
    #[error("known framework-adapter registration alias {0} is already registered")]
    DuplicateRegistrationAlias(String),
    #[error("known framework-adapter discovery name {0} is already registered")]
    DuplicateDiscoveryName(String),
}
