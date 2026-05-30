use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use super::{
    DEFAULT_GEMINI_BASE_URL, GEMINI_API_KEY_ENV, GEMINI_MODEL_ROLE, GEMINI_RESPONSE_MIME_TYPE,
    GEMINI_USER_ROLE, ProviderChatMessage, ProviderChatRole, ProviderNamespace,
    ProviderRuntimeError, ResolvedProviderRoute,
};

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    Ok((
        DEFAULT_GEMINI_BASE_URL.to_string(),
        Some(super::required_env(ProviderNamespace::Gemini, GEMINI_API_KEY_ENV)?),
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
    struct GeminiChatPart<'a> {
        text: &'a str,
    }

    #[derive(Serialize)]
    struct GeminiChatContent<'a> {
        role: &'static str,
        parts: Vec<GeminiChatPart<'a>>,
    }

    #[derive(Serialize)]
    struct GeminiSystemInstruction<'a> {
        parts: Vec<GeminiChatPart<'a>>,
    }

    #[derive(Serialize)]
    struct GeminiGenerationConfig {
        response_mime_type: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_output_tokens: Option<u32>,
    }

    #[derive(Serialize)]
    struct GeminiChatRequest<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        system_instruction: Option<GeminiSystemInstruction<'a>>,
        contents: Vec<GeminiChatContent<'a>>,
        generation_config: GeminiGenerationConfig,
    }

    let api_key = route.api_key.as_ref().ok_or(ProviderRuntimeError::MissingApiKey {
        namespace: route.namespace.as_str(),
        env_key: GEMINI_API_KEY_ENV,
    })?;
    let endpoint = format!(
        "{}/models/{}:generateContent",
        route.base_url.trim_end_matches('/'),
        route.model_id
    );
    let system_prompt = super::system_prompt_from_messages(messages);
    let body = GeminiChatRequest {
        system_instruction: system_prompt.as_deref().map(|content| GeminiSystemInstruction {
            parts: vec![GeminiChatPart { text: content }],
        }),
        contents: messages
            .iter()
            .filter_map(|message| match message.role {
                ProviderChatRole::System => None,
                ProviderChatRole::User => Some(GeminiChatContent {
                    role: GEMINI_USER_ROLE,
                    parts: vec![GeminiChatPart { text: message.content.as_str() }],
                }),
                ProviderChatRole::Assistant => Some(GeminiChatContent {
                    role: GEMINI_MODEL_ROLE,
                    parts: vec![GeminiChatPart { text: message.content.as_str() }],
                }),
            })
            .collect(),
        generation_config: GeminiGenerationConfig {
            response_mime_type: GEMINI_RESPONSE_MIME_TYPE,
            max_output_tokens: max_tokens,
        },
    };

    let request_builder = client.post(&endpoint).query(&[("key", api_key)]).json(&body);
    let response = super::execute_with_retry(request_builder, "gemini", &route.model_id)?;
    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().unwrap_or_default();
        return Err(ProviderRuntimeError::Api { status, body });
    }

    let parsed: GeminiResponse =
        response.json().map_err(|error| ProviderRuntimeError::BadResponse(error.to_string()))?;
    let content = parsed
        .candidates
        .into_iter()
        .find_map(|candidate| candidate.content)
        .and_then(|content| {
            let parts = content.parts;
            let rendered =
                parts.into_iter().filter_map(|part| part.text).collect::<Vec<_>>().join("\n");
            (!rendered.trim().is_empty()).then_some(rendered)
        })
        .ok_or_else(|| {
            ProviderRuntimeError::BadResponse("provider returned an empty completion".to_string())
        })?;

    Ok(content)
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    #[serde(default)]
    content: Option<GeminiContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    #[serde(default)]
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    #[serde(default)]
    text: Option<String>,
}
