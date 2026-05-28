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

#[cfg(test)]
mod tests {
    use std::env;
    use std::sync::{Mutex, OnceLock};

    use reqwest::blocking::Client;

    use super::{
        COPILOT_GITHUB_TOKEN_ENV, GH_TOKEN_ENV, GITHUB_MODELS_BASE_URL_ENV, GITHUB_MODELS_ORG_ENV,
        GITHUB_MODELS_TOKEN_ENV, GITHUB_TOKEN_ENV, ProviderChatMessage, ProviderNamespace,
        ResolvedProviderRoute, execute_chat, resolve_credentials,
    };

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn resolve_credentials_returns_error_when_all_token_env_vars_are_unset() {
        let _guard = env_lock().lock().ok();
        let saved: Vec<(_, Option<_>)> =
            [GITHUB_MODELS_TOKEN_ENV, GITHUB_TOKEN_ENV, COPILOT_GITHUB_TOKEN_ENV, GH_TOKEN_ENV]
                .iter()
                .map(|k| (*k, env::var(k).ok()))
                .collect();

        for (k, _) in &saved {
            unsafe { env::remove_var(k) };
        }

        let result = resolve_credentials();

        for (k, v) in saved {
            match v {
                Some(val) => unsafe { env::set_var(k, val) },
                None => unsafe { env::remove_var(k) },
            }
        }

        assert!(result.is_err());
    }

    #[test]
    fn resolve_credentials_succeeds_and_returns_token_when_env_var_is_set() {
        let _guard = env_lock().lock().ok();
        let saved = env::var(GITHUB_MODELS_TOKEN_ENV).ok();
        let base_url_saved = env::var(GITHUB_MODELS_BASE_URL_ENV).ok();
        let org_saved = env::var(GITHUB_MODELS_ORG_ENV).ok();

        unsafe {
            env::set_var(GITHUB_MODELS_TOKEN_ENV, "ghm-test-token");
            env::remove_var(GITHUB_MODELS_BASE_URL_ENV);
            env::remove_var(GITHUB_MODELS_ORG_ENV);
        }

        let result = resolve_credentials();

        match saved {
            Some(v) => unsafe { env::set_var(GITHUB_MODELS_TOKEN_ENV, v) },
            None => unsafe { env::remove_var(GITHUB_MODELS_TOKEN_ENV) },
        }
        match base_url_saved {
            Some(v) => unsafe { env::set_var(GITHUB_MODELS_BASE_URL_ENV, v) },
            None => unsafe { env::remove_var(GITHUB_MODELS_BASE_URL_ENV) },
        }
        match org_saved {
            Some(v) => unsafe { env::set_var(GITHUB_MODELS_ORG_ENV, v) },
            None => unsafe { env::remove_var(GITHUB_MODELS_ORG_ENV) },
        }

        let (base_url, api_key) = result.expect("expected Ok credentials");
        assert!(!base_url.is_empty());
        assert_eq!(api_key.as_deref(), Some("ghm-test-token"));
    }

    #[test]
    fn resolve_base_url_embeds_org_name_when_org_env_is_set() {
        let _guard = env_lock().lock().ok();
        let token_saved = env::var(GITHUB_MODELS_TOKEN_ENV).ok();
        let base_url_saved = env::var(GITHUB_MODELS_BASE_URL_ENV).ok();
        let org_saved = env::var(GITHUB_MODELS_ORG_ENV).ok();

        unsafe {
            env::set_var(GITHUB_MODELS_TOKEN_ENV, "ghm-org-token");
            env::remove_var(GITHUB_MODELS_BASE_URL_ENV);
            env::set_var(GITHUB_MODELS_ORG_ENV, "my-org");
        }

        let result = resolve_credentials();

        match token_saved {
            Some(v) => unsafe { env::set_var(GITHUB_MODELS_TOKEN_ENV, v) },
            None => unsafe { env::remove_var(GITHUB_MODELS_TOKEN_ENV) },
        }
        match base_url_saved {
            Some(v) => unsafe { env::set_var(GITHUB_MODELS_BASE_URL_ENV, v) },
            None => unsafe { env::remove_var(GITHUB_MODELS_BASE_URL_ENV) },
        }
        match org_saved {
            Some(v) => unsafe { env::set_var(GITHUB_MODELS_ORG_ENV, v) },
            None => unsafe { env::remove_var(GITHUB_MODELS_ORG_ENV) },
        }

        let (base_url, _) = result.expect("expected Ok credentials");
        assert!(base_url.contains("my-org"), "expected org in base URL, got: {base_url}");
    }

    #[test]
    fn execute_chat_propagates_network_error_when_endpoint_is_unreachable() {
        unsafe { std::env::set_var("BOUNDLINE_TEST_DISABLE_RETRIES", "1") };
        let client = Client::new();
        let route = ResolvedProviderRoute {
            namespace: ProviderNamespace::GitHubModels,
            model_id: "openai/gpt-4.1".to_string(),
            base_url: "http://127.0.0.1:1".to_string(),
            api_key: Some("ghm-test-token".to_string()),
        };
        let messages: Vec<ProviderChatMessage> = Vec::new();
        let result = execute_chat(&client, &route, &messages, None);
        unsafe { std::env::remove_var("BOUNDLINE_TEST_DISABLE_RETRIES") };
        assert!(result.is_err());
    }
}
