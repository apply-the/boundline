use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::{Url, blocking::Client};
use serde::Deserialize;

use super::{
    COPILOT_ACCEPT_ENCODING_HEADER, COPILOT_ACCEPT_ENCODING_VALUE, COPILOT_API_KEY_ENV,
    COPILOT_API_URL_ENV, COPILOT_EDITOR_PLUGIN_VERSION_HEADER, COPILOT_EDITOR_PLUGIN_VERSION_VALUE,
    COPILOT_EDITOR_VERSION_HEADER, COPILOT_EDITOR_VERSION_VALUE, COPILOT_GITHUB_API_VERSION_HEADER,
    COPILOT_GITHUB_API_VERSION_VALUE, COPILOT_GITHUB_TOKEN_ENV, COPILOT_INTEGRATION_ID_HEADER,
    COPILOT_INTEGRATION_ID_VALUE, COPILOT_OPENAI_ORGANIZATION_HEADER,
    COPILOT_OPENAI_ORGANIZATION_VALUE, COPILOT_TOKEN_ENV_HINT, COPILOT_TOKEN_EXCHANGE_URL,
    COPILOT_USER_AGENT, DEFAULT_COPILOT_BASE_URL, GH_TOKEN_ENV, GITHUB_COPILOT_API_TOKEN_ENV,
    GITHUB_TOKEN_ENV, JSON_ACCEPT_HEADER_VALUE, ProviderChatMessage, ProviderNamespace,
    ProviderRuntimeError, ResolvedProviderRoute, openai_compatible,
};
use crate::adapters::auth_profile_store;

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
const COPILOT_USER_BOOTSTRAP_URL: &str = "https://api.github.com/copilot_internal/user";
const COPILOT_PAT_403_HINT: &str = "This token looks like a fine-grained or classic personal access token. GitHub Models may accept it while the legacy Copilot token-exchange path may not. Prefer GITHUB_COPILOT_API_TOKEN + COPILOT_API_URL or the /copilot_internal/user bootstrap path.";
const COPILOT_OAUTH_403_HINT: &str = "This token already looks like a GitHub OAuth user token. Confirm that the authenticated account has GitHub Copilot access and that the token belongs to the same account you expect Boundline to use.";
const AUTH_PROFILE_PROVIDER_KEY: &str = "github-copilot";
/// Safety margin in seconds subtracted from token expiry to avoid using expired tokens.
const SESSION_TOKEN_EXPIRY_MARGIN_SECS: u64 = 60;

#[derive(Debug)]
struct CopilotRuntimeAuth {
    base_url: String,
    bearer_token: String,
}

/// Cached session token from the Copilot token exchange endpoint.
struct CachedSessionToken {
    token: String,
    base_url: String,
    expires_at_secs: u64,
}

static SESSION_TOKEN_CACHE: Mutex<Option<CachedSessionToken>> = Mutex::new(None);

fn current_epoch_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn cached_session_auth() -> Option<CopilotRuntimeAuth> {
    let guard = SESSION_TOKEN_CACHE.lock().ok()?;
    let cached = guard.as_ref()?;
    if current_epoch_secs() >= cached.expires_at_secs {
        return None;
    }
    Some(CopilotRuntimeAuth {
        base_url: cached.base_url.clone(),
        bearer_token: cached.token.clone(),
    })
}

fn store_session_auth(token: &str, base_url: &str) {
    // Copilot session tokens encode expiry as `exp=<unix_ts>;` segments.
    let expires_at = token
        .split(';')
        .find_map(|seg| seg.trim().strip_prefix("exp="))
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or_else(|| current_epoch_secs() + 1800);

    let effective_expiry = expires_at.saturating_sub(SESSION_TOKEN_EXPIRY_MARGIN_SECS);
    if let Ok(mut guard) = SESSION_TOKEN_CACHE.lock() {
        *guard = Some(CachedSessionToken {
            token: token.to_string(),
            base_url: base_url.to_string(),
            expires_at_secs: effective_expiry,
        });
    }
}

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    if let Some(api_token) = direct_api_token() {
        let base_url = super::env_string(COPILOT_API_URL_ENV)
            .unwrap_or_else(|| DEFAULT_COPILOT_BASE_URL.to_string());
        return Ok((base_url, Some(api_token)));
    }

    // Explicit Copilot-specific env var takes precedence over stored profiles.
    if let Some(copilot_token) = super::env_string(COPILOT_GITHUB_TOKEN_ENV) {
        return Ok((DEFAULT_COPILOT_BASE_URL.to_string(), Some(copilot_token)));
    }

    // Stored auth profile from device-flow login is preferred over generic
    // ambient GitHub tokens (GH_TOKEN, GITHUB_TOKEN).
    if let Some(profile_token) = auth_profile_token() {
        return Ok((DEFAULT_COPILOT_BASE_URL.to_string(), Some(profile_token)));
    }

    let api_key = super::env_string(GH_TOKEN_ENV)
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
    let auth = resolve_runtime_auth(
        client,
        route,
        COPILOT_TOKEN_EXCHANGE_URL,
        COPILOT_USER_BOOTSTRAP_URL,
    )?;
    let runtime_route = runtime_route_for_base_url(route, &auth.base_url);
    openai_compatible::execute_prompt(
        client,
        &runtime_route,
        system_prompt,
        user_prompt,
        Some(auth.bearer_token),
        &COPILOT_REQUEST_HEADERS,
    )
}

