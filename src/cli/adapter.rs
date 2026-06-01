//! Adapter command execution for registration, inspection, and removal.

use std::collections::BTreeMap;
use std::env;
use std::io::{self, IsTerminal};
use std::path::Component;
use std::path::Path;

use dialoguer::{Input, theme::ColorfulTheme};
use thiserror::Error;

use crate::adapters::agent::{
    FrameworkAdapterHost, FrameworkAdapterHostError, SubprocessFrameworkAdapterHost,
};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::adapters::{
    FrameworkAdapterConfigFieldDefinition, FrameworkAdapterConfigValue,
    FrameworkAdapterDescribeResponse, FrameworkAdapterFieldValueKind,
    FrameworkAdapterPreflightBlockReason, FrameworkAdapterPreflightRequest,
    FrameworkAdapterPreflightResponse, FrameworkAdapterPreflightStatus,
    format_framework_adapter_transports, framework_adapter_supports_v1_transport,
};
use crate::cli::CommandExitStatus;
use crate::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, KnownAdapterProfileDefinition,
    PersistedAdapterConfiguration,
};
use crate::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    StoredAdapterConfigValueState,
};
use crate::domain::trace::current_timestamp_millis;
use crate::registry::agent_registry::{
    FrameworkAdapterProfileRegistry, FrameworkAdapterRegistryError,
};

const STATUS_BUILT_IN_DEFAULT: &str = "built_in_default";
const STATUS_READY: &str = "ready";
const STATUS_BLOCKED: &str = "blocked";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_REMOVED: &str = "removed";
const EXECUTION_SOURCE_BUILT_IN: &str = "built_in";
const REASON_ADAPTER_ALREADY_SELECTED: &str = "adapter_already_selected";
const REASON_MISSING_REQUIRED_CONFIG: &str = "missing_required_config";
const REASON_INCOMPATIBLE_PROTOCOL: &str = "incompatible_protocol";
const REASON_UNEXPECTED_ADAPTER_ID: &str = "unexpected_adapter_id";
const REASON_UNSUPPORTED_TRANSPORT: &str = "unsupported_transport";
const REASON_UNAVAILABLE_BINARY: &str = "unavailable_binary";
const REASON_SETUP_CANCELLED: &str = "setup_cancelled";
const CUSTOM_PROFILE_NAME: &str = "custom";
const REMOVE_RECOVERY_TEMPLATE: &str = "boundline adapter remove --workspace";
const ADD_RECOVERY_TEMPLATE: &str = "boundline adapter add";
const CONFIG_SHOW_TEMPLATE: &str = "boundline config show --workspace";
const CONFIG_STATE_COMPLETE: &str = "complete";
const CONFIG_STATE_MISSING_REQUIRED: &str = "missing_required";
const CONFIG_STATE_INVALID: &str = "invalid";
const CURRENT_BOUNDLINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Rendered result for adapter CLI commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

/// Minimal prompt interface used by guided adapter setup.
pub trait AdapterConfigInteractor: std::fmt::Debug {
    fn input(&mut self, prompt: &str, initial: &str, secret: bool) -> Result<String, String>;
}

#[derive(Debug, Default)]
struct DialoguerAdapterConfigInteractor;

impl AdapterConfigInteractor for DialoguerAdapterConfigInteractor {
    fn input(&mut self, prompt: &str, initial: &str, secret: bool) -> Result<String, String> {
        let _ = secret;
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .with_initial_text(initial.to_string())
            .allow_empty(true)
            .interact_text()
            .map_err(|error| error.to_string())
    }
}

/// Input for `boundline adapter add`.
#[derive(Debug)]
pub struct AddAdapterRequest<'a> {
    pub profile: &'a str,
    pub workspace: Option<&'a Path>,
    pub id: Option<&'a str>,
    pub command: Option<&'a str>,
    pub arg: &'a [String],
    pub set: &'a [String],
    pub non_interactive: bool,
    /// Override TTY detection for testing. `None` means auto-detect from stdin/stdout.
    pub interactive_terminal_override: Option<bool>,
    /// Inject a custom interactor for guided adapter setup tests.
    pub interactor: Option<Box<dyn AdapterConfigInteractor>>,
}

/// Input for `boundline adapter show`.
#[derive(Debug, Clone, Copy)]
pub struct ShowAdapterRequest<'a> {
    pub workspace: Option<&'a Path>,
}

/// Input for `boundline adapter remove`.
#[derive(Debug, Clone, Copy)]
pub struct RemoveAdapterRequest<'a> {
    pub workspace: Option<&'a Path>,
}

