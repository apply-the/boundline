use std::env;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::adapters::env_layer::{
    ANTHROPIC_API_KEY_ENV, ANTHROPIC_BASE_URL_ENV, COPILOT_API_KEY_ENV, COPILOT_API_URL_ENV,
    COPILOT_GITHUB_TOKEN_ENV, DEEPSEEK_API_KEY_ENV, DEEPSEEK_BASE_URL_ENV, GEMINI_API_KEY_ENV,
    GH_TOKEN_ENV, GITHUB_COPILOT_API_TOKEN_ENV, GITHUB_MODELS_BASE_URL_ENV, GITHUB_MODELS_ORG_ENV,
    GITHUB_MODELS_TOKEN_ENV, GITHUB_TOKEN_ENV, GROK_API_KEY_ENV, GROK_BASE_URL_ENV,
    GROQ_API_KEY_ENV, GROQ_BASE_URL_ENV, OLLAMA_BASE_URL_ENV, OPENAI_API_KEY_ENV,
    OPENAI_BASE_URL_ENV,
};
use crate::domain::configuration::{ModelRoute, RuntimeKind};

#[path = "provider_runtime/anthropic.rs"]
mod anthropic;
#[path = "provider_runtime/copilot.rs"]
mod copilot;
#[path = "provider_runtime/deepseek.rs"]
mod deepseek;
#[path = "provider_runtime/gemini.rs"]
mod gemini;
#[path = "provider_runtime/github_models.rs"]
mod github_models;
#[path = "provider_runtime/grok.rs"]
mod grok;
#[path = "provider_runtime/groq.rs"]
mod groq;
#[path = "provider_runtime/ollama.rs"]
mod ollama;
#[path = "provider_runtime/openai.rs"]
mod openai;
#[path = "provider_runtime/openai_compatible.rs"]
mod openai_compatible;

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_GROK_BASE_URL: &str = "https://api.x.ai/v1";
const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434/v1";
const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
const DEFAULT_GITHUB_MODELS_BASE_URL: &str = "https://models.github.ai/inference";
const DEFAULT_COPILOT_BASE_URL: &str = "https://api.individual.githubcopilot.com";
const COPILOT_TOKEN_EXCHANGE_URL: &str = "https://api.github.com/copilot_internal/v2/token";
const OPENAI_CHAT_COMPLETIONS_PATH: &str = "/chat/completions";
const ANTHROPIC_MESSAGES_PATH: &str = "/messages";
const ANTHROPIC_ROOT_MESSAGES_PATH: &str = "/v1/messages";
const COPILOT_INTEGRATION_ID_HEADER: &str = "Copilot-Integration-Id";
const COPILOT_INTEGRATION_ID_VALUE: &str = "vscode-chat";
const COPILOT_EDITOR_VERSION_HEADER: &str = "Editor-Version";
const COPILOT_EDITOR_VERSION_VALUE: &str = "vscode/1.107.0";
const COPILOT_EDITOR_PLUGIN_VERSION_HEADER: &str = "Editor-Plugin-Version";
const COPILOT_EDITOR_PLUGIN_VERSION_VALUE: &str = "copilot-chat/0.35.0";
const COPILOT_USER_AGENT: &str = "GitHubCopilotChat/0.35.0";
const COPILOT_GITHUB_API_VERSION_HEADER: &str = "X-Github-Api-Version";
const COPILOT_GITHUB_API_VERSION_VALUE: &str = "2025-04-01";
const COPILOT_ACCEPT_ENCODING_HEADER: &str = "Accept-Encoding";
const COPILOT_ACCEPT_ENCODING_VALUE: &str = "identity";
const COPILOT_OPENAI_ORGANIZATION_HEADER: &str = "Openai-Organization";
const COPILOT_OPENAI_ORGANIZATION_VALUE: &str = "github-copilot";
const GITHUB_API_ACCEPT_HEADER_VALUE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION_HEADER: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";
const JSON_ACCEPT_HEADER_VALUE: &str = "application/json";
const ANTHROPIC_VERSION_HEADER: &str = "anthropic-version";
const ANTHROPIC_VERSION_VALUE: &str = "2023-06-01";
const GEMINI_RESPONSE_MIME_TYPE: &str = "application/json";
const PROVIDER_TIMEOUT_SECS: u64 = 90;
const DEFAULT_PROVIDER_CHAT_MAX_TOKENS: u32 = 2048;
const ANALYSIS_RESPONSE_SCHEMA: &str = r#"{"headline":"...","summary":"...","risks":["..."]}"#;
const CHANGE_RESPONSE_SCHEMA: &str = r#"{"headline":"...","summary":"...","changes":[{"path":"relative/path","find":"exact existing substring","replace":"replacement text"}]}"#;
const REVIEW_RESPONSE_SCHEMA: &str = r#"{"disposition":"approve|concern|block","summary":"...","details":"optional","required_action":"optional","evidence_refs":["optional"]}"#;
const REVISION_RESPONSE_SCHEMA: &str =
    r#"{"headline":"...","summary":"...","revised_artifact":"...","applied_feedback":["..."]}"#;
const CODE_FENCE_PREFIX: &str = "```json";
const CODE_FENCE_SUFFIX: &str = "```";
const GEMINI_USER_ROLE: &str = "user";
const GEMINI_MODEL_ROLE: &str = "model";
const ANTHROPIC_TEXT_BLOCK_KIND: &str = "text";
const ANTHROPIC_USER_ROLE: &str = "user";
const ANTHROPIC_ASSISTANT_ROLE: &str = "assistant";
const MODEL_NAMESPACE_SEPARATOR: char = '/';
const COPILOT_TOKEN_ENV_HINT: &str =
    "GITHUB_COPILOT_API_TOKEN, COPILOT_GITHUB_TOKEN, GH_TOKEN, GITHUB_TOKEN, or COPILOT_API_KEY";