pub(super) fn execute_chat(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
) -> Result<String, ProviderRuntimeError> {
    execute_chat_with_exchange_and_bootstrap(
        client,
        route,
        messages,
        max_tokens,
        COPILOT_TOKEN_EXCHANGE_URL,
        COPILOT_USER_BOOTSTRAP_URL,
    )
}

#[allow(dead_code)]
pub(super) fn execute_chat_with_exchange(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
    exchange_url: &str,
) -> Result<String, ProviderRuntimeError> {
    execute_chat_with_exchange_and_bootstrap(
        client,
        route,
        messages,
        max_tokens,
        exchange_url,
        COPILOT_USER_BOOTSTRAP_URL,
    )
}

pub(super) fn execute_chat_with_exchange_and_bootstrap(
    client: &Client,
    route: &ResolvedProviderRoute,
    messages: &[ProviderChatMessage],
    max_tokens: Option<u32>,
    exchange_url: &str,
    user_bootstrap_url: &str,
) -> Result<String, ProviderRuntimeError> {
    let auth = resolve_runtime_auth(client, route, exchange_url, user_bootstrap_url)?;
    let runtime_route = runtime_route_for_base_url(route, &auth.base_url);
    openai_compatible::execute_chat(
        client,
        &runtime_route,
        messages,
        max_tokens,
        Some(auth.bearer_token),
        &COPILOT_REQUEST_HEADERS,
    )
}