#[derive(Debug, Clone)]
struct AdapterRegistrationPlan {
    selection_mode: AdapterSelectionMode,
    adapter_id: String,
    display_name: String,
    command: String,
    args: Vec<String>,
    discovery_state: AdapterDiscoveryState,
    compatibility_line: String,
    profile_defaults: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct PreparedConfigValues {
    protocol_values: Vec<FrameworkAdapterConfigValue>,
    stored_values: Vec<AdapterConfigValueRecord>,
    missing_fields: Vec<String>,
    schema_fingerprint: String,
    interactive_resolution: bool,
}

#[derive(Debug, Error)]
enum AdapterCommandError {
    #[error("adapter commands require --workspace")]
    WorkspaceRequired,
    #[error("failed to read or write adapter config: {0}")]
    ConfigStore(#[from] ConfigStoreError),
    #[error("failed to resolve known adapter profiles: {0}")]
    Registry(#[from] FrameworkAdapterRegistryError),
    #[error("failed to run framework adapter: {0}")]
    Host(#[from] FrameworkAdapterHostError),
    #[error("unknown adapter profile `{0}`")]
    UnknownProfile(String),
    #[error("custom adapter registration requires --id")]
    CustomIdRequired,
    #[error("custom adapter registration requires --command")]
    CustomCommandRequired,
    #[error("invalid --set value `{entry}`; expected key=value")]
    InvalidSetValue { entry: String },
    #[error("invalid value for `{field_key}`: expected {expected}")]
    InvalidFieldValue { field_key: String, expected: &'static str },
    #[error(
        "guided adapter setup requires an interactive terminal or --non-interactive with all required --set values"
    )]
    InteractiveTerminalUnavailable,
    #[error("{0}")]
    PromptInteraction(String),
}

/// Executes `boundline adapter add`.
pub fn execute_add(request: AddAdapterRequest<'_>) -> AdapterCommandReport {
    match execute_add_inner(request, AdapterRegistrationSource::AdapterAdd) {
        Ok(report) => report,
        Err(error) => command_error_report("add", error),
    }
}

pub(crate) fn execute_add_from_init(request: AddAdapterRequest<'_>) -> AdapterCommandReport {
    match execute_add_inner(request, AdapterRegistrationSource::Init) {
        Ok(report) => report,
        Err(error) => command_error_report("add", error),
    }
}

/// Executes `boundline adapter show`.
pub fn execute_show(request: ShowAdapterRequest<'_>) -> AdapterCommandReport {
    match execute_show_inner(request) {
        Ok(report) => report,
        Err(error) => command_error_report("show", error),
    }
}

/// Executes `boundline adapter remove`.
pub fn execute_remove(request: RemoveAdapterRequest<'_>) -> AdapterCommandReport {
    match execute_remove_inner(request) {
        Ok(report) => report,
        Err(error) => command_error_report("remove", error),
    }
}

fn execute_add_inner(
    mut request: AddAdapterRequest<'_>,
    registration_source: AdapterRegistrationSource,
) -> Result<AdapterCommandReport, AdapterCommandError> {
    let workspace = required_workspace(request.workspace)?;
    let interactive_terminal = request
        .interactive_terminal_override
        .unwrap_or_else(|| io::stdin().is_terminal() && io::stdout().is_terminal());
    let store = FileConfigStore::for_workspace(workspace);
    if let Some(existing) = store.local_adapter()? {
        return Ok(existing_adapter_blocked_report(&existing.selection.adapter_id, workspace));
    }

    let plan = resolve_registration_plan(&request)?;
    let host = adapter_host(&plan.command, &plan.args, workspace)?;
    let describe = match host.describe() {
        Ok(describe) => describe,
        Err(error) => {
            if let Some(report) =
                unavailable_binary_report(&plan, request.profile, workspace, &error)
            {
                return Ok(report);
            }
            return Err(error.into());
        }
    };

    if describe.protocol_line != plan.compatibility_line {
        return Ok(blocked_report(
            &plan.adapter_id,
            REASON_INCOMPATIBLE_PROTOCOL,
            Some(format!(
                "{ADD_RECOVERY_TEMPLATE} {} --workspace {}",
                request.profile,
                workspace.display()
            )),
            Vec::new(),
        ));
    }

    if plan.selection_mode == AdapterSelectionMode::KnownProfile
        && describe.adapter_id != plan.adapter_id
    {
        return Ok(blocked_report(
            &plan.adapter_id,
            REASON_UNEXPECTED_ADAPTER_ID,
            Some(format!(
                "{ADD_RECOVERY_TEMPLATE} {} --workspace {}",
                request.profile,
                workspace.display()
            )),
            Vec::new(),
        ));
    }

    if !framework_adapter_supports_v1_transport(&describe.supported_transports) {
        return Ok(unsupported_transport_report(
            &plan.adapter_id,
            request.profile,
            workspace,
            &describe,
        ));
    }

    let mut prepared = prepare_config_values(&plan, request.set, &describe)?;
    if !prepared.missing_fields.is_empty() && !request.non_interactive {
        if !interactive_terminal {
            return Err(AdapterCommandError::InteractiveTerminalUnavailable);
        }
        prepared = match collect_guided_config_values(
            &plan,
            request.set,
            &describe,
            &prepared,
            request.interactor.as_mut(),
        ) {
            Ok(prepared) => prepared,
            Err(AdapterCommandError::PromptInteraction(message))
                if is_guided_cancel_message(&message) =>
            {
                return Ok(cancelled_report(&plan.adapter_id, workspace, &request, &message));
            }
            Err(error) => return Err(error),
        };
    }
    if !prepared.missing_fields.is_empty() {
        return Ok(blocked_report(
            &plan.adapter_id,
            REASON_MISSING_REQUIRED_CONFIG,
            Some(format!(
                "{ADD_RECOVERY_TEMPLATE} {} --workspace {}",
                request.profile,
                workspace.display()
            )),
            prepared.missing_fields,
        ));
    }

    let preflight = host.preflight(&FrameworkAdapterPreflightRequest {
        boundline_version: CURRENT_BOUNDLINE_VERSION.to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        non_interactive: request.non_interactive,
        config_values: prepared.protocol_values.clone(),
    })?;

    if preflight.status == FrameworkAdapterPreflightStatus::Blocked {
        return Ok(blocked_preflight_report(&plan.adapter_id, &preflight));
    }

    let persisted =
        build_persisted_configuration(&plan, &prepared, &preflight, registration_source);
    save_adapter_selection(&store, persisted.clone())?;
    Ok(ready_report(&persisted, &describe))
}

fn execute_show_inner(
    request: ShowAdapterRequest<'_>,
) -> Result<AdapterCommandReport, AdapterCommandError> {
    let workspace = required_workspace(request.workspace)?;
    let store = FileConfigStore::for_workspace(workspace);
    let Some(adapter) = store.local_adapter()? else {
        let registry = FrameworkAdapterProfileRegistry::boundline_known_profiles()?;
        return Ok(built_in_default_report(&registry));
    };

    let describe =
        adapter_host(&adapter.selection.command, &adapter.selection.args, workspace)?.describe()?;
    Ok(configured_show_report(&adapter, &describe))
}

fn execute_remove_inner(
    request: RemoveAdapterRequest<'_>,
) -> Result<AdapterCommandReport, AdapterCommandError> {
    let workspace = required_workspace(request.workspace)?;
    let store = FileConfigStore::for_workspace(workspace);
    let Some(mut config) = store.load_local()? else {
        return Ok(removed_report());
    };

    config.adapter = None;
    store.save_local(&config)?;
    Ok(removed_report())
}

fn required_workspace(workspace: Option<&Path>) -> Result<&Path, AdapterCommandError> {
    workspace.ok_or(AdapterCommandError::WorkspaceRequired)
}

fn resolve_registration_plan(
    request: &AddAdapterRequest<'_>,
) -> Result<AdapterRegistrationPlan, AdapterCommandError> {
    if request.profile == CUSTOM_PROFILE_NAME {
        return resolve_custom_registration_plan(request);
    }

    let registry = FrameworkAdapterProfileRegistry::boundline_known_profiles()?;
    let profile = registry
        .resolve_profile(request.profile)
        .ok_or_else(|| AdapterCommandError::UnknownProfile(request.profile.to_string()))?;
    Ok(known_profile_registration_plan(profile, request))
}

fn resolve_custom_registration_plan(
    request: &AddAdapterRequest<'_>,
) -> Result<AdapterRegistrationPlan, AdapterCommandError> {
    let adapter_id = request.id.ok_or(AdapterCommandError::CustomIdRequired)?;
    let command = request.command.ok_or(AdapterCommandError::CustomCommandRequired)?;
    Ok(AdapterRegistrationPlan {
        selection_mode: AdapterSelectionMode::Custom,
        adapter_id: adapter_id.to_string(),
        display_name: adapter_id.to_string(),
        command: command.to_string(),
        args: request.arg.to_vec(),
        discovery_state: AdapterDiscoveryState::ExplicitCommand,
        compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
        profile_defaults: BTreeMap::new(),
    })
}

fn known_profile_registration_plan(
    profile: &KnownAdapterProfileDefinition,
    request: &AddAdapterRequest<'_>,
) -> AdapterRegistrationPlan {
    let command = request.command.unwrap_or(&profile.default_command);
    let discovery_state = if request.command.is_some() {
        AdapterDiscoveryState::ExplicitCommand
    } else if command_exists_on_path(command) {
        AdapterDiscoveryState::DiscoveredOnPath
    } else {
        AdapterDiscoveryState::Unresolved
    };

    AdapterRegistrationPlan {
        selection_mode: AdapterSelectionMode::KnownProfile,
        adapter_id: profile.adapter_id.clone(),
        display_name: profile.display_name.clone(),
        command: command.to_string(),
        args: request.arg.to_vec(),
        discovery_state,
        compatibility_line: profile.compatibility_line.clone(),
        profile_defaults: profile
            .prefilled_fields
            .iter()
            .map(|field| (field.field_key.clone(), field.value_text.clone()))
            .collect(),
    }
}

fn prepare_config_values(
    plan: &AdapterRegistrationPlan,
    set_values: &[String],
    describe: &FrameworkAdapterDescribeResponse,
) -> Result<PreparedConfigValues, AdapterCommandError> {
    let overrides = parse_set_values(set_values)?;

    prepare_config_values_from_overrides(plan, &overrides, describe, false)
}

fn prepare_config_values_from_overrides(
    plan: &AdapterRegistrationPlan,
    overrides: &BTreeMap<String, String>,
    describe: &FrameworkAdapterDescribeResponse,
    interactive_resolution: bool,
) -> Result<PreparedConfigValues, AdapterCommandError> {
    let mut protocol_values = Vec::new();
    let mut stored_values = Vec::new();
    let mut missing_fields = Vec::new();

    for field in &describe.required_config_fields {
        match resolve_field_value_text(field, plan, overrides) {
            Some((value_text, value_source)) => {
                let protocol_value = protocol_value_from_text(field, &value_text)?;
                stored_values.push(stored_value_record(field, &protocol_value, value_source));
                protocol_values.push(protocol_value);
            }
            None if field.required => missing_fields.push(field.field_key.clone()),
            None => {}
        }
    }

    Ok(PreparedConfigValues {
        protocol_values,
        stored_values,
        missing_fields,
        schema_fingerprint: config_schema_fingerprint(describe),
        interactive_resolution,
    })
}

fn collect_guided_config_values(
    plan: &AdapterRegistrationPlan,
    set_values: &[String],
    describe: &FrameworkAdapterDescribeResponse,
    prepared: &PreparedConfigValues,
    interactor: Option<&mut Box<dyn AdapterConfigInteractor>>,
) -> Result<PreparedConfigValues, AdapterCommandError> {
    let mut overrides = parse_set_values(set_values)?;
    let mut prompted_fields = Vec::new();
    let mut default_interactor: Box<dyn AdapterConfigInteractor> =
        Box::new(DialoguerAdapterConfigInteractor);
    let interactor: &mut dyn AdapterConfigInteractor = match interactor {
        Some(interactor) => interactor.as_mut(),
        None => default_interactor.as_mut(),
    };

    for field_key in &prepared.missing_fields {
        let field = describe
            .required_config_fields
            .iter()
            .find(|field| field.field_key == *field_key)
            .ok_or_else(|| {
                AdapterCommandError::PromptInteraction(format!(
                    "missing config prompt definition for `{field_key}`"
                ))
            })?;
        let prompt = guided_field_prompt(field);
        let initial = field.default_value_text.as_deref().unwrap_or_default();
        let value = interactor
            .input(&prompt, initial, field.secret)
            .map_err(AdapterCommandError::PromptInteraction)?;
        overrides.insert(field.field_key.clone(), value);
        prompted_fields.push(field.field_key.clone());
    }

    let mut prepared = prepare_config_values_from_overrides(plan, &overrides, describe, true)?;
    for value in &mut prepared.stored_values {
        if prompted_fields.contains(&value.field_key) {
            value.value_source = AdapterValueSource::OperatorPrompt;
        }
    }
    Ok(prepared)
}

fn guided_field_prompt(field: &FrameworkAdapterConfigFieldDefinition) -> String {
    let prompt_text = if field.prompt_text.trim().is_empty() {
        field.display_label.as_str()
    } else {
        field.prompt_text.as_str()
    };

    if field.help_text.trim().is_empty() {
        prompt_text.to_string()
    } else {
        format!("{prompt_text} ({})", field.help_text)
    }
}

fn parse_set_values(
    set_values: &[String],
) -> Result<BTreeMap<String, String>, AdapterCommandError> {
    let mut parsed = BTreeMap::new();
    for entry in set_values {
        let Some((key, value)) = entry.split_once('=') else {
            return Err(AdapterCommandError::InvalidSetValue { entry: entry.clone() });
        };
        parsed.insert(key.to_string(), value.to_string());
    }
    Ok(parsed)
}

fn resolve_field_value_text(
    field: &FrameworkAdapterConfigFieldDefinition,
    plan: &AdapterRegistrationPlan,
    overrides: &BTreeMap<String, String>,
) -> Option<(String, AdapterValueSource)> {
    if let Some(value) = overrides.get(&field.field_key) {
        return Some((value.clone(), AdapterValueSource::CliFlag));
    }

    plan.profile_defaults
        .get(&field.field_key)
        .map(|value| (value.clone(), AdapterValueSource::KnownProfileDefault))
}

fn protocol_value_from_text(
    field: &FrameworkAdapterConfigFieldDefinition,
    value_text: &str,
) -> Result<FrameworkAdapterConfigValue, AdapterCommandError> {
    let mut value = FrameworkAdapterConfigValue {
        field_key: field.field_key.clone(),
        value_kind: field.value_kind,
        string_value: None,
        path_value: None,
        bool_value: None,
        int_value: None,
    };

    match field.value_kind {
        FrameworkAdapterFieldValueKind::String | FrameworkAdapterFieldValueKind::Enum => {
            value.string_value = Some(value_text.to_string());
        }
        FrameworkAdapterFieldValueKind::Path => {
            value.path_value = Some(value_text.to_string());
        }
        FrameworkAdapterFieldValueKind::Boolean => {
            value.bool_value = Some(value_text.parse::<bool>().map_err(|_| {
                AdapterCommandError::InvalidFieldValue {
                    field_key: field.field_key.clone(),
                    expected: "boolean",
                }
            })?);
        }
        FrameworkAdapterFieldValueKind::Integer => {
            value.int_value = Some(value_text.parse::<i64>().map_err(|_| {
                AdapterCommandError::InvalidFieldValue {
                    field_key: field.field_key.clone(),
                    expected: "integer",
                }
            })?);
        }
    }

    Ok(value)
}

fn stored_value_record(
    field: &FrameworkAdapterConfigFieldDefinition,
    protocol_value: &FrameworkAdapterConfigValue,
    value_source: AdapterValueSource,
) -> AdapterConfigValueRecord {
    AdapterConfigValueRecord {
        field_key: field.field_key.clone(),
        value_kind: adapter_value_kind(field.value_kind),
        secret: field.secret,
        string_value: protocol_value.string_value.clone(),
        path_value: protocol_value.path_value.clone(),
        bool_value: protocol_value.bool_value,
        int_value: protocol_value.int_value,
        value_source,
        resolution_state: StoredAdapterConfigValueState::Present,
    }
}

fn adapter_value_kind(value_kind: FrameworkAdapterFieldValueKind) -> AdapterValueKind {
    match value_kind {
        FrameworkAdapterFieldValueKind::String => AdapterValueKind::String,
        FrameworkAdapterFieldValueKind::Path => AdapterValueKind::Path,
        FrameworkAdapterFieldValueKind::Boolean => AdapterValueKind::Boolean,
        FrameworkAdapterFieldValueKind::Integer => AdapterValueKind::Integer,
        FrameworkAdapterFieldValueKind::Enum => AdapterValueKind::Enum,
    }
}

pub(crate) fn config_schema_fingerprint(describe: &FrameworkAdapterDescribeResponse) -> String {
    let field_keys = describe
        .required_config_fields
        .iter()
        .map(|field| field.field_key.as_str())
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{}:{}", describe.protocol_line, describe.adapter_id, field_keys)
}

fn build_persisted_configuration(
    plan: &AdapterRegistrationPlan,
    prepared: &PreparedConfigValues,
    preflight: &FrameworkAdapterPreflightResponse,
    registration_source: AdapterRegistrationSource,
) -> PersistedAdapterConfiguration {
    let timestamp = current_timestamp_millis();
    let stored_values = if preflight.normalized_config_values.is_empty() {
        prepared.stored_values.clone()
    } else {
        preflight
            .normalized_config_values
            .iter()
            .map(|value| stored_value_from_normalized(value, &prepared.stored_values))
            .collect()
    };

    PersistedAdapterConfiguration {
        selection: AdapterSelectionRecord {
            selection_mode: plan.selection_mode,
            adapter_id: plan.adapter_id.clone(),
            display_name: plan.display_name.clone(),
            command: plan.command.clone(),
            args: plan.args.clone(),
            registration_source,
            discovery_state: plan.discovery_state,
            compatibility_line: plan.compatibility_line.clone(),
            updated_at: timestamp,
        },
        schema_fingerprint: prepared.schema_fingerprint.clone(),
        completeness_state: AdapterConfigCompletenessState::Complete,
        interactive_resolution: prepared.interactive_resolution,
        last_validated_at: Some(timestamp),
        value_count: stored_values.len(),
        values: stored_values,
    }
}

fn stored_value_from_normalized(
    normalized: &FrameworkAdapterConfigValue,
    stored_values: &[AdapterConfigValueRecord],
) -> AdapterConfigValueRecord {
    let source = stored_values
        .iter()
        .find(|value| value.field_key == normalized.field_key)
        .map(|value| value.value_source)
        .unwrap_or(AdapterValueSource::CliFlag);

    AdapterConfigValueRecord {
        field_key: normalized.field_key.clone(),
        value_kind: adapter_value_kind(normalized.value_kind),
        secret: stored_values
            .iter()
            .find(|value| value.field_key == normalized.field_key)
            .map(|value| value.secret)
            .unwrap_or(false),
        string_value: normalized.string_value.clone(),
        path_value: normalized.path_value.clone(),
        bool_value: normalized.bool_value,
        int_value: normalized.int_value,
        value_source: source,
        resolution_state: StoredAdapterConfigValueState::Present,
    }
}

fn save_adapter_selection(
    store: &FileConfigStore,
    persisted: PersistedAdapterConfiguration,
) -> Result<(), AdapterCommandError> {
    let mut config = store.load_local()?.unwrap_or_default();
    config.adapter = Some(persisted);
    store.save_local(&config)?;
    Ok(())
}

fn adapter_host(
    command: &str,
    args: &[String],
    workspace: &Path,
) -> Result<SubprocessFrameworkAdapterHost, FrameworkAdapterHostError> {
    Ok(SubprocessFrameworkAdapterHost::new(command)?
        .with_args(args.to_vec())
        .with_working_directory(workspace.to_path_buf()))
}

fn built_in_default_report(registry: &FrameworkAdapterProfileRegistry) -> AdapterCommandReport {
    let mut lines = vec![
        format!("status: {STATUS_BUILT_IN_DEFAULT}"),
        format!("execution_source: {EXECUTION_SOURCE_BUILT_IN}"),
    ];

    if let Some(profile) =
        registry.profiles().find(|profile| command_exists_on_path(&profile.default_command))
    {
        lines.push(format!("discovery_hint: {} available on PATH", profile.adapter_id));
        lines.push(format!(
            "activation_required: {ADD_RECOVERY_TEMPLATE} {}",
            profile.registration_alias
        ));
    }

    AdapterCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: render_lines(lines),
    }
}

fn ready_report(
    persisted: &PersistedAdapterConfiguration,
    describe: &FrameworkAdapterDescribeResponse,
) -> AdapterCommandReport {
    AdapterCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: render_lines([
            format!("status: {STATUS_READY}"),
            format!("adapter_id: {}", persisted.selection.adapter_id),
            format!("selection_mode: {}", selection_mode_label(persisted.selection.selection_mode)),
            format!("command: {}", persisted.selection.command),
            format!(
                "discovery_state: {}",
                discovery_state_label(persisted.selection.discovery_state)
            ),
            format!("compatibility_line: {}", persisted.selection.compatibility_line),
            format!(
                "supported_transports: {}",
                format_framework_adapter_transports(&describe.supported_transports)
            ),
            format!("declared_stage_overrides: {}", stage_list(describe)),
            format!("declared_hook_subscriptions: {}", hook_list(describe)),
            format!("config_state: {}", completeness_label(persisted.completeness_state)),
        ]),
    }
}

fn configured_show_report(
    persisted: &PersistedAdapterConfiguration,
    describe: &FrameworkAdapterDescribeResponse,
) -> AdapterCommandReport {
    AdapterCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: render_lines([
            "status: configured".to_string(),
            format!("adapter_id: {}", persisted.selection.adapter_id),
            format!("display_name: {}", persisted.selection.display_name),
            format!("command: {}", persisted.selection.command),
            format!(
                "discovery_state: {}",
                discovery_state_label(persisted.selection.discovery_state)
            ),
            format!("compatibility_line: {}", persisted.selection.compatibility_line),
            format!("config_state: {}", completeness_label(persisted.completeness_state)),
            format!(
                "supported_transports: {}",
                format_framework_adapter_transports(&describe.supported_transports)
            ),
            format!("declared_stage_overrides: {}", stage_list(describe)),
            format!("declared_hook_subscriptions: {}", hook_list(describe)),
        ]),
    }
}

