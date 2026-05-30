use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::config_store::FileConfigStore;
use crate::domain::auth_profile::AuthProfileStore;

const AUTH_PROFILES_FILE_NAME: &str = "auth-profiles.json";

#[derive(Debug, Error)]
pub enum AuthProfileStoreError {
    #[error("failed to read auth profiles at {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("failed to parse auth profiles at {path}: {source}")]
    Parse { path: PathBuf, source: serde_json::Error },
    #[error("failed to serialize auth profiles: {source}")]
    Serialize { source: serde_json::Error },
    #[error("failed to write auth profiles at {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
}

pub fn auth_profiles_path() -> PathBuf {
    FileConfigStore::global_config_dir().join(AUTH_PROFILES_FILE_NAME)
}

pub fn load_auth_profiles() -> Result<AuthProfileStore, AuthProfileStoreError> {
    load_from(auth_profiles_path().as_path())
}

pub fn save_auth_profiles(store: &AuthProfileStore) -> Result<PathBuf, AuthProfileStoreError> {
    save_to(auth_profiles_path().as_path(), store)
}

fn load_from(path: &Path) -> Result<AuthProfileStore, AuthProfileStoreError> {
    if !path.is_file() {
        return Ok(AuthProfileStore::empty());
    }

    let contents = fs::read_to_string(path)
        .map_err(|source| AuthProfileStoreError::Read { path: path.to_path_buf(), source })?;

    serde_json::from_str::<AuthProfileStore>(&contents)
        .map_err(|source| AuthProfileStoreError::Parse { path: path.to_path_buf(), source })
}

fn save_to(path: &Path, store: &AuthProfileStore) -> Result<PathBuf, AuthProfileStoreError> {
    let encoded = serde_json::to_string_pretty(store)
        .map_err(|source| AuthProfileStoreError::Serialize { source })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AuthProfileStoreError::Write {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &encoded)
        .map_err(|source| AuthProfileStoreError::Write { path: tmp_path.clone(), source })?;

    fs::rename(&tmp_path, path)
        .map_err(|source| AuthProfileStoreError::Write { path: path.to_path_buf(), source })?;

    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_returns_empty_when_file_missing() {
        let dir = std::env::temp_dir().join("boundline-auth-test-missing");
        let path = dir.join(AUTH_PROFILES_FILE_NAME);
        let _ = fs::remove_file(&path);

        let result = load_from(&path);
        assert!(result.is_ok());
        let store = result.unwrap_or_else(|_| AuthProfileStore::empty());
        assert!(store.profiles.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("boundline-auth-test-roundtrip");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(AUTH_PROFILES_FILE_NAME);

        let mut store = AuthProfileStore::empty();
        store.set_token(
            "github-copilot",
            "gho_test123".to_string(),
            "2026-05-28T12:00:00Z".to_string(),
        );

        let save_result = save_to(&path, &store);
        assert!(save_result.is_ok());

        let loaded = load_from(&path);
        assert!(loaded.is_ok());
        let restored = loaded.unwrap_or_else(|_| AuthProfileStore::empty());
        assert_eq!(restored.get_token("github-copilot"), Some("gho_test123"));

        let _ = fs::remove_dir_all(&dir);
    }
}