fn runtime_route_for_base_url(
    route: &ResolvedProviderRoute,
    base_url: &str,
) -> ResolvedProviderRoute {
    let mut runtime_route = route.clone();
    runtime_route.base_url = base_url.to_string();
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
    if status != 403 && status != 404 {
        return None;
    }

    let trimmed = github_token.trim();
    if trimmed.starts_with(GITHUB_CLASSIC_PAT_PREFIX)
        || trimmed.starts_with(GITHUB_FINE_GRAINED_PAT_PREFIX)
    {
        return Some(COPILOT_PAT_403_HINT);
    }
    if trimmed.starts_with(GITHUB_OAUTH_TOKEN_PREFIX) {
        return Some(COPILOT_OAUTH_403_HINT);
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

fn resolve_runtime_auth(
    client: &Client,
    route: &ResolvedProviderRoute,
    exchange_url: &str,
    user_bootstrap_url: &str,
) -> Result<CopilotRuntimeAuth, ProviderRuntimeError> {
    if direct_api_token().is_some() {
        let bearer_token = route.api_key.clone().ok_or(ProviderRuntimeError::MissingApiKey {
            namespace: ProviderNamespace::Copilot.as_str(),
            env_key: COPILOT_TOKEN_ENV_HINT,
        })?;
        return Ok(CopilotRuntimeAuth { base_url: route.base_url.clone(), bearer_token });
    }

    // Return cached session token if still valid.
    if let Some(cached) = cached_session_auth() {
        return Ok(cached);
    }

    let github_token = route.api_key.as_deref().ok_or(ProviderRuntimeError::MissingApiKey {
        namespace: ProviderNamespace::Copilot.as_str(),
        env_key: COPILOT_TOKEN_ENV_HINT,
    })?;

    // PATs cannot authenticate to Copilot endpoints; reroute transparently
    // to the GitHub Models inference API which accepts them as bearer tokens.
    if let Some(rerouted) = reroute_pat_to_github_models(github_token) {
        return Ok(rerouted);
    }

    let bootstrap_result =
        fetch_runtime_base_url_from_user(client, github_token, user_bootstrap_url);
    match bootstrap_result {
        Ok(base_url) => Ok(CopilotRuntimeAuth { base_url, bearer_token: github_token.to_string() }),
        Err(user_error) => {
            let session_token = match exchange_session_token(client, github_token, exchange_url) {
                Ok(session_token) => session_token,
                Err(exchange_error) => {
                    return Err(ProviderRuntimeError::CredentialExchange {
                        namespace: ProviderNamespace::Copilot.as_str(),
                        message: format!(
                            "Copilot user bootstrap failed first ({user_error}); token exchange fallback also failed ({exchange_error})"
                        ),
                    });
                }
            };
            let base_url = derive_runtime_base_url_from_token(&session_token)
                .unwrap_or_else(|| route.base_url.clone());
            store_session_auth(&session_token, &base_url);
            Ok(CopilotRuntimeAuth { base_url, bearer_token: session_token })
        }
    }
}

fn direct_api_token() -> Option<String> {
    super::env_string(GITHUB_COPILOT_API_TOKEN_ENV)
}

fn auth_profile_token() -> Option<String> {
    auth_profile_store::load_auth_profiles()
        .ok()
        .and_then(|store| store.get_token(AUTH_PROFILE_PROVIDER_KEY).map(str::to_string))
}

fn is_personal_access_token(token: &str) -> bool {
    let trimmed = token.trim();
    trimmed.starts_with(GITHUB_CLASSIC_PAT_PREFIX)
        || trimmed.starts_with(GITHUB_FINE_GRAINED_PAT_PREFIX)
}

/// When the available token is a PAT (ghp_ / github_pat_), reroute to the
/// GitHub Models inference endpoint which accepts PATs as direct bearer tokens.
fn reroute_pat_to_github_models(token: &str) -> Option<CopilotRuntimeAuth> {
    if !is_personal_access_token(token) {
        return None;
    }
    tracing::info!(
        "Token is a personal access token; rerouting to GitHub Models inference endpoint"
    );
    Some(CopilotRuntimeAuth {
        base_url: super::DEFAULT_GITHUB_MODELS_BASE_URL.to_string(),
        bearer_token: token.to_string(),
    })
}

fn fetch_runtime_base_url_from_user(
    client: &Client,
    github_token: &str,
    user_bootstrap_url: &str,
) -> Result<String, ProviderRuntimeError> {
    #[derive(Debug, Deserialize)]
    struct CopilotUserResponse {
        #[serde(default)]
        endpoints: Option<CopilotUserEndpoints>,
    }

    #[derive(Debug, Deserialize)]
    struct CopilotUserEndpoints {
        #[serde(default)]
        api: Option<String>,
    }

    let request_builder = client
        .get(user_bootstrap_url)
        .header("Authorization", format!("Bearer {github_token}"))
        .header("Accept", JSON_ACCEPT_HEADER_VALUE)
        .header(COPILOT_ACCEPT_ENCODING_HEADER, COPILOT_ACCEPT_ENCODING_VALUE)
        .header(COPILOT_EDITOR_VERSION_HEADER, COPILOT_EDITOR_VERSION_VALUE)
        .header(COPILOT_EDITOR_PLUGIN_VERSION_HEADER, COPILOT_EDITOR_PLUGIN_VERSION_VALUE)
        .header(COPILOT_INTEGRATION_ID_HEADER, COPILOT_INTEGRATION_ID_VALUE)
        .header(COPILOT_GITHUB_API_VERSION_HEADER, COPILOT_GITHUB_API_VERSION_VALUE)
        .header("User-Agent", COPILOT_USER_AGENT);

    let response = super::execute_with_retry(request_builder, "copilot", "user_bootstrap")
        .map_err(|error| ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message: format!("GitHub Copilot user bootstrap request failed: {error}"),
        })?;

    let status = response.status().as_u16();
    if status >= 400 {
        let body = response.text().unwrap_or_default();
        let mut message = format!("GitHub Copilot user bootstrap returned {status}: {body}");
        if let Some(hint) = token_exchange_error_hint(github_token, status) {
            message.push(' ');
            message.push_str(hint);
        }
        return Err(ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message,
        });
    }

    let parsed: CopilotUserResponse =
        response.json().map_err(|error| ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message: format!("GitHub Copilot user bootstrap returned invalid JSON: {error}"),
        })?;

    let endpoint = parsed
        .endpoints
        .and_then(|endpoints| endpoints.api)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProviderRuntimeError::CredentialExchange {
            namespace: ProviderNamespace::Copilot.as_str(),
            message: "GitHub Copilot user bootstrap did not return endpoints.api.".to_string(),
        })?;
    Ok(endpoint)
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