fn existing_adapter_blocked_report(adapter_id: &str, workspace: &Path) -> AdapterCommandReport {
    blocked_report(
        adapter_id,
        REASON_ADAPTER_ALREADY_SELECTED,
        Some(format!("{REMOVE_RECOVERY_TEMPLATE} {}", workspace.display())),
        Vec::new(),
    )
}

fn blocked_preflight_report(
    adapter_id: &str,
    preflight: &FrameworkAdapterPreflightResponse,
) -> AdapterCommandReport {
    let reason = match preflight
        .reason
        .unwrap_or(FrameworkAdapterPreflightBlockReason::InvalidConfig)
    {
        FrameworkAdapterPreflightBlockReason::MissingRequiredConfig => {
            REASON_MISSING_REQUIRED_CONFIG
        }
        FrameworkAdapterPreflightBlockReason::IncompatibleProtocol => REASON_INCOMPATIBLE_PROTOCOL,
        FrameworkAdapterPreflightBlockReason::InvalidConfig => "invalid_config",
        FrameworkAdapterPreflightBlockReason::UnavailableResource => "unavailable_resource",
    };
    blocked_report(adapter_id, reason, preflight.recovery.clone(), preflight.missing_fields.clone())
}

fn blocked_report(
    adapter_id: &str,
    reason: &str,
    recovery: Option<String>,
    missing_fields: Vec<String>,
) -> AdapterCommandReport {
    let mut lines = vec![
        format!("status: {STATUS_BLOCKED}"),
        format!("adapter_id: {adapter_id}"),
        format!("reason: {reason}"),
    ];
    if !missing_fields.is_empty() {
        lines.push(format!("missing_fields: {}", missing_fields.join(", ")));
    }
    if let Some(recovery) = recovery {
        lines.push(format!("recovery: {recovery}"));
    }
    AdapterCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: render_lines(lines),
    }
}