const GITHUB_MODELS_TOKEN_ENV_HINT: &str =
    "GITHUB_MODELS_TOKEN, GITHUB_TOKEN, COPILOT_GITHUB_TOKEN, or GH_TOKEN";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderWorkspaceFile {
    pub path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAnalysisRequest {
    pub goal: String,
    pub phase: String,
    pub files: Vec<ProviderWorkspaceFile>,
    #[doc(hidden)]
    pub guidance_context: Vec<String>,
    #[doc(hidden)]
    pub persona: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderChangeRequest {
    pub goal: String,
    pub phase: String,
    pub allowed_paths: Vec<String>,
    pub files: Vec<ProviderWorkspaceFile>,
    #[doc(hidden)]
    pub guidance_context: Vec<String>,
    #[doc(hidden)]
    pub persona: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderReviewRequest {
    pub goal: String,
    pub phase: String,
    pub reviewer_id: String,
    pub reviewer_role: String,
    pub attempt_id: String,
    pub files: Vec<ProviderWorkspaceFile>,
    pub prior_context: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRevisionRequest {
    pub goal: String,
    pub phase: String,
    pub reviser_id: String,
    pub target_refs: Vec<String>,
    pub current_artifact: String,
    pub accepted_feedback: Vec<String>,
    pub prior_context: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAnalysisResponse {
    pub headline: String,
    pub summary: String,
    #[serde(default)]
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderChangeResponse {
    pub headline: String,
    pub summary: String,
    #[serde(default)]
    pub changes: Vec<ProviderWorkspaceChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderWorkspaceChange {
    pub path: String,
    pub find: String,
    pub replace: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderReviewDisposition {
    Approve,
    Concern,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderReviewResponse {
    pub disposition: ProviderReviewDisposition,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_action: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRevisionResponse {
    pub headline: String,
    pub summary: String,
    pub revised_artifact: String,
    #[serde(default)]
    pub applied_feedback: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderChatRole {
    System,
    User,
    Assistant,
}

impl ProviderChatRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderChatMessage {
    pub role: ProviderChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderNamespace {
    OpenAi,
    DeepSeek,
    Grok,
    Groq,
    Ollama,
    Anthropic,
    Gemini,
    GitHubModels,
    Copilot,
}

impl ProviderNamespace {
    const fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::DeepSeek => "deepseek",
            Self::Grok => "grok",
            Self::Groq => "groq",
            Self::Ollama => "ollama",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::GitHubModels => "github-models",
            Self::Copilot => "copilot",
        }
    }

    const fn is_openai_compatible(self) -> bool {
        matches!(
            self,
            Self::OpenAi
                | Self::DeepSeek
                | Self::Grok
                | Self::Groq
                | Self::Ollama
                | Self::GitHubModels
                | Self::Copilot
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedProviderRoute {
    namespace: ProviderNamespace,
    model_id: String,
    base_url: String,
    api_key: Option<String>,
}

#[derive(Debug, Error)]
pub enum ProviderRuntimeError {
    #[error("provider runtime is not configured for {namespace} (missing {env_key})")]
    MissingApiKey { namespace: &'static str, env_key: &'static str },
    #[error("provider credential exchange failed for {namespace}: {message}")]
    CredentialExchange { namespace: &'static str, message: String },
    #[error("provider model is empty for runtime {runtime}")]
    MissingModel { runtime: RuntimeKind },
    #[error("provider model namespace `{namespace}` is not supported")]
    UnsupportedNamespace { namespace: String },
    #[error("failed to render provider prompt: {0}")]
    PromptRender(String),
    #[error("provider request failed: {0}")]
    Network(String),
    #[error("provider API error ({status}): {body}")]
    Api { status: u16, body: String },
    #[error("provider response was not valid: {0}")]
    BadResponse(String),
    #[error("provider change set is invalid: {0}")]
    InvalidChangeSet(String),
}

impl ProviderRuntimeError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::BadResponse(_) => true,
            Self::Api { status, .. } => {
                matches!(status, 408 | 429 | 500 | 502 | 503 | 504 | 529)
            }
            Self::MissingApiKey { .. }
            | Self::CredentialExchange { .. }
            | Self::MissingModel { .. }
            | Self::UnsupportedNamespace { .. }
            | Self::PromptRender(_)
            | Self::InvalidChangeSet(_) => false,
        }
    }
}

pub(crate) fn execute_with_retry(
    request_builder: reqwest::blocking::RequestBuilder,
    provider_name: &str,
    model_name: &str,
) -> Result<reqwest::blocking::Response, ProviderRuntimeError> {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let max_attempts = if std::env::var("BOUNDLINE_TEST_DISABLE_RETRIES").is_ok() { 1 } else { 10 };
    let mut attempt = 1;
    let initial_delay_ms = 300;
    let max_delay_ms = 5000;
    let mut current_delay_ms = initial_delay_ms;

    loop {
        let request = request_builder.try_clone().ok_or_else(|| {
            ProviderRuntimeError::Network("request cannot be cloned for retry".to_string())
        })?;

        match request.send() {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    return Ok(response);
                }

                let status_u16 = status.as_u16();
                let is_retryable_status =
                    matches!(status_u16, 408 | 429 | 500 | 502 | 503 | 504 | 529);

                if !is_retryable_status || attempt >= max_attempts {
                    return Ok(response);
                }

                // Poor man's jitter to avoid adding the rand crate
                let now =
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros()
                        as u64;
                let jitter = now % (current_delay_ms as u64 / 2).max(1);
                let delay = std::cmp::min(current_delay_ms as u64 + jitter, max_delay_ms as u64);

                tracing::warn!(
                    provider = provider_name,
                    model = model_name,
                    attempt = attempt,
                    max_attempts = max_attempts,
                    delay_ms = delay,
                    reason = status_u16,
                    "retrying provider request due to transient HTTP error",
                );

                std::thread::sleep(Duration::from_millis(delay));
                current_delay_ms = std::cmp::min(current_delay_ms * 2, max_delay_ms);
                attempt += 1;
            }
            Err(e) => {
                if attempt >= max_attempts {
                    return Err(ProviderRuntimeError::Network(e.to_string()));
                }

                let now =
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros()
                        as u64;
                let jitter = now % (current_delay_ms as u64 / 2).max(1);
                let delay = std::cmp::min(current_delay_ms as u64 + jitter, max_delay_ms as u64);

                tracing::warn!(
                    provider = provider_name,
                    model = model_name,
                    attempt = attempt,
                    max_attempts = max_attempts,
                    delay_ms = delay,
                    reason = %e,
                    "retrying provider request due to network error",
                );

                std::thread::sleep(Duration::from_millis(delay));
                current_delay_ms = std::cmp::min(current_delay_ms * 2, max_delay_ms);
                attempt += 1;
            }
        }
    }
}

pub fn route_is_available(route: &ModelRoute) -> bool {
    resolve_provider_route(route).is_ok()
}

pub fn route_uses_explicit_provider_namespace(route: &ModelRoute) -> bool {
    let trimmed = route.model.trim();
    match trimmed.split_once(MODEL_NAMESPACE_SEPARATOR) {
        Some((namespace, model_id)) => {
            !model_id.trim().is_empty() && namespace_from_label(namespace.trim()).is_ok()
        }
        None => false,
    }
}

pub fn analyze_workspace(
    route: &ModelRoute,
    request: &ProviderAnalysisRequest,
) -> Result<ProviderAnalysisResponse, ProviderRuntimeError> {
    let prompt = build_analysis_prompt(request)?;
    let system_prompt = compose_enriched_system_prompt(
        analysis_system_prompt(),
        &request.guidance_context,
        request.persona.as_deref(),
    );
    dispatch_structured_prompt(route, &system_prompt, &prompt)
}

pub fn propose_workspace_changes(
    route: &ModelRoute,
    request: &ProviderChangeRequest,
) -> Result<ProviderChangeResponse, ProviderRuntimeError> {
    let prompt = build_change_prompt(request)?;
    let system_prompt = compose_enriched_system_prompt(
        change_system_prompt(),
        &request.guidance_context,
        request.persona.as_deref(),
    );
    let response = dispatch_structured_prompt(route, &system_prompt, &prompt)?;
    validate_change_response(request, &response)?;
    Ok(response)
}

pub fn review_workspace(
    route: &ModelRoute,
    request: &ProviderReviewRequest,
) -> Result<ProviderReviewResponse, ProviderRuntimeError> {
    let prompt = build_review_prompt(request)?;
    dispatch_structured_prompt(route, review_system_prompt(), &prompt)
}

pub fn revise_artifact(
    route: &ModelRoute,
    request: &ProviderRevisionRequest,
) -> Result<ProviderRevisionResponse, ProviderRuntimeError> {
    let prompt = build_revision_prompt(request)?;
    dispatch_structured_prompt(route, revision_system_prompt(), &prompt)
}

pub fn chat_completion(
    route: &ModelRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    let resolved = resolve_provider_route(route)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(PROVIDER_TIMEOUT_SECS))
        .build()
        .unwrap_or_else(|_| Client::new());

    if resolved.namespace.is_openai_compatible() {
        execute_openai_compatible_chat(&client, &resolved, messages, max_tokens)
    } else if resolved.namespace == ProviderNamespace::Anthropic {
        execute_anthropic_chat(&client, &resolved, messages, max_tokens)
    } else {
        execute_gemini_chat(&client, &resolved, messages, max_tokens)
    }
}

fn analysis_system_prompt() -> &'static str {
    "You are Boundline's planning runtime. Reply with JSON only. Base the answer only on the supplied goal and files. Do not invent files or hidden context."
}

fn change_system_prompt() -> &'static str {
    "You are Boundline's implementation runtime. Reply with JSON only. Produce safe bounded find/replace edits against the supplied files. Use only allowed paths, keep every find string exact, and never invent missing code context."
}

/// Returns a persona-specific framing prefix that focuses the agent on the
/// quality attributes most relevant to the intended role.
pub fn persona_system_prompt_prefix(persona: &str) -> &'static str {
    match persona {
        "delivery-engineer" => {
            "You are acting as a delivery engineer. Focus on working, testable code that meets acceptance criteria. Prefer simplicity over cleverness. Every change must compile and pass existing tests."
        }
        "system-architect" => {
            "You are acting as a system architect. Focus on boundary contracts, integration surfaces, and structural correctness. Ensure domain separation, dependency direction, and extensibility."
        }
        "verification-lead" => {
            "You are acting as a verification lead. Focus on edge cases, error paths, and comprehensive test coverage. Validate every assumption with explicit assertions."
        }
        "product-strategist" => {
            "You are acting as a product strategist. Focus on user-facing behavior and acceptance criteria alignment. Ensure deliverables match the stated outcome."
        }
        "operations-governor" => {
            "You are acting as an operations governor. Focus on operational safety, observability hooks, graceful degradation, and rollback capability."
        }
        "domain-steward" => {
            "You are acting as a domain steward. Focus on ubiquitous language, aggregate boundaries, and domain model integrity. Ensure naming and structure match the domain model."
        }
        _ => "",
    }
}

/// Composes an enriched system prompt by appending resolved guidance content
/// and an optional persona prefix to the base implementation prompt.
///
/// When both `guidance_content` and `persona` are empty/absent, returns the
/// base prompt unchanged for backward compatibility.
pub fn compose_enriched_system_prompt(
    base: &str,
    guidance_content: &[String],
    persona: Option<&str>,
) -> String {
    if guidance_content.is_empty() && persona.is_none_or(|p| p.is_empty()) {
        return base.to_string();
    }

    let mut composed = String::with_capacity(base.len() + 2048);

    if let Some(persona_label) = persona {
        let prefix = persona_system_prompt_prefix(persona_label);
        if !prefix.is_empty() {
            composed.push_str(prefix);
            composed.push_str("\n\n");
        }
    }

    composed.push_str(base);

    if !guidance_content.is_empty() {
        composed.push_str("\n\n## Implementation Guidance\n\nApply the following project conventions and language guidance when producing changes:\n");
        for (index, content) in guidance_content.iter().enumerate() {
            composed.push_str("\n---\n");
            composed.push_str(&format!("### Guidance {}", index + 1));
            composed.push('\n');
            composed.push_str(content);
            composed.push('\n');
        }
    }

    composed
}

fn review_system_prompt() -> &'static str {
    "You are Boundline's review runtime. Reply with JSON only. Judge only the supplied goal, reviewer role, prior context, and workspace files. Choose exactly one disposition: approve, concern, or block. Use concern for non-blocking issues that still require follow-up; use block only for clear stop conditions."
}

fn revision_system_prompt() -> &'static str {
    "You are Boundline's revision runtime. Reply with JSON only. Rewrite only the supplied artifact text using the accepted feedback and prior context. Preserve bounded facts, do not invent external evidence, and return the full revised artifact body."
}

fn build_analysis_prompt(
    request: &ProviderAnalysisRequest,
) -> Result<String, ProviderRuntimeError> {
    let files = serde_json::to_string_pretty(&request.files)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    Ok(format!(
        "Goal:\n{}\n\nPhase:\n{}\n\nWorkspace Files:\n{}\n\nReturn exactly this JSON shape:\n{}",
        request.goal.trim(),
        request.phase.trim(),
        files,
        ANALYSIS_RESPONSE_SCHEMA,
    ))
}

fn build_change_prompt(request: &ProviderChangeRequest) -> Result<String, ProviderRuntimeError> {
    let files = serde_json::to_string_pretty(&request.files)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    let allowed_paths = serde_json::to_string_pretty(&request.allowed_paths)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    Ok(format!(
        "Goal:\n{}\n\nPhase:\n{}\n\nAllowed Paths:\n{}\n\nWorkspace Files:\n{}\n\nReturn exactly this JSON shape:\n{}\n\nUse zero or more changes. If no credible safe edit exists, return an empty changes array with an explanatory summary.",
        request.goal.trim(),
        request.phase.trim(),
        allowed_paths,
        files,
        CHANGE_RESPONSE_SCHEMA,
    ))
}

fn build_review_prompt(request: &ProviderReviewRequest) -> Result<String, ProviderRuntimeError> {
    let files = serde_json::to_string_pretty(&request.files)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    let prior_context = serde_json::to_string_pretty(&request.prior_context)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    Ok(format!(
        "Goal:\n{}\n\nPhase:\n{}\n\nReviewer:\n{} ({})\n\nAttempt:\n{}\n\nPrior Context:\n{}\n\nWorkspace Files:\n{}\n\nReturn exactly this JSON shape:\n{}",
        request.goal.trim(),
        request.phase.trim(),
        request.reviewer_role.trim(),
        request.reviewer_id.trim(),
        request.attempt_id.trim(),
        prior_context,
        files,
        REVIEW_RESPONSE_SCHEMA,
    ))
}

fn build_revision_prompt(
    request: &ProviderRevisionRequest,
) -> Result<String, ProviderRuntimeError> {
    let target_refs = serde_json::to_string_pretty(&request.target_refs)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    let accepted_feedback = serde_json::to_string_pretty(&request.accepted_feedback)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    let prior_context = serde_json::to_string_pretty(&request.prior_context)
        .map_err(|error| ProviderRuntimeError::PromptRender(error.to_string()))?;
    Ok(format!(
        "Goal:\n{}\n\nPhase:\n{}\n\nReviser:\n{}\n\nTarget Refs:\n{}\n\nAccepted Feedback:\n{}\n\nPrior Context:\n{}\n\nCurrent Artifact:\n{}\n\nReturn exactly this JSON shape:\n{}",
        request.goal.trim(),
        request.phase.trim(),
        request.reviser_id.trim(),
        target_refs,
        accepted_feedback,
        prior_context,
        request.current_artifact.trim(),
        REVISION_RESPONSE_SCHEMA,
    ))
}

fn validate_change_response(
    request: &ProviderChangeRequest,
    response: &ProviderChangeResponse,
) -> Result<(), ProviderRuntimeError> {
    for change in &response.changes {
        if change.path.trim().is_empty() {
            return Err(ProviderRuntimeError::InvalidChangeSet(
                "change path must not be empty".to_string(),
            ));
        }
        if !request.allowed_paths.iter().any(|path| path == &change.path) {
            return Err(ProviderRuntimeError::InvalidChangeSet(format!(
                "change path '{}' is outside the allowed workspace target set",
                change.path
            )));
        }
        if change.find.is_empty() {
            return Err(ProviderRuntimeError::InvalidChangeSet(format!(
                "change path '{}' is missing an exact find string",
                change.path
            )));
        }
        if change.find == change.replace {
            return Err(ProviderRuntimeError::InvalidChangeSet(format!(
                "change path '{}' does not modify the file contents",
                change.path
            )));
        }
    }

    Ok(())
}

fn dispatch_structured_prompt<T>(
    route: &ModelRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<T, ProviderRuntimeError>
where
    T: DeserializeOwned,
{
    let resolved = resolve_provider_route(route)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(PROVIDER_TIMEOUT_SECS))
        .build()
        .unwrap_or_else(|_| Client::new());
    let response_text = if resolved.namespace.is_openai_compatible() {
        execute_openai_compatible_prompt(&client, &resolved, system_prompt, user_prompt)?
    } else if resolved.namespace == ProviderNamespace::Anthropic {
        execute_anthropic_prompt(&client, &resolved, system_prompt, user_prompt)?
    } else {
        execute_gemini_prompt(&client, &resolved, system_prompt, user_prompt)?
    };

    parse_structured_response(&response_text)
}

fn execute_openai_compatible_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ProviderRuntimeError> {
    match route.namespace {
        ProviderNamespace::OpenAi => {
            openai::execute_prompt(client, route, system_prompt, user_prompt)
        }
        ProviderNamespace::DeepSeek => {
            deepseek::execute_prompt(client, route, system_prompt, user_prompt)
        }
        ProviderNamespace::Grok => grok::execute_prompt(client, route, system_prompt, user_prompt),
        ProviderNamespace::Groq => groq::execute_prompt(client, route, system_prompt, user_prompt),
        ProviderNamespace::Ollama => {
            ollama::execute_prompt(client, route, system_prompt, user_prompt)
        }
        ProviderNamespace::GitHubModels => {
            github_models::execute_prompt(client, route, system_prompt, user_prompt)
        }
        ProviderNamespace::Copilot => {
            copilot::execute_prompt(client, route, system_prompt, user_prompt)
        }
        _ => Err(ProviderRuntimeError::UnsupportedNamespace {
            namespace: route.namespace.as_str().to_string(),
        }),
    }
}

fn execute_openai_compatible_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    match route.namespace {
        ProviderNamespace::OpenAi => openai::execute_chat(client, route, messages, max_tokens),
        ProviderNamespace::DeepSeek => deepseek::execute_chat(client, route, messages, max_tokens),
        ProviderNamespace::Grok => grok::execute_chat(client, route, messages, max_tokens),
        ProviderNamespace::Groq => groq::execute_chat(client, route, messages, max_tokens),
        ProviderNamespace::Ollama => ollama::execute_chat(client, route, messages, max_tokens),
        ProviderNamespace::GitHubModels => {
            github_models::execute_chat(client, route, messages, max_tokens)
        }
        ProviderNamespace::Copilot => copilot::execute_chat(client, route, messages, max_tokens),
        _ => Err(ProviderRuntimeError::UnsupportedNamespace {
            namespace: route.namespace.as_str().to_string(),
        }),
    }
}

#[cfg(test)]
fn execute_openai_compatible_chat_with_exchange(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
    copilot_exchange_url: &str,
) -> Result<String, ProviderRuntimeError> {
    match route.namespace {
        ProviderNamespace::Copilot => copilot::execute_chat_with_exchange(
            client,
            route,
            messages,
            max_tokens,
            copilot_exchange_url,
        ),
        ProviderNamespace::OpenAi
        | ProviderNamespace::DeepSeek
        | ProviderNamespace::Grok
        | ProviderNamespace::Groq
        | ProviderNamespace::Ollama
        | ProviderNamespace::GitHubModels => {
            execute_openai_compatible_chat(client, route, messages, max_tokens)
        }
        _ => Err(ProviderRuntimeError::UnsupportedNamespace {
            namespace: route.namespace.as_str().to_string(),
        }),
    }
}

fn execute_anthropic_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ProviderRuntimeError> {
    anthropic::execute_prompt(client, route, system_prompt, user_prompt)
}

fn execute_anthropic_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    anthropic::execute_chat(client, route, messages, max_tokens)
}

fn execute_gemini_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ProviderRuntimeError> {
    gemini::execute_prompt(client, route, system_prompt, user_prompt)
}

fn execute_gemini_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    gemini::execute_chat(client, route, messages, max_tokens)
}

