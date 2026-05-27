use reqwest::{Url, blocking::Client};
use serde::Deserialize;

use super::{
    COPILOT_ACCEPT_ENCODING_HEADER, COPILOT_ACCEPT_ENCODING_VALUE, COPILOT_API_KEY_ENV,
    COPILOT_EDITOR_PLUGIN_VERSION_HEADER, COPILOT_EDITOR_PLUGIN_VERSION_VALUE,
    COPILOT_EDITOR_VERSION_HEADER, COPILOT_EDITOR_VERSION_VALUE, COPILOT_GITHUB_API_VERSION_HEADER,
    COPILOT_GITHUB_API_VERSION_VALUE, COPILOT_GITHUB_TOKEN_ENV, COPILOT_INTEGRATION_ID_HEADER,
    COPILOT_INTEGRATION_ID_VALUE, COPILOT_OPENAI_ORGANIZATION_HEADER,
    COPILOT_OPENAI_ORGANIZATION_VALUE, COPILOT_TOKEN_ENV_HINT, COPILOT_TOKEN_EXCHANGE_URL,
    COPILOT_USER_AGENT, DEFAULT_COPILOT_BASE_URL, GH_TOKEN_ENV, GITHUB_TOKEN_ENV,
    JSON_ACCEPT_HEADER_VALUE, ProviderChatMessage, ProviderNamespace, ProviderRuntimeError,
    ResolvedProviderRoute, openai_compatible,
};

const COPILOT_REQUEST_HEADERS: [openai_compatible::RequestHeader; 6] = [
    (COPILOT_ACCEPT_ENCODING_HEADER, COPILOT_ACCEPT_ENCODING_VALUE),
    (COPILOT_EDITOR_VERSION_HEADER, COPILOT_EDITOR_VERSION_VALUE),
    (COPILOT_EDITOR_PLUGIN_VERSION_HEADER, COPILOT_EDITOR_PLUGIN_VERSION_VALUE),
    (COPILOT_INTEGRATION_ID_HEADER, COPILOT_INTEGRATION_ID_VALUE),
    ("User-Agent", COPILOT_USER_AGENT),
    (COPILOT_OPENAI_ORGANIZATION_HEADER, COPILOT_OPENAI_ORGANIZATION_VALUE),
];
const COPILOT_PROXY_ENDPOINT_KEY: &str = "proxy-ep=";
const COPILOT_PROXY_HOST_PREFIX: &str = "proxy.";
const COPILOT_API_HOST_PREFIX: &str = "api.";
const HTTP_SCHEME: &str = "http";
const HTTP_SCHEME_PREFIX: &str = "http://";
const HTTPS_SCHEME: &str = "https";
const HTTPS_SCHEME_PREFIX: &str = "https://";
const GITHUB_OAUTH_TOKEN_PREFIX: &str = "gho_";
const GITHUB_CLASSIC_PAT_PREFIX: &str = "ghp_";
const GITHUB_FINE_GRAINED_PAT_PREFIX: &str = "github_pat_";
const COPILOT_PAT_404_HINT: &str = "This token looks like a GitHub personal access token. GitHub Copilot exchange usually requires a GitHub OAuth user token, such as `gh auth token` or a device-login token, on an account with active Copilot access.";
const COPILOT_OAUTH_404_HINT: &str = "This token already looks like a GitHub OAuth user token. Confirm that the authenticated account has GitHub Copilot access and that the token belongs to the same account you expect Boundline to use.";

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    let api_key = super::env_string(COPILOT_GITHUB_TOKEN_ENV)
        .or_else(|| super::env_string(GH_TOKEN_ENV))
        .or_else(|| super::env_string(GITHUB_TOKEN_ENV))
        .or_else(|| super::env_string(COPILOT_API_KEY_ENV));

    match api_key {
        Some(api_key) => Ok((DEFAULT_COPILOT_BASE_URL.to_string(), Some(api_key))),
        None => Err(ProviderRuntimeError::MissingApiKey {
            namespace: ProviderNamespace::Copilot.as_str(),
            env_key: COPILOT_TOKEN_ENV_HINT,
        }),
    }
}

pub(super) fn execute_prompt(
    client: &Client,
    route: &ResolvedProviderRoute,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ProviderRuntimeError> {
    let session_token = resolve_session_token(client, route, COPILOT_TOKEN_EXCHANGE_URL)?;
    let runtime_route = runtime_route_for_session(route, &session_token);
    openai_compatible::execute_prompt(
        client,
        &runtime_route,
        system_prompt,
        user_prompt,
        Some(session_token),
        &COPILOT_REQUEST_HEADERS,
    )
}

pub(super) fn execute_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    execute_chat_with_exchange(client, route, messages, max_tokens, COPILOT_TOKEN_EXCHANGE_URL)
}

pub(super) fn execute_chat_with_exchange(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
    exchange_url: &str,
) -> Result<String, ProviderRuntimeError> {
    let session_token = resolve_session_token(client, route, exchange_url)?;
    let runtime_route = runtime_route_for_session(route, &session_token);
    openai_compatible::execute_chat(
        client,
        &runtime_route,
        messages,
        max_tokens,
        Some(session_token),
        &COPILOT_REQUEST_HEADERS,
    )
}