fn unsupported_transport_report(
    adapter_id: &str,
    profile: &str,
    workspace: &Path,
    describe: &FrameworkAdapterDescribeResponse,
) -> AdapterCommandReport {
    let mut lines = vec![
        format!("status: {STATUS_BLOCKED}"),
        format!("adapter_id: {adapter_id}"),
        format!("reason: {REASON_UNSUPPORTED_TRANSPORT}"),
        format!("recovery: {ADD_RECOVERY_TEMPLATE} {profile} --workspace {}", workspace.display()),
    ];
    if !describe.supported_transports.is_empty() {
        lines.push(format!(
            "supported_transports: {}",
            format_framework_adapter_transports(&describe.supported_transports)
        ));
    }
    AdapterCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: render_lines(lines),
    }
}

fn cancelled_report(
    adapter_id: &str,
    workspace: &Path,
    request: &AddAdapterRequest<'_>,
    detail: &str,
) -> AdapterCommandReport {
    AdapterCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: render_lines([
            format!("status: {STATUS_CANCELLED}"),
            format!("adapter_id: {adapter_id}"),
            format!("reason: {REASON_SETUP_CANCELLED}"),
            format!("recovery: {}", resume_add_command(request, workspace)),
            format!(
                "inspect_or_edit: {CONFIG_SHOW_TEMPLATE} {} --scope workspace",
                workspace.display()
            ),
            format!("detail: {detail}"),
        ]),
    }
}