fn system_prompt_from_messages(messages: &[ProviderChatMessage]) -> Option<String> {
    let combined = messages
        .iter()
        .filter(|message| message.role == ProviderChatRole::System)
        .map(|message| message.content.trim())
        .filter(|content| !content.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    (!combined.is_empty()).then_some(combined)
}

fn parse_structured_response<T>(content: &str) -> Result<T, ProviderRuntimeError>
where
    T: DeserializeOwned,
{
    if let Ok(parsed) = serde_json::from_str::<T>(content.trim()) {
        return Ok(parsed);
    }

    if let Some(stripped) = strip_code_fence(content)
        && let Ok(parsed) = serde_json::from_str::<T>(stripped)
    {
        return Ok(parsed);
    }

    if let Some(candidate) = first_json_object(content)
        && let Ok(parsed) = serde_json::from_str::<T>(&candidate)
    {
        return Ok(parsed);
    }

    Err(ProviderRuntimeError::BadResponse(
        "provider did not return a valid JSON object".to_string(),
    ))
}

fn strip_code_fence(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    if !trimmed.starts_with(CODE_FENCE_PREFIX) || !trimmed.ends_with(CODE_FENCE_SUFFIX) {
        return None;
    }

    let without_prefix = trimmed.strip_prefix(CODE_FENCE_PREFIX)?.trim();
    without_prefix.strip_suffix(CODE_FENCE_SUFFIX).map(str::trim)
}

fn first_json_object(content: &str) -> Option<String> {
    let mut depth = 0_u32;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in content.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => {
                if start.is_none() {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    let object_start = start?;
                    return content.get(object_start..=index).map(str::to_string);
                }
            }
            _ => {}
        }
    }

    None
}