fn runtime_route_for_session(
    route: &ResolvedProviderRoute,
    session_token: &str,
) -> ResolvedProviderRoute {
    let mut runtime_route = route.clone();
    if route.base_url == DEFAULT_COPILOT_BASE_URL
        && let Some(base_url) = derive_runtime_base_url_from_token(session_token)
    {
        runtime_route.base_url = base_url;
    }
    runtime_route
}

pub(super) fn derive_runtime_base_url_from_token(session_token: &str) -> Option<String> {
    let proxy_endpoint = session_token
        .split(';')
        .find_map(|segment| segment.trim().strip_prefix(COPILOT_PROXY_ENDPOINT_KEY))?
        .trim();
    if proxy_endpoint.is_empty() {
        return None;
    }

    let proxy_host = resolve_proxy_host(proxy_endpoint)?;
    let api_host = proxy_host
        .strip_prefix(COPILOT_PROXY_HOST_PREFIX)
        .map(|suffix| format!("{COPILOT_API_HOST_PREFIX}{suffix}"))
        .unwrap_or(proxy_host);
    Some(format!("{HTTPS_SCHEME}://{api_host}"))
}

pub(super) fn token_exchange_error_hint(github_token: &str, status: u16) -> Option<&'static str> {
    if status != 404 {
        return None;
    }

    let trimmed = github_token.trim();
    if trimmed.starts_with(GITHUB_CLASSIC_PAT_PREFIX)
        || trimmed.starts_with(GITHUB_FINE_GRAINED_PAT_PREFIX)
    {
        return Some(COPILOT_PAT_404_HINT);
    }
    if trimmed.starts_with(GITHUB_OAUTH_TOKEN_PREFIX) {
        return Some(COPILOT_OAUTH_404_HINT);
    }
    None
}

fn resolve_proxy_host(proxy_endpoint: &str) -> Option<String> {
    let url_text = if proxy_endpoint.starts_with(HTTP_SCHEME_PREFIX)
        || proxy_endpoint.starts_with(HTTPS_SCHEME_PREFIX)
    {
        proxy_endpoint.to_string()
    } else {
        format!("{HTTPS_SCHEME}://{proxy_endpoint}")
    };
    let parsed = Url::parse(&url_text).ok()?;
    if parsed.scheme() != HTTP_SCHEME && parsed.scheme() != HTTPS_SCHEME {
        return None;
    }
    parsed.host_str().map(|host| host.to_ascii_lowercase())
}

fn resolve_session_token(
    client: &Client,
    route: &ResolvedProviderRoute,
    exchange_url: &str,
) -> Result<String, ProviderRuntimeError> {
    let github_token = route.api_key.as_deref().ok_or(ProviderRuntimeError::MissingApiKey {
        namespace: ProviderNamespace::Copilot.as_str(),
        env_key: COPILOT_TOKEN_ENV_HINT,
    })?;
    exchange_session_token(client, github_token, exchange_url)
}

fn exchange_session_token(
    client: &Client,
    github_token: &str,
    exchange_url: &str,
) -> Result<String, ProviderRuntimeError> {
    #[derive(Debug, Deserialize)]
    struct CopilotTokenExchangeResponse {
        #[serde(default)]
        token: Option<String>,
    }

    let request_builder = client
        .get(exchange_url)
        .header("Authorization", format!("Bearer {github_token}"))
        .header("Accept", JSON_ACCEPT_HEADER_VALUE)
        .header(COPILOT_ACCEPT_ENCODING_HEADER, COPILOT_ACCEPT_ENCODING_VALUE)
        .header(COPILOT_EDITOR_VERSION_HEADER, COPILOT_EDITOR_VERSION_VALUE)
        .header(COPILOT_EDITOR_PLUGIN_VERSION_HEADER, COPILOT_EDITOR_PLUGIN_VERSION_VALUE)
        .header(COPILOT_INTEGRATION_ID_HEADER, COPILOT_INTEGRATION_ID_VALUE)
        .header(COPILOT_GITHUB_API_VERSION_HEADER, COPILOT_GITHUB_API_VERSION_VALUE)
        .header("User-Agent", COPILOT_USER_AGENT);

    let response = super::execute_with_retry(request_builder, "copilot", "token_exchange")
        .map_err(|error| ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message: format!("GitHub Copilot token exchange request failed: {error}"),
        })?;

    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().unwrap_or_default();
        let mut message = format!("GitHub Copilot token exchange returned {status}: {body}");
        if let Some(hint) = token_exchange_error_hint(github_token, status) {
            message.push(' ');
            message.push_str(hint);
        }
        return Err(ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message,
        });
    }

    let parsed: CopilotTokenExchangeResponse =
        response.json().map_err(|error| ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message: format!("GitHub Copilot token exchange returned invalid JSON: {error}"),
        })?;

    let token = parsed.token.ok_or_else(|| ProviderRuntimeError::CredentialExchange {
        namespace: ProviderNamespace::Copilot.as_str(),
        message: "GitHub Copilot token exchange did not return a session token.".to_string(),
    })?;

    if token.trim().is_empty() {
        return Err(ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message: "GitHub Copilot token exchange returned an empty session token.".to_string(),
        });
    }

    Ok(token)
}