fn resume_add_command(request: &AddAdapterRequest<'_>, workspace: &Path) -> String {
    let mut parts = vec![
        "boundline".to_string(),
        "adapter".to_string(),
        "add".to_string(),
        request.profile.to_string(),
        "--workspace".to_string(),
        workspace.display().to_string(),
    ];

    if let Some(id) = request.id {
        parts.push("--id".to_string());
        parts.push(id.to_string());
    }
    if let Some(command) = request.command {
        parts.push("--command".to_string());
        parts.push(command.to_string());
    }
    for arg in request.arg {
        parts.push("--arg".to_string());
        parts.push(arg.clone());
    }
    for value in request.set {
        parts.push("--set".to_string());
        parts.push(value.clone());
    }

    parts.join(" ")
}

fn is_guided_cancel_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("cancel")
        || normalized.contains("interrupted")
        || normalized.contains("ctrl-c")
        || normalized.contains("eof")
}

fn unavailable_binary_report(
    plan: &AdapterRegistrationPlan,
    profile: &str,
    workspace: &Path,
    error: &FrameworkAdapterHostError,
) -> Option<AdapterCommandReport> {
    match error {
        FrameworkAdapterHostError::Spawn { source, .. }
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            let recovery = format!(
                "{ADD_RECOVERY_TEMPLATE} {profile} --workspace {} --command /path/to/{}",
                workspace.display(),
                plan.command
            );
            let mut lines = vec![
                format!("status: {STATUS_BLOCKED}"),
                format!("adapter_id: {}", plan.adapter_id),
                format!("command: {}", plan.command),
                format!("discovery_state: {}", discovery_state_label(plan.discovery_state)),
                format!("reason: {REASON_UNAVAILABLE_BINARY}"),
                format!("recovery: {recovery}"),
            ];
            if plan.discovery_state == AdapterDiscoveryState::Unresolved {
                lines.push(
                    "detail: default command was not found on PATH; install the adapter or pass --command"
                        .to_string(),
                );
            }
            Some(AdapterCommandReport {
                exit_status: CommandExitStatus::NonSuccess,
                terminal_output: render_lines(lines),
            })
        }
        _ => None,
    }
}