fn resolve_provider_route(
    route: &ModelRoute,
) -> Result<ResolvedProviderRoute, ProviderRuntimeError> {
    let (explicit_namespace, model_id) = parse_model_selection(route)?;
    let namespace =
        explicit_namespace.unwrap_or_else(|| default_namespace_for_runtime(route.runtime));
    let (base_url, api_key) = resolve_credentials(namespace)?;

    Ok(ResolvedProviderRoute { namespace, model_id, base_url, api_key })
}

fn parse_model_selection(
    route: &ModelRoute,
) -> Result<(Option<ProviderNamespace>, String), ProviderRuntimeError> {
    let trimmed = route.model.trim();
    if trimmed.is_empty() {
        return Err(ProviderRuntimeError::MissingModel { runtime: route.runtime });
    }

    if let Some((namespace, model_id)) = trimmed.split_once(MODEL_NAMESPACE_SEPARATOR) {
        let provider_namespace = namespace_from_label(namespace.trim())?;
        let normalized_model_id = model_id.trim().to_string();
        if normalized_model_id.is_empty() {
            return Err(ProviderRuntimeError::MissingModel { runtime: route.runtime });
        }
        return Ok((Some(provider_namespace), normalized_model_id));
    }

    Ok((None, trimmed.to_string()))
}

fn namespace_from_label(label: &str) -> Result<ProviderNamespace, ProviderRuntimeError> {
    let normalized = label.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "openai" => Ok(ProviderNamespace::OpenAi),
        // Keep `codex/...` as a compatibility alias for older route strings,
        // but resolve it through the OpenAI provider surface.
        "codex" => Ok(ProviderNamespace::OpenAi),
        "deepseek" => Ok(ProviderNamespace::DeepSeek),
        "grok" => Ok(ProviderNamespace::Grok),
        "groq" => Ok(ProviderNamespace::Groq),
        "ollama" => Ok(ProviderNamespace::Ollama),
        "anthropic" | "claude" => Ok(ProviderNamespace::Anthropic),
        "gemini" => Ok(ProviderNamespace::Gemini),
        "github-models" | "github_models" => Ok(ProviderNamespace::GitHubModels),
        "copilot" => Ok(ProviderNamespace::Copilot),
        _ => Err(ProviderRuntimeError::UnsupportedNamespace { namespace: label.to_string() }),
    }
}

fn default_namespace_for_runtime(runtime: RuntimeKind) -> ProviderNamespace {
    match runtime {
        RuntimeKind::Claude => ProviderNamespace::Anthropic,
        RuntimeKind::Codex => ProviderNamespace::OpenAi,
        RuntimeKind::Copilot => ProviderNamespace::Copilot,
        RuntimeKind::Gemini => ProviderNamespace::Gemini,
    }
}

fn resolve_credentials(
    namespace: ProviderNamespace,
) -> Result<(String, Option<String>), ProviderRuntimeError> {
    match namespace {
        ProviderNamespace::OpenAi => openai::resolve_credentials(),
        ProviderNamespace::DeepSeek => deepseek::resolve_credentials(),
        ProviderNamespace::Grok => grok::resolve_credentials(),
        ProviderNamespace::Groq => groq::resolve_credentials(),
        ProviderNamespace::Ollama => ollama::resolve_credentials(),
        ProviderNamespace::Anthropic => anthropic::resolve_credentials(),
        ProviderNamespace::Gemini => gemini::resolve_credentials(),
        ProviderNamespace::GitHubModels => github_models::resolve_credentials(),
        ProviderNamespace::Copilot => copilot::resolve_credentials(),
    }
}

#[cfg(test)]
fn anthropic_messages_endpoint(base_url: &str) -> String {
    anthropic::messages_endpoint(base_url)
}

fn required_env(
    namespace: ProviderNamespace,
    env_key: &'static str,
) -> Result<String, ProviderRuntimeError> {
    env_string(env_key)
        .ok_or(ProviderRuntimeError::MissingApiKey { namespace: namespace.as_str(), env_key })
}

