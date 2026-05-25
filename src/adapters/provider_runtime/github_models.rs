use reqwest::blocking::Client;

use super::{
    COPILOT_GITHUB_TOKEN_ENV, DEFAULT_GITHUB_MODELS_BASE_URL, GH_TOKEN_ENV,
    GITHUB_API_ACCEPT_HEADER_VALUE, GITHUB_API_VERSION_HEADER, GITHUB_API_VERSION_VALUE,
    GITHUB_MODELS_BASE_URL_ENV, GITHUB_MODELS_ORG_ENV, GITHUB_MODELS_TOKEN_ENV,
    GITHUB_MODELS_TOKEN_ENV_HINT, GITHUB_TOKEN_ENV, ProviderChatMessage, ProviderNamespace,
    ProviderRuntimeError, ResolvedProviderRoute, openai_compatible,
};

const GITHUB_MODELS_API_ROOT: &str = "https://models.github.ai";
const GITHUB_ORGS_PATH: &str = "/orgs/";
const GITHUB_INFERENCE_PATH: &str = "/inference";
const GITHUB_MODELS_REQUEST_HEADERS: [openai_compatible::RequestHeader; 2] = [
    ("Accept", GITHUB_API_ACCEPT_HEADER_VALUE),
    (GITHUB_API_VERSION_HEADER, GITHUB_API_VERSION_VALUE),
];

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    let api_key = super::env_string(GITHUB_MODELS_TOKEN_ENV)
        .or_else(|| super::env_string(GITHUB_TOKEN_ENV))
        .or_else(|| super::env_string(COPILOT_GITHUB_TOKEN_ENV))
        .or_else(|| super::env_string(GH_TOKEN_ENV));

    match api_key {
        Some(api_key) => Ok((resolve_base_url(), Some(api_key))),
        None => Err(ProviderRuntimeError::MissingApiKey {
            namespace: ProviderNamespace::GitHubModels.as_str(),
            env_key: GITHUB_MODELS_TOKEN_ENV_HINT,
        }),
    }
}

pub(super) fn execute_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ProviderRuntimeError> {
    openai_compatible::execute_prompt(
        client,
        route,
        system_prompt,
        user_prompt,
        route.api_key.clone(),
        &GITHUB_MODELS_REQUEST_HEADERS,
    )
}

pub(super) fn execute_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    openai_compatible::execute_chat(
        client,
        route,
        messages,
        max_tokens,
        route.api_key.clone(),
        &GITHUB_MODELS_REQUEST_HEADERS,
    )
}

fn resolve_base_url() -> String {
    if let Some(base_url) = super::env_string(GITHUB_MODELS_BASE_URL_ENV) {
        return base_url;
    }

    if let Some(org) = super::env_string(GITHUB_MODELS_ORG_ENV) {
        return format!("{GITHUB_MODELS_API_ROOT}{GITHUB_ORGS_PATH}{org}{GITHUB_INFERENCE_PATH}");
    }

    DEFAULT_GITHUB_MODELS_BASE_URL.to_string()
}