fn removed_report() -> AdapterCommandReport {
    AdapterCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: render_lines([
            format!("status: {STATUS_REMOVED}"),
            format!("execution_source: {EXECUTION_SOURCE_BUILT_IN}"),
        ]),
    }
}

fn command_error_report(action: &str, error: AdapterCommandError) -> AdapterCommandReport {
    AdapterCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: format!("adapter {action} error: {error}"),
    }
}

fn render_lines(lines: impl IntoIterator<Item = String>) -> String {
    lines.into_iter().collect::<Vec<_>>().join("\n")
}

fn stage_list(describe: &FrameworkAdapterDescribeResponse) -> String {
    describe
        .declared_stage_overrides
        .iter()
        .map(|stage| stage.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn hook_list(describe: &FrameworkAdapterDescribeResponse) -> String {
    describe
        .declared_hook_subscriptions
        .iter()
        .map(|hook| hook.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn selection_mode_label(selection_mode: AdapterSelectionMode) -> &'static str {
    match selection_mode {
        AdapterSelectionMode::None => "none",
        AdapterSelectionMode::KnownProfile => "known_profile",
        AdapterSelectionMode::Custom => "custom",
    }
}

pub(crate) fn discovery_state_label(discovery_state: AdapterDiscoveryState) -> &'static str {
    match discovery_state {
        AdapterDiscoveryState::ExplicitCommand => "explicit_command",
        AdapterDiscoveryState::DiscoveredOnPath => "discovered_on_path",
        AdapterDiscoveryState::Unresolved => "unresolved",
    }
}

fn completeness_label(completeness: AdapterConfigCompletenessState) -> &'static str {
    match completeness {
        AdapterConfigCompletenessState::Complete => CONFIG_STATE_COMPLETE,
        AdapterConfigCompletenessState::MissingRequired => CONFIG_STATE_MISSING_REQUIRED,
        AdapterConfigCompletenessState::Invalid => CONFIG_STATE_INVALID,
    }
}

pub(crate) fn command_exists_on_path(command: &str) -> bool {
    let command_path = Path::new(command);
    if contains_path_component(command_path) {
        return command_path.is_file();
    }

    env::var_os("PATH")
        .map(|path_var| {
            env::split_paths(&path_var).any(|directory| directory.join(command).is_file())
        })
        .unwrap_or(false)
}

fn contains_path_component(path: &Path) -> bool {
    let mut components = path.components();
    matches!(components.next(), Some(Component::RootDir | Component::CurDir | Component::ParentDir))
        || path.components().count() > 1
        || path.as_os_str().to_string_lossy().contains(std::path::MAIN_SEPARATOR)
        || path.as_os_str().to_string_lossy().contains('/')
        || path.as_os_str().to_string_lossy().contains('\\')
}
