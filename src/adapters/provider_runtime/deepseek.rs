use reqwest::blocking::Client;

use super::{
    DEEPSEEK_API_KEY_ENV, DEEPSEEK_BASE_URL_ENV, DEFAULT_DEEPSEEK_BASE_URL, ProviderChatMessage,
    ProviderNamespace, ProviderRuntimeError, ResolvedProviderRoute, openai_compatible,
};

pub(super) fn resolve_credentials() -> Result<(String, Option<String>), ProviderRuntimeError> {
    openai_compatible::resolve_credentials(
        ProviderNamespace::DeepSeek,
        DEEPSEEK_API_KEY_ENV,
        DEEPSEEK_BASE_URL_ENV,
        DEFAULT_DEEPSEEK_BASE_URL,
        false,
    )
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

#[cfg(test)]
mod tests {
    use reqwest::blocking::Client;

    use super::{ProviderChatMessage, ProviderNamespace, ResolvedProviderRoute, execute_chat};

    #[test]
    fn execute_chat_propagates_network_error_when_endpoint_is_unreachable() {
        unsafe { std::env::set_var("BOUNDLINE_TEST_DISABLE_RETRIES", "1") };
        let client = Client::new();
        let route = ResolvedProviderRoute {
            namespace: ProviderNamespace::DeepSeek,
            model_id: "deepseek-chat".to_string(),
            base_url: "http://127.0.0.1:1".to_string(),
            api_key: Some("sk-test".to_string()),
        };
        let messages: Vec<ProviderChatMessage> = Vec::new();
        let result = execute_chat(&client, &route, &messages, None);
        unsafe { std::env::remove_var("BOUNDLINE_TEST_DISABLE_RETRIES") };
        assert!(result.is_err());
    }
}