fn env_string(key: &'static str) -> Option<String> {
    env::var(key).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::ffi::OsString;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::sync::{Mutex, MutexGuard};
    use std::thread;
    use std::time::Duration;

    use reqwest::blocking::Client;
    use serde_json::json;

    use crate::domain::configuration::{ModelRoute, RuntimeKind};

    use super::{
        ANTHROPIC_API_KEY_ENV, ANTHROPIC_BASE_URL_ENV, COPILOT_ACCEPT_ENCODING_HEADER,
        COPILOT_ACCEPT_ENCODING_VALUE, COPILOT_API_KEY_ENV, COPILOT_API_URL_ENV,
        COPILOT_EDITOR_PLUGIN_VERSION_HEADER, COPILOT_EDITOR_PLUGIN_VERSION_VALUE,
        COPILOT_EDITOR_VERSION_HEADER, COPILOT_EDITOR_VERSION_VALUE,
        COPILOT_GITHUB_API_VERSION_HEADER, COPILOT_GITHUB_API_VERSION_VALUE,
        COPILOT_GITHUB_TOKEN_ENV, COPILOT_INTEGRATION_ID_VALUE, COPILOT_TOKEN_ENV_HINT,
        COPILOT_USER_AGENT, DEEPSEEK_API_KEY_ENV, DEEPSEEK_BASE_URL_ENV,
        DEFAULT_ANTHROPIC_BASE_URL, DEFAULT_COPILOT_BASE_URL, DEFAULT_DEEPSEEK_BASE_URL,
        GH_TOKEN_ENV, GITHUB_API_ACCEPT_HEADER_VALUE, GITHUB_API_VERSION_HEADER,
        GITHUB_API_VERSION_VALUE, GITHUB_COPILOT_API_TOKEN_ENV, GITHUB_MODELS_BASE_URL_ENV,
        GITHUB_MODELS_ORG_ENV, GITHUB_MODELS_TOKEN_ENV, GITHUB_TOKEN_ENV, OPENAI_API_KEY_ENV,
        OPENAI_BASE_URL_ENV, PROVIDER_TIMEOUT_SECS, ProviderAnalysisResponse, ProviderChatMessage,
        ProviderChatRole, ProviderNamespace, ProviderReviewDisposition, ProviderReviewRequest,
        ProviderReviewResponse, ProviderRevisionRequest, ProviderRuntimeError,
        ProviderWorkspaceFile, ResolvedProviderRoute, anthropic_messages_endpoint, chat_completion,
        compose_enriched_system_prompt, execute_openai_compatible_chat_with_exchange,
        first_json_object, parse_structured_response, persona_system_prompt_prefix,
        resolve_provider_route, review_workspace, revise_artifact,
        route_uses_explicit_provider_namespace,
    };

    struct EnvRestore<'a> {
        saved: BTreeMap<&'static str, Option<OsString>>,
        _lock: MutexGuard<'a, ()>,
    }

    impl Drop for EnvRestore<'_> {
        fn drop(&mut self) {
            unsafe {
                for (key, value) in &self.saved {
                    match value {
                        Some(value) => env::set_var(key, value),
                        None => env::remove_var(key),
                    }
                }
            }
        }
    }

    fn with_env_test<T>(tracked_keys: &[&'static str], action: impl FnOnce() -> T) -> T {
        let lock_result = super::super::SHARED_ENV_LOCK.get_or_init(|| Mutex::new(())).lock();
        let lock = match lock_result {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        let saved =
            tracked_keys.iter().map(|key| (*key, env::var_os(key))).collect::<BTreeMap<_, _>>();
        let restore = EnvRestore { saved, _lock: lock };

        unsafe {
            for key in tracked_keys {
                env::remove_var(key);
            }
        }

        let result = action();
        drop(restore);
        result
    }

    fn request_headers_complete(buffer: &[u8]) -> Option<usize> {
        buffer.windows(4).position(|window| window == b"\r\n\r\n").map(|index| index + 4)
    }

    fn request_content_length(buffer: &[u8]) -> Option<usize> {
        let headers_end = request_headers_complete(buffer)?;
        let headers = String::from_utf8_lossy(&buffer[..headers_end]);
        headers.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if !name.trim().eq_ignore_ascii_case("content-length") {
                return None;
            }
            value.trim().parse::<usize>().ok()
        })
    }

    fn request_complete(buffer: &[u8]) -> bool {
        match (request_headers_complete(buffer), request_content_length(buffer)) {
            (Some(headers_end), Some(content_length)) => {
                buffer.len() >= headers_end + content_length
            }
            (Some(_), None) => true,
            _ => false,
        }
    }

    fn spawn_single_response_server(
        response_body: String,
    ) -> Result<(String, mpsc::Receiver<String>, thread::JoinHandle<()>), String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = Vec::new();
                let mut chunk = [0_u8; 4096];
                loop {
                    match stream.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(read) => {
                            buffer.extend_from_slice(&chunk[..read]);
                            if request_complete(&buffer) {
                                break;
                            }
                        }
                        Err(_) => return,
                    }
                }

                let request_text = String::from_utf8_lossy(&buffer).to_string();
                let _ = sender.send(request_text);
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        Ok((format!("http://{address}"), receiver, handle))
    }

    fn spawn_path_response_server(
        responses: Vec<(&'static str, String)>,
    ) -> Result<(String, mpsc::Receiver<String>, thread::JoinHandle<()>), String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
        let address = listener.local_addr().map_err(|error| error.to_string())?;
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            for _ in 0..responses.len() {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buffer = Vec::new();
                let mut chunk = [0_u8; 4096];
                loop {
                    match stream.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(read) => {
                            buffer.extend_from_slice(&chunk[..read]);
                            if request_complete(&buffer) {
                                break;
                            }
                        }
                        Err(_) => return,
                    }
                }

                let request_text = String::from_utf8_lossy(&buffer).to_string();
                let _ = sender.send(request_text.clone());
                let path = request_text
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");
                let response_body = responses
                    .iter()
                    .find(|(candidate, _)| *candidate == path)
                    .map(|(_, body)| body.clone())
                    .unwrap_or_else(|| "{\"message\":\"not found\"}".to_string());
                let status = if responses.iter().any(|(candidate, _)| *candidate == path) {
                    "200 OK"
                } else {
                    "404 Not Found"
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        Ok((format!("http://{address}"), receiver, handle))
    }

    #[test]
    fn resolve_provider_route_uses_runtime_fallback_when_model_has_no_namespace() {
        with_env_test(&[COPILOT_GITHUB_TOKEN_ENV, GH_TOKEN_ENV, GITHUB_TOKEN_ENV], || {
            unsafe {
                env::set_var(COPILOT_GITHUB_TOKEN_ENV, "github-token");
            }
            let route = ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() };
            let resolved = resolve_provider_route(&route);
            assert!(resolved.is_ok());
            let resolved = match resolved {
                Ok(value) => value,
                Err(error) => panic!("unexpected error: {error}"),
            };
            assert_eq!(resolved.namespace.as_str(), "copilot");
            assert_eq!(resolved.model_id, "gpt-5.4".to_string());
            assert_eq!(resolved.base_url, DEFAULT_COPILOT_BASE_URL.to_string());
            assert_eq!(resolved.api_key.as_deref(), Some("github-token"));
        });
    }

    #[test]
    fn resolve_provider_route_prefers_explicit_model_namespace() {
        with_env_test(
            &[DEEPSEEK_API_KEY_ENV, COPILOT_GITHUB_TOKEN_ENV, GH_TOKEN_ENV, GITHUB_TOKEN_ENV],
            || {
                unsafe {
                    env::set_var(DEEPSEEK_API_KEY_ENV, "token");
                }
                let route = ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "deepseek/deepseek-chat".to_string(),
                };
                let resolved = resolve_provider_route(&route);
                assert!(resolved.is_ok());
                let resolved = match resolved {
                    Ok(value) => value,
                    Err(error) => panic!("unexpected error: {error}"),
                };
                assert_eq!(resolved.namespace.as_str(), "deepseek");
                assert_eq!(resolved.model_id, "deepseek-chat".to_string());
            },
        );
    }

    #[test]
    fn resolve_provider_route_supports_explicit_github_models_namespace() {
        with_env_test(
            &[
                GITHUB_MODELS_TOKEN_ENV,
                GITHUB_MODELS_ORG_ENV,
                GITHUB_MODELS_BASE_URL_ENV,
                GITHUB_TOKEN_ENV,
                COPILOT_GITHUB_TOKEN_ENV,
                GH_TOKEN_ENV,
            ],
            || {
                unsafe {
                    env::set_var(GITHUB_MODELS_TOKEN_ENV, "github-models-token");
                    env::set_var(GITHUB_MODELS_ORG_ENV, "octo-org");
                }
                let route = ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "github-models/openai/gpt-4.1".to_string(),
                };
                let resolved = resolve_provider_route(&route);
                assert!(resolved.is_ok());
                let resolved = match resolved {
                    Ok(value) => value,
                    Err(error) => panic!("unexpected error: {error}"),
                };
                assert_eq!(resolved.namespace.as_str(), "github-models");
                assert_eq!(resolved.model_id, "openai/gpt-4.1".to_string());
                assert_eq!(
                    resolved.base_url,
                    "https://models.github.ai/orgs/octo-org/inference".to_string()
                );
                assert_eq!(resolved.api_key.as_deref(), Some("github-models-token"));
            },
        );
    }

    #[test]
    fn resolve_provider_route_prefers_explicit_copilot_token_over_generic_github_tokens() {
        with_env_test(
            &[COPILOT_GITHUB_TOKEN_ENV, GH_TOKEN_ENV, GITHUB_TOKEN_ENV, COPILOT_API_KEY_ENV],
            || {
                unsafe {
                    env::set_var(COPILOT_GITHUB_TOKEN_ENV, "copilot-token");
                    env::set_var(GH_TOKEN_ENV, "gh-token");
                    env::set_var(GITHUB_TOKEN_ENV, "github-token");
                    env::set_var(COPILOT_API_KEY_ENV, "legacy-token");
                }
                let route =
                    ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() };
                let resolved = resolve_provider_route(&route);
                assert!(resolved.is_ok());
                let resolved = match resolved {
                    Ok(value) => value,
                    Err(error) => panic!("unexpected error: {error}"),
                };
                assert_eq!(resolved.api_key.as_deref(), Some("copilot-token"));
                assert_eq!(resolved.base_url, DEFAULT_COPILOT_BASE_URL.to_string());
            },
        );
    }

    #[test]
    fn resolve_provider_route_prefers_direct_copilot_api_token_and_url() {
        with_env_test(
            &[
                GITHUB_COPILOT_API_TOKEN_ENV,
                COPILOT_API_URL_ENV,
                COPILOT_GITHUB_TOKEN_ENV,
                GH_TOKEN_ENV,
                GITHUB_TOKEN_ENV,
                COPILOT_API_KEY_ENV,
            ],
            || {
                unsafe {
                    env::set_var(GITHUB_COPILOT_API_TOKEN_ENV, "direct-copilot-token");
                    env::set_var(COPILOT_API_URL_ENV, "https://copilot.example.test");
                    env::set_var(COPILOT_GITHUB_TOKEN_ENV, "github-bootstrap-token");
                }
                let route =
                    ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() };
                let resolved =
                    resolve_provider_route(&route).expect("copilot route should resolve");
                assert_eq!(resolved.api_key.as_deref(), Some("direct-copilot-token"));
                assert_eq!(resolved.base_url, "https://copilot.example.test".to_string());
            },
        );
    }

    #[test]
    fn resolve_provider_route_does_not_fallback_to_openai_for_copilot_runtime() {
        with_env_test(
            &[
                COPILOT_GITHUB_TOKEN_ENV,
                GH_TOKEN_ENV,
                GITHUB_TOKEN_ENV,
                COPILOT_API_KEY_ENV,
                OPENAI_API_KEY_ENV,
            ],
            || {
                unsafe {
                    env::set_var(OPENAI_API_KEY_ENV, "openai-token");
                }
                let route =
                    ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() };
                let resolved = resolve_provider_route(&route);
                // When a stored auth profile exists, Copilot credentials resolve
                // successfully from the profile; confirm the OpenAI token is never
                // used. When no profile exists, credentials should be missing.
                match resolved {
                    Ok(r) => assert_ne!(
                        r.api_key.as_deref(),
                        Some("openai-token"),
                        "Copilot route must not use OpenAI credentials"
                    ),
                    Err(ref e) => assert!(matches!(
                        e,
                        ProviderRuntimeError::MissingApiKey { namespace, env_key }
                            if *namespace == "copilot" && *env_key == COPILOT_TOKEN_ENV_HINT
                    )),
                }
            },
        );
    }

    #[test]
    fn resolve_provider_route_ignores_legacy_copilot_base_url_override() {
        with_env_test(&[COPILOT_GITHUB_TOKEN_ENV], || {
            unsafe {
                env::set_var(COPILOT_GITHUB_TOKEN_ENV, "github-token");
                env::set_var("COPILOT_BASE_URL", "https://example.invalid");
            }
            let route = ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() };
            let resolved = resolve_provider_route(&route);
            assert!(resolved.is_ok());
            let resolved = match resolved {
                Ok(value) => value,
                Err(error) => panic!("unexpected error: {error}"),
            };
            assert_eq!(resolved.base_url, DEFAULT_COPILOT_BASE_URL.to_string());
            unsafe {
                env::remove_var("COPILOT_BASE_URL");
            }
        });
    }

    #[test]
    fn resolve_provider_route_uses_documented_deepseek_base_url() {
        with_env_test(&[DEEPSEEK_API_KEY_ENV, DEEPSEEK_BASE_URL_ENV], || {
            unsafe {
                env::set_var(DEEPSEEK_API_KEY_ENV, "deepseek-token");
            }
            let route = ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "deepseek/deepseek-chat".to_string(),
            };
            let resolved = resolve_provider_route(&route);
            assert!(resolved.is_ok());
            let resolved = match resolved {
                Ok(value) => value,
                Err(error) => panic!("unexpected error: {error}"),
            };
            assert_eq!(resolved.base_url, DEFAULT_DEEPSEEK_BASE_URL.to_string());
        });
    }

    #[test]
    fn resolve_provider_route_uses_configurable_anthropic_base_url() {
        with_env_test(&[ANTHROPIC_API_KEY_ENV, ANTHROPIC_BASE_URL_ENV], || {
            unsafe {
                env::set_var(ANTHROPIC_API_KEY_ENV, "anthropic-token");
                env::set_var(ANTHROPIC_BASE_URL_ENV, "https://api.deepseek.com/anthropic");
            }
            let route =
                ModelRoute { runtime: RuntimeKind::Claude, model: "claude-sonnet-4".to_string() };
            let resolved = resolve_provider_route(&route);
            assert!(resolved.is_ok());
            let resolved = match resolved {
                Ok(value) => value,
                Err(error) => panic!("unexpected error: {error}"),
            };
            assert_eq!(resolved.base_url, "https://api.deepseek.com/anthropic".to_string());
        });
    }

    #[test]
    fn anthropic_messages_endpoint_handles_root_and_prefixed_base_urls() {
        assert_eq!(
            anthropic_messages_endpoint(DEFAULT_ANTHROPIC_BASE_URL),
            "https://api.anthropic.com/v1/messages".to_string()
        );
        assert_eq!(
            anthropic_messages_endpoint("https://api.deepseek.com/anthropic"),
            "https://api.deepseek.com/anthropic/messages".to_string()
        );
        assert_eq!(
            anthropic_messages_endpoint("https://api.anthropic.com/v1"),
            "https://api.anthropic.com/v1/messages".to_string()
        );
    }

    #[test]
    fn resolve_provider_route_uses_openai_for_codex_runtime() {
        with_env_test(&[OPENAI_API_KEY_ENV], || {
            unsafe {
                env::set_var(OPENAI_API_KEY_ENV, "token");
            }
            let route = ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5.4".to_string() };
            let resolved = resolve_provider_route(&route);
            assert!(resolved.is_ok());
            let resolved = match resolved {
                Ok(value) => value,
                Err(error) => panic!("unexpected error: {error}"),
            };
            assert_eq!(resolved.namespace.as_str(), "openai");
            assert_eq!(resolved.model_id, "gpt-5.4".to_string());
        });
    }

    #[test]
    fn route_uses_explicit_provider_namespace_requires_supported_prefix() {
        assert!(route_uses_explicit_provider_namespace(&ModelRoute {
            runtime: RuntimeKind::Copilot,
            model: "openai/gpt-5.4".to_string(),
        }));
        assert!(!route_uses_explicit_provider_namespace(&ModelRoute {
            runtime: RuntimeKind::Copilot,
            model: "gpt-5.4".to_string(),
        }));
        assert!(!route_uses_explicit_provider_namespace(&ModelRoute {
            runtime: RuntimeKind::Copilot,
            model: "unknown/gpt-5.4".to_string(),
        }));
    }

    #[test]
    fn parse_structured_response_accepts_fenced_json() {
        let response = parse_structured_response::<ProviderAnalysisResponse>(
            "```json\n{\"headline\":\"ok\",\"summary\":\"done\",\"risks\":[\"none\"]}\n```",
        );
        assert!(response.is_ok());
        let response = match response {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error}"),
        };
        assert_eq!(response.headline, "ok".to_string());
        assert_eq!(response.summary, "done".to_string());
        assert_eq!(response.risks, vec!["none".to_string()]);
    }

    #[test]
    fn first_json_object_extracts_embedded_payload() {
        let payload = first_json_object("prefix {\"headline\":\"ok\"} suffix");
        assert_eq!(payload, Some("{\"headline\":\"ok\"}".to_string()));
    }

    #[test]
    fn parse_structured_response_reports_invalid_json() {
        let response = parse_structured_response::<ProviderAnalysisResponse>("not json");
        assert!(matches!(response, Err(ProviderRuntimeError::BadResponse(_))));
    }

    #[test]
    fn parse_structured_review_response_accepts_enum_payload() {
        let response = parse_structured_response::<ProviderReviewResponse>(
            r#"{"disposition":"concern","summary":"needs a retry","required_action":"rerun tests","evidence_refs":["src/lib.rs"]}"#,
        );
        assert!(response.is_ok());
        let response = match response {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error}"),
        };
        assert_eq!(response.disposition, ProviderReviewDisposition::Concern);
        assert_eq!(response.summary, "needs a retry");
        assert_eq!(response.required_action.as_deref(), Some("rerun tests"));
        assert_eq!(response.evidence_refs, vec!["src/lib.rs".to_string()]);
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn review_workspace_dispatches_to_openai_compatible_endpoint() {
        with_env_test(&[OPENAI_API_KEY_ENV, OPENAI_BASE_URL_ENV], || {
            let response_body = json!({
                "choices": [
                    {
                        "message": {
                            "content": "{\"disposition\":\"approve\",\"summary\":\"ready to merge\",\"evidence_refs\":[\"src/lib.rs\"]}"
                        }
                    }
                ]
            })
            .to_string();
            let server = spawn_single_response_server(response_body);
            assert!(server.is_ok());
            let (base_url, receiver, handle) = match server {
                Ok(value) => value,
                Err(error) => panic!("unexpected server error: {error}"),
            };

            unsafe {
                env::set_var(OPENAI_API_KEY_ENV, "token");
                env::set_var(OPENAI_BASE_URL_ENV, &base_url);
            }

            let response = review_workspace(
                &ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "openai/test-review-model".to_string(),
                },
                &ProviderReviewRequest {
                    goal: "Review the workspace".to_string(),
                    phase: "review".to_string(),
                    reviewer_id: "safety".to_string(),
                    reviewer_role: "Safety".to_string(),
                    attempt_id: "attempt-1".to_string(),
                    files: vec![ProviderWorkspaceFile {
                        path: "src/lib.rs".to_string(),
                        contents: "pub fn add(left: i32, right: i32) -> i32 { left + right }"
                            .to_string(),
                    }],
                    prior_context: json!({"latest_validation_status": "passed"}),
                },
            );
            assert!(response.is_ok());
            let response = match response {
                Ok(value) => value,
                Err(error) => panic!("unexpected review error: {error}"),
            };
            assert_eq!(response.disposition, ProviderReviewDisposition::Approve);
            assert_eq!(response.summary, "ready to merge");
            assert_eq!(response.evidence_refs, vec!["src/lib.rs".to_string()]);

            let request_text = receiver.recv().ok();
            assert!(request_text.is_some());
            let request_text = request_text.unwrap_or_default();
            let request_text_lower = request_text.to_ascii_lowercase();
            assert!(request_text.contains("POST /chat/completions"), "{request_text}");
            assert!(request_text_lower.contains("authorization: bearer token"), "{request_text}");
            assert!(request_text.contains("test-review-model"), "{request_text}");
            assert!(request_text.contains("Review the workspace"), "{request_text}");
            assert!(request_text.contains("Safety (safety)"), "{request_text}");
            assert!(request_text.contains("latest_validation_status"), "{request_text}");

            assert!(handle.join().is_ok());
        });
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn chat_completion_dispatches_history_to_openai_compatible_endpoint() {
        with_env_test(&[OPENAI_API_KEY_ENV, OPENAI_BASE_URL_ENV], || {
            let response_body = json!({
                "choices": [
                    {
                        "message": {
                            "content": "ready"
                        }
                    }
                ]
            })
            .to_string();
            let server = spawn_single_response_server(response_body);
            assert!(server.is_ok());
            let (base_url, receiver, handle) = match server {
                Ok(value) => value,
                Err(error) => panic!("unexpected server error: {error}"),
            };

            unsafe {
                env::set_var(OPENAI_API_KEY_ENV, "token");
                env::set_var(OPENAI_BASE_URL_ENV, &base_url);
            }

            let response = chat_completion(
                &ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/test-chat-model".to_string(),
                },
                &[
                    ProviderChatMessage {
                        role: ProviderChatRole::System,
                        content: "Keep answers concise.".to_string(),
                    },
                    ProviderChatMessage {
                        role: ProviderChatRole::User,
                        content: "Summarize the current state.".to_string(),
                    },
                    ProviderChatMessage {
                        role: ProviderChatRole::Assistant,
                        content: "Previous answer.".to_string(),
                    },
                ],
                Some(256),
            );
            assert!(response.is_ok());
            assert_eq!(response.unwrap_or_default(), "ready".to_string());

            let request_text = receiver.recv().ok();
            assert!(request_text.is_some());
            let request_text = request_text.unwrap_or_default();
            let request_text_lower = request_text.to_ascii_lowercase();
            assert!(request_text.contains("POST /chat/completions"), "{request_text}");
            assert!(request_text_lower.contains("authorization: bearer token"), "{request_text}");
            assert!(request_text.contains("test-chat-model"), "{request_text}");
            assert!(request_text.contains("Keep answers concise."), "{request_text}");
            assert!(request_text.contains("Summarize the current state."), "{request_text}");
            assert!(request_text.contains("Previous answer."), "{request_text}");
            assert!(request_text.contains("max_tokens"), "{request_text}");

            assert!(handle.join().is_ok());
        });
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn revise_artifact_dispatches_revision_prompt_to_openai_compatible_endpoint() {
        with_env_test(&[OPENAI_API_KEY_ENV, OPENAI_BASE_URL_ENV], || {
            let response_body = json!({
                "choices": [
                    {
                        "message": {
                            "content": json!({
                                "headline": "revised discovery brief",
                                "summary": "accepted persistence and validation feedback",
                                "revised_artifact": "# Discovery\n\n- persistence: postgres\n- validation: cargo test\n",
                                "applied_feedback": [
                                    "add persistence choice",
                                    "add validation target"
                                ]
                            }).to_string()
                        }
                    }
                ]
            })
            .to_string();
            let server = spawn_single_response_server(response_body);
            assert!(server.is_ok());
            let (base_url, receiver, handle) = match server {
                Ok(value) => value,
                Err(error) => panic!("unexpected server error: {error}"),
            };

            unsafe {
                env::set_var(OPENAI_API_KEY_ENV, "token");
                env::set_var(OPENAI_BASE_URL_ENV, &base_url);
            }

            let response = revise_artifact(
                &ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "openai/test-revision-model".to_string(),
                },
                &ProviderRevisionRequest {
                    goal: "Revise the discovery brief".to_string(),
                    phase: "planning".to_string(),
                    reviser_id: "reviser".to_string(),
                    target_refs: vec![
                        ".boundline/governance/planning/discovery/brief.md".to_string(),
                    ],
                    current_artifact: "# Discovery\n\n- persistence: TBD\n".to_string(),
                    accepted_feedback: vec![
                        "add persistence choice".to_string(),
                        "add validation target".to_string(),
                    ],
                    prior_context: json!({"vote_strategy": "majority"}),
                },
            );
            assert!(response.is_ok());
            let response = match response {
                Ok(value) => value,
                Err(error) => panic!("unexpected revision error: {error}"),
            };
            assert_eq!(response.headline, "revised discovery brief");
            assert!(response.revised_artifact.contains("postgres"));
            assert_eq!(
                response.applied_feedback,
                vec!["add persistence choice".to_string(), "add validation target".to_string()]
            );

            let request_text = receiver.recv().ok();
            assert!(request_text.is_some());
            let request_text = request_text.unwrap_or_default();
            let request_text_lower = request_text.to_ascii_lowercase();
            assert!(request_text.contains("POST /chat/completions"), "{request_text}");
            assert!(request_text_lower.contains("authorization: bearer token"), "{request_text}");
            assert!(request_text.contains("test-revision-model"), "{request_text}");
            assert!(request_text.contains("Revise the discovery brief"), "{request_text}");
            assert!(request_text.contains("add persistence choice"), "{request_text}");
            assert!(request_text.contains("vote_strategy"), "{request_text}");

            assert!(handle.join().is_ok());
        });
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn chat_completion_uses_copilot_token_exchange_before_chat_request() {
        let exchange_response_body = json!({
            "token": "copilot-session-token"
        })
        .to_string();
        let exchange_server = spawn_single_response_server(exchange_response_body);
        assert!(exchange_server.is_ok());
        let (exchange_base_url, exchange_receiver, exchange_handle) = match exchange_server {
            Ok(value) => value,
            Err(error) => panic!("unexpected exchange server error: {error}"),
        };

        let chat_response_body = json!({
            "choices": [
                {
                    "message": {
                        "content": "ready"
                    }
                }
            ]
        })
        .to_string();
        let chat_server = spawn_single_response_server(chat_response_body);
        assert!(chat_server.is_ok());
        let (chat_base_url, chat_receiver, chat_handle) = match chat_server {
            Ok(value) => value,
            Err(error) => panic!("unexpected chat server error: {error}"),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(PROVIDER_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());
        let route = ResolvedProviderRoute {
            namespace: ProviderNamespace::Copilot,
            model_id: "gpt-5.4".to_string(),
            base_url: chat_base_url,
            api_key: Some("gho_test_token".to_string()),
        };

        let response = execute_openai_compatible_chat_with_exchange(
            &client,
            &route,
            &[ProviderChatMessage { role: ProviderChatRole::User, content: "hello".to_string() }],
            Some(64),
            &format!("{exchange_base_url}/copilot_internal/v2/token"),
        );
        assert!(response.is_ok());
        assert_eq!(response.unwrap_or_default(), "ready".to_string());

        let exchange_request = exchange_receiver.recv().ok();
        assert!(exchange_request.is_some());
        let exchange_request = exchange_request.unwrap_or_default();
        let exchange_request_lower = exchange_request.to_ascii_lowercase();
        assert!(exchange_request.contains("GET /copilot_internal/v2/token"), "{exchange_request}");
        assert!(
            exchange_request_lower.contains("authorization: bearer gho_test_token"),
            "{exchange_request}"
        );
        assert!(
            exchange_request_lower.contains(&format!(
                "{}: {}",
                COPILOT_EDITOR_VERSION_HEADER.to_ascii_lowercase(),
                COPILOT_EDITOR_VERSION_VALUE.to_ascii_lowercase()
            )),
            "{exchange_request}"
        );
        assert!(
            exchange_request_lower.contains(&format!(
                "{}: {}",
                COPILOT_EDITOR_PLUGIN_VERSION_HEADER.to_ascii_lowercase(),
                COPILOT_EDITOR_PLUGIN_VERSION_VALUE.to_ascii_lowercase()
            )),
            "{exchange_request}"
        );
        assert!(
            exchange_request_lower
                .contains(&format!("copilot-integration-id: {}", COPILOT_INTEGRATION_ID_VALUE)),
            "{exchange_request}"
        );
        assert!(
            exchange_request_lower.contains(&format!(
                "{}: {}",
                COPILOT_GITHUB_API_VERSION_HEADER.to_ascii_lowercase(),
                COPILOT_GITHUB_API_VERSION_VALUE.to_ascii_lowercase()
            )),
            "{exchange_request}"
        );
        assert!(
            exchange_request_lower.contains(&format!(
                "{}: {}",
                COPILOT_ACCEPT_ENCODING_HEADER.to_ascii_lowercase(),
                COPILOT_ACCEPT_ENCODING_VALUE.to_ascii_lowercase()
            )),
            "{exchange_request}"
        );
        assert!(
            exchange_request_lower
                .contains(&format!("user-agent: {}", COPILOT_USER_AGENT.to_ascii_lowercase())),
            "{exchange_request}"
        );

        let chat_request = chat_receiver.recv().ok();
        assert!(chat_request.is_some());
        let chat_request = chat_request.unwrap_or_default();
        let chat_request_lower = chat_request.to_ascii_lowercase();
        assert!(chat_request.contains("POST /chat/completions"), "{chat_request}");
        assert!(
            chat_request_lower.contains("authorization: bearer copilot-session-token"),
            "{chat_request}"
        );
        assert!(
            chat_request_lower
                .contains(&format!("copilot-integration-id: {}", COPILOT_INTEGRATION_ID_VALUE)),
            "{chat_request}"
        );

        assert!(exchange_handle.join().is_ok());
        assert!(chat_handle.join().is_ok());
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn chat_completion_uses_direct_copilot_api_token_without_bootstrap() {
        with_env_test(
            &[
                GITHUB_COPILOT_API_TOKEN_ENV,
                COPILOT_API_URL_ENV,
                COPILOT_GITHUB_TOKEN_ENV,
                GH_TOKEN_ENV,
                GITHUB_TOKEN_ENV,
                COPILOT_API_KEY_ENV,
            ],
            || {
                let response_body = json!({
                    "choices": [
                        {
                            "message": {
                                "content": "ready"
                            }
                        }
                    ]
                })
                .to_string();
                let server =
                    spawn_single_response_server(response_body).expect("server should start");
                let (base_url, receiver, handle) = server;

                unsafe {
                    env::set_var(GITHUB_COPILOT_API_TOKEN_ENV, "direct-copilot-token");
                    env::set_var(COPILOT_API_URL_ENV, base_url.clone());
                }

                let response = chat_completion(
                    &ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() },
                    &[ProviderChatMessage {
                        role: ProviderChatRole::User,
                        content: "hello".to_string(),
                    }],
                    Some(64),
                );
                assert!(response.is_ok());
                assert_eq!(response.unwrap_or_default(), "ready".to_string());

                let request_text = receiver.recv().expect("chat request should be captured");
                let request_text_lower = request_text.to_ascii_lowercase();
                assert!(request_text.contains("POST /chat/completions"), "{request_text}");
                assert!(
                    request_text_lower.contains("authorization: bearer direct-copilot-token"),
                    "{request_text}"
                );
                assert!(handle.join().is_ok());
            },
        );
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn chat_completion_bootstraps_copilot_runtime_from_user_endpoint_before_chat_request() {
        let chat_response_body = json!({
            "choices": [
                {
                    "message": {
                        "content": "ready"
                    }
                }
            ]
        })
        .to_string();
        let chat_server = spawn_single_response_server(chat_response_body).expect("chat server");
        let (chat_base_url, chat_receiver, chat_handle) = chat_server;

        let user_response_body = json!({
            "login": "sdk-e2e-user",
            "copilot_plan": "individual_pro",
            "endpoints": {
                "api": chat_base_url,
                "telemetry": "https://localhost:1/telemetry"
            }
        })
        .to_string();
        let user_server =
            spawn_path_response_server(vec![("/copilot_internal/user", user_response_body)])
                .expect("user bootstrap server");
        let (user_base_url, user_receiver, user_handle) = user_server;

        let client = Client::builder()
            .timeout(Duration::from_secs(PROVIDER_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());
        let route = ResolvedProviderRoute {
            namespace: ProviderNamespace::Copilot,
            model_id: "gpt-5.4".to_string(),
            base_url: DEFAULT_COPILOT_BASE_URL.to_string(),
            api_key: Some("gho_test_token".to_string()),
        };

        let response = super::copilot::execute_chat_with_exchange_and_bootstrap(
            &client,
            &route,
            &[ProviderChatMessage { role: ProviderChatRole::User, content: "hello".to_string() }],
            Some(64),
            "http://127.0.0.1:9/copilot_internal/v2/token",
            &format!("{user_base_url}/copilot_internal/user"),
        );
        assert!(response.is_ok());
        assert_eq!(response.unwrap_or_default(), "ready".to_string());

        let user_request = user_receiver.recv().expect("user bootstrap request should be captured");
        let user_request_lower = user_request.to_ascii_lowercase();
        assert!(user_request.contains("GET /copilot_internal/user"), "{user_request}");
        assert!(
            user_request_lower.contains("authorization: bearer gho_test_token"),
            "{user_request}"
        );

        let chat_request = chat_receiver.recv().expect("chat request should be captured");
        let chat_request_lower = chat_request.to_ascii_lowercase();
        assert!(chat_request.contains("POST /chat/completions"), "{chat_request}");
        assert!(
            chat_request_lower.contains("authorization: bearer gho_test_token"),
            "{chat_request}"
        );

        assert!(user_handle.join().is_ok());
        assert!(chat_handle.join().is_ok());
    }

    #[test]
    #[cfg_attr(coverage, ignore = "coverage sandbox disallows loopback provider mock servers")]
    fn chat_completion_dispatches_to_github_models_inference_endpoint() {
        with_env_test(&[GITHUB_MODELS_TOKEN_ENV, GITHUB_MODELS_BASE_URL_ENV], || {
            let response_body = json!({
                "choices": [
                    {
                        "message": {
                            "content": "ready"
                        }
                    }
                ]
            })
            .to_string();
            let server = spawn_single_response_server(response_body);
            assert!(server.is_ok());
            let (base_url, receiver, handle) = match server {
                Ok(value) => value,
                Err(error) => panic!("unexpected server error: {error}"),
            };

            unsafe {
                env::set_var(GITHUB_MODELS_TOKEN_ENV, "github-models-token");
                env::set_var(GITHUB_MODELS_BASE_URL_ENV, format!("{base_url}/inference"));
            }

            let response = chat_completion(
                &ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "github-models/openai/gpt-4.1".to_string(),
                },
                &[ProviderChatMessage {
                    role: ProviderChatRole::User,
                    content: "Summarize the current state.".to_string(),
                }],
                Some(128),
            );
            assert!(response.is_ok());
            assert_eq!(response.unwrap_or_default(), "ready".to_string());

            let request_text = receiver.recv().ok();
            assert!(request_text.is_some());
            let request_text = request_text.unwrap_or_default();
            let request_text_lower = request_text.to_ascii_lowercase();
            assert!(request_text.contains("POST /inference/chat/completions"), "{request_text}");
            assert!(
                request_text_lower.contains("authorization: bearer github-models-token"),
                "{request_text}"
            );
            assert!(
                request_text_lower.contains(&format!(
                    "accept: {}",
                    GITHUB_API_ACCEPT_HEADER_VALUE.to_ascii_lowercase()
                )),
                "{request_text}"
            );
            assert!(
                request_text_lower.contains(&format!(
                    "{}: {}",
                    GITHUB_API_VERSION_HEADER.to_ascii_lowercase(),
                    GITHUB_API_VERSION_VALUE.to_ascii_lowercase()
                )),
                "{request_text}"
            );
            assert!(request_text.contains("openai/gpt-4.1"), "{request_text}");
            assert!(request_text.contains("Summarize the current state."), "{request_text}");

            assert!(handle.join().is_ok());
        });
    }

    #[test]
    fn copilot_session_token_derives_runtime_base_url_from_proxy_hint() {
        let derived = super::copilot::derive_runtime_base_url_from_token(
            "copilot-session-token;proxy-ep=https://proxy.individual.githubcopilot.com;",
        );
        assert_eq!(derived.as_deref(), Some(DEFAULT_COPILOT_BASE_URL));

        let derived = super::copilot::derive_runtime_base_url_from_token(
            "copilot-session-token;proxy-ep=proxy.contoso.test:8443;",
        );
        assert_eq!(derived.as_deref(), Some("https://api.contoso.test"));

        let derived = super::copilot::derive_runtime_base_url_from_token(
            "copilot-session-token;proxy-ep=javascript:alert(1);",
        );
        assert!(derived.is_none());
    }

    #[test]
    fn copilot_exchange_403_hints_direct_or_bootstrap_paths_for_personal_tokens() {
        let hint = super::copilot::token_exchange_error_hint("ghp_example", 403);
        assert!(hint.is_some());
        let hint = hint.unwrap_or_default();
        assert!(hint.contains("personal access token"), "{hint}");
        assert!(hint.contains("GITHUB_COPILOT_API_TOKEN"), "{hint}");

        let oauth_hint = super::copilot::token_exchange_error_hint("gho_example", 403);
        assert!(oauth_hint.is_some());

        let no_hint = super::copilot::token_exchange_error_hint("ghp_example", 500);
        assert!(no_hint.is_none());
    }

    #[test]
    fn persona_system_prompt_prefix_covers_known_personas() {
        assert!(persona_system_prompt_prefix("delivery-engineer").contains("delivery engineer"));
        assert!(persona_system_prompt_prefix("system-architect").contains("system architect"));
        assert!(persona_system_prompt_prefix("verification-lead").contains("verification lead"));
        assert_eq!(persona_system_prompt_prefix("unknown"), "");
    }

    #[test]
    fn compose_enriched_system_prompt_combines_persona_and_guidance() {
        let prompt = compose_enriched_system_prompt(
            "Base prompt.",
            &["# Rust\nUse Result-based errors.".to_string()],
            Some("delivery-engineer"),
        );

        assert!(prompt.starts_with("You are acting as a delivery engineer."), "{prompt}");
        assert!(prompt.contains("Base prompt."), "{prompt}");
        assert!(prompt.contains("## Implementation Guidance"), "{prompt}");
        assert!(prompt.contains("### Guidance 1"), "{prompt}");
        assert!(prompt.contains("Use Result-based errors."), "{prompt}");
    }
}
