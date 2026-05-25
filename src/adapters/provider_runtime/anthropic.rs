use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use super::{
    ANTHROPIC_API_KEY_ENV, ANTHROPIC_ASSISTANT_ROLE, ANTHROPIC_BASE_URL_ENV,
    ANTHROPIC_MESSAGES_PATH, ANTHROPIC_ROOT_MESSAGES_PATH, ANTHROPIC_TEXT_BLOCK_KIND,
    ANTHROPIC_USER_ROLE, ANTHROPIC_VERSION_HEADER, ANTHROPIC_VERSION_VALUE,
    DEFAULT_ANTHROPIC_BASE_URL, DEFAULT_PROVIDER_CHAT_MAX_TOKENS, ProviderChatMessage,
    ProviderChatRole, ProviderNamespace, ProviderRuntimeError, ResolvedProviderRoute,
};

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    Ok((
        super::env_string(ANTHROPIC_BASE_URL_ENV)
            .unwrap_or_else(|| DEFAULT_ANTHROPIC_BASE_URL.to_string()),
        Some(super::required_env(ProviderNamespace::Anthropic, ANTHROPIC_API_KEY_ENV)?),
    ))
}

pub(super) fn execute_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ProviderRuntimeError> {
    let messages = [
        ProviderChatMessage { role: ProviderChatRole::System, content: system_prompt.to_string() },
        ProviderChatMessage { role: ProviderChatRole::User, content: user_prompt.to_string() },
    ];
    execute_chat(client, route, &messages, None)
}

pub(super) fn execute_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    #[derive(Serialize)]
    struct AnthropicChatRequestMessage<'a> {
        role: &'static str,
        content: &'a str,
    }

    #[derive(Serialize)]
    struct AnthropicChatRequest<'a> {
        model: &'a str,
        max_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        system: Option<String>,
        messages: Vec<AnthropicChatRequestMessage<'a>>,
    }

    let endpoint = messages_endpoint(&route.base_url);
    let api_key = route.api_key.as_ref().ok_or(ProviderRuntimeError::MissingApiKey {
        namespace: route.namespace.as_str(),
        env_key: ANTHROPIC_API_KEY_ENV,
    })?;
    let system = super::system_prompt_from_messages(messages);
    let body = AnthropicChatRequest {
        model: &route.model_id,
        max_tokens: max_tokens.unwrap_or(DEFAULT_PROVIDER_CHAT_MAX_TOKENS),
        system,
        messages: messages
            .iter()
            .filter_map(|message| match message.role {
                ProviderChatRole::System => None,
                ProviderChatRole::User => Some(AnthropicChatRequestMessage {
                    role: ANTHROPIC_USER_ROLE,
                    content: message.content.as_str(),
                }),
                ProviderChatRole::Assistant => Some(AnthropicChatRequestMessage {
                    role: ANTHROPIC_ASSISTANT_ROLE,
                    content: message.content.as_str(),
                }),
            })
            .collect(),
    };

    let response = client
        .post(&endpoint)
        .header("x-api-key", api_key)
        .header(ANTHROPIC_VERSION_HEADER, ANTHROPIC_VERSION_VALUE)
        .json(&body)
        .send()
        .map_err(|error| ProviderRuntimeError::Network(error.to_string()))?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().unwrap_or_default();
        return Err(ProviderRuntimeError::Api { status, body });
    }

    let parsed: AnthropicResponse =
        response.json().map_err(|error| ProviderRuntimeError::BadResponse(error.to_string()))?;
    let content = parsed
        .content
        .into_iter()
        .filter(|block| block.kind == ANTHROPIC_TEXT_BLOCK_KIND)
        .filter_map(|block| block.text)
        .collect::<Vec<_>>()
        .join("\n");
    if content.trim().is_empty() {
        return Err(ProviderRuntimeError::BadResponse(
            "provider returned an empty completion".to_string(),
        ));
    }

    Ok(content)
}

pub(super) fn messages_endpoint(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    let has_path = match trimmed.find("://") {
        Some(index) => trimmed[index + 3..].contains('/'),
        None => trimmed.contains('/'),
    };
    let suffix = if has_path { ANTHROPIC_MESSAGES_PATH } else { ANTHROPIC_ROOT_MESSAGES_PATH };
    format!("{trimmed}{suffix}")
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}
