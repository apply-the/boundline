use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const AUTH_PROFILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfileStore {
    pub version: u32,
    pub profiles: BTreeMap<String, ProviderAuthEntry>,
}

impl AuthProfileStore {
    pub fn empty() -> Self {
        Self { version: AUTH_PROFILE_VERSION, profiles: BTreeMap::new() }
    }

    pub fn get_token(&self, provider_id: &str) -> Option<&str> {
        self.profiles.get(provider_id).map(ProviderAuthEntry::token_value)
    }

    pub fn set_token(&mut self, provider_id: &str, token: String, obtained_at: String) {
        self.profiles.insert(
            provider_id.to_string(),
            ProviderAuthEntry::Token { provider: provider_id.to_string(), token, obtained_at },
        );
    }

    pub fn remove_provider(&mut self, provider_id: &str) -> bool {
        self.profiles.remove(provider_id).is_some()
    }

    pub fn list_providers(&self) -> Vec<&str> {
        self.profiles.keys().map(String::as_str).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderAuthEntry {
    Token { provider: String, token: String, obtained_at: String },
    ApiKey { provider: String, key: String },
}

impl ProviderAuthEntry {
    pub fn token_value(&self) -> &str {
        match self {
            Self::Token { token, .. } => token,
            Self::ApiKey { key, .. } => key,
        }
    }

    pub fn provider_id(&self) -> &str {
        match self {
            Self::Token { provider, .. } | Self::ApiKey { provider, .. } => provider,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_store_has_no_providers() {
        let store = AuthProfileStore::empty();
        assert_eq!(store.version, AUTH_PROFILE_VERSION);
        assert!(store.profiles.is_empty());
        assert!(store.list_providers().is_empty());
    }

    #[test]
    fn set_and_get_token() {
        let mut store = AuthProfileStore::empty();
        store.set_token(
            "github-copilot",
            "gho_abc123".to_string(),
            "2026-05-28T12:00:00Z".to_string(),
        );

        assert_eq!(store.get_token("github-copilot"), Some("gho_abc123"));
        assert_eq!(store.list_providers(), vec!["github-copilot"]);
    }

    #[test]
    fn remove_provider_returns_true_when_present() {
        let mut store = AuthProfileStore::empty();
        store.set_token(
            "github-copilot",
            "gho_abc123".to_string(),
            "2026-05-28T12:00:00Z".to_string(),
        );

        assert!(store.remove_provider("github-copilot"));
        assert!(store.get_token("github-copilot").is_none());
    }

    #[test]
    fn remove_provider_returns_false_when_absent() {
        let mut store = AuthProfileStore::empty();
        assert!(!store.remove_provider("github-copilot"));
    }

    #[test]
    fn serialization_roundtrip() {
        let mut store = AuthProfileStore::empty();
        store.set_token(
            "github-copilot",
            "gho_token".to_string(),
            "2026-05-28T12:00:00Z".to_string(),
        );

        let json = serde_json::to_string_pretty(&store).expect("serialize");
        let restored: AuthProfileStore = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.version, store.version);
        assert_eq!(restored.get_token("github-copilot"), Some("gho_token"));
    }
}
