use reqwest::blocking::Client;

use super::{
    DEFAULT_OLLAMA_BASE_URL, OLLAMA_BASE_URL_ENV, ProviderChatMessage, ProviderRuntimeError,
    ResolvedProviderRoute, openai_compatible,
};

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    Ok((
        super::env_string(OLLAMA_BASE_URL_ENV)
            .unwrap_or_else(|| DEFAULT_OLLAMA_BASE_URL.to_string()),
        None,
    ))
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
        &[],
    )
}

pub(super) fn execute_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    openai_compatible::execute_chat(client, route, messages, max_tokens, route.api_key.clone(), &[])
}
