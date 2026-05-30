use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use super::{
    OPENAI_API_KEY_ENV, OPENAI_BASE_URL_ENV, OPENAI_CHAT_COMPLETIONS_PATH, ProviderChatMessage,
    ProviderChatRole, ProviderNamespace, ProviderRuntimeError, ResolvedProviderRoute,
};

pub(super) type RequestHeader = (&'static str, &'static str);

pub(super) fn execute_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
    bearer_token: Option<String>,
    headers: &[RequestHeader],
) -> Result<String, ProviderRuntimeError> {
    let messages = [
        ProviderChatMessage { role: ProviderChatRole::System, content: system_prompt.to_string() },
        ProviderChatMessage { role: ProviderChatRole::User, content: user_prompt.to_string() },
    ];
    execute_chat(client, route, &messages, None, bearer_token, headers)
}

pub(super) fn execute_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
    bearer_token: Option<String>,
    headers: &[RequestHeader],
) -> Result<String, ProviderRuntimeError> {
    #[derive(Serialize)]
    struct OpenAiChatRequestMessage<'a> {
        role: &'a str,
        content: &'a str,
    }

    #[derive(Serialize)]
    struct OpenAiChatRequest<'a> {
        model: &'a str,
        messages: Vec<OpenAiChatRequestMessage<'a>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_tokens: Option<u32>,
    }

    let endpoint =
        format!("{}{}", route.base_url.trim_end_matches('/'), OPENAI_CHAT_COMPLETIONS_PATH);
    let body = OpenAiChatRequest {
        model: &route.model_id,
        messages: messages
            .iter()
            .map(|message| OpenAiChatRequestMessage {
                role: message.role.as_str(),
                content: message.content.as_str(),
            })
            .collect(),
        max_tokens,
    };

    let mut request = client.post(&endpoint).json(&body);
    if let Some(api_key) = bearer_token.as_deref() {
        request = request.bearer_auth(api_key);
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }

    let response = super::execute_with_retry(request, "openai_compatible", &route.model_id)?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().unwrap_or_default();
        return Err(ProviderRuntimeError::Api { status, body });
    }

    let parsed: OpenAiCompletionResponse =
        response.json().map_err(|error| ProviderRuntimeError::BadResponse(error.to_string()))?;
    let content =
        parsed.choices.into_iter().find_map(|choice| choice.message.content).unwrap_or_default();
    if content.trim().is_empty() {
        return Err(ProviderRuntimeError::BadResponse(
            "provider returned an empty completion".to_string(),
        ));
    }

    Ok(content)
}

pub(super) fn resolve_credentials(
    namespace: ProviderNamespace,
    api_key_env: &'static str,
    base_url_env: &'static str,
    default_base_url: &'static str,
    allow_openai_fallback: bool,
) -> Result<(String, Option<String>), ProviderRuntimeError> {
    let api_key = super::env_string(api_key_env);
    let base_url = super::env_string(base_url_env);

    if api_key.is_some() || base_url.is_some() || !allow_openai_fallback {
        let resolved_api_key = match api_key {
            Some(value) => value,
            None => {
                return Err(ProviderRuntimeError::MissingApiKey {
                    namespace: namespace.as_str(),
                    env_key: api_key_env,
                });
            }
        };
        return Ok((
            base_url.unwrap_or_else(|| default_base_url.to_string()),
            Some(resolved_api_key),
        ));
    }

    let openai_api_key = super::required_env(ProviderNamespace::OpenAi, OPENAI_API_KEY_ENV)?;
    let openai_base_url =
        super::env_string(OPENAI_BASE_URL_ENV).unwrap_or_else(|| default_base_url.to_string());
    Ok((openai_base_url, Some(openai_api_key)))
}

#[derive(Debug, Deserialize)]
struct OpenAiCompletionResponse {
    choices: Vec<OpenAiCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompletionChoice {
    message: OpenAiCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiCompletionMessage {
    content: Option<String>,
}
