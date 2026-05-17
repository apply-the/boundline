use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::configuration::{ConfigFile, RoutingConfig};

const LOCAL_CONFIG_RELATIVE: &str = ".boundline/config.toml";

#[derive(Debug, Clone)]
pub struct FileConfigStore {
    workspace: PathBuf,
}

impl FileConfigStore {
    pub fn for_workspace(workspace: &Path) -> Self {
        Self { workspace: workspace.to_path_buf() }
    }

    pub fn local_config_path(&self) -> PathBuf {
        self.workspace.join(LOCAL_CONFIG_RELATIVE)
    }

    pub fn global_config_path() -> PathBuf {
        if let Some(xdg_home) = env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg_home).join("boundline/config.toml");
        }

        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        home.join(".config/boundline/config.toml")
    }

    pub fn load_local(&self) -> Result<Option<ConfigFile>, ConfigStoreError> {
        load_from_path(&self.local_config_path())
    }

    pub fn save_local(&self, config: &ConfigFile) -> Result<PathBuf, ConfigStoreError> {
        save_to_path(&self.local_config_path(), config)
    }

    pub fn load_global() -> Result<Option<ConfigFile>, ConfigStoreError> {
        load_from_path(&Self::global_config_path())
    }

    pub fn save_global(config: &ConfigFile) -> Result<PathBuf, ConfigStoreError> {
        save_to_path(&Self::global_config_path(), config)
    }

    pub fn local_routing(&self) -> Result<Option<RoutingConfig>, ConfigStoreError> {
        Ok(self.load_local()?.map(|cfg| cfg.routing))
    }

    pub fn global_routing() -> Result<Option<RoutingConfig>, ConfigStoreError> {
        Ok(Self::load_global()?.map(|cfg| cfg.routing))
    }
}

fn load_from_path(path: &Path) -> Result<Option<ConfigFile>, ConfigStoreError> {
    if !path.is_file() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)
        .map_err(|source| ConfigStoreError::Read { path: path.to_path_buf(), source })?;

    let parsed = toml::from_str::<ConfigFile>(&contents)
        .map_err(|source| ConfigStoreError::Parse { path: path.to_path_buf(), source })?;

    parsed.routing.validate().map_err(|source| ConfigStoreError::InvalidConfig {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;

    Ok(Some(parsed))
}

fn save_to_path(path: &Path, config: &ConfigFile) -> Result<PathBuf, ConfigStoreError> {
    config.routing.validate().map_err(|source| ConfigStoreError::InvalidConfig {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;

    let encoded = toml::to_string_pretty(config)
        .map_err(|source| ConfigStoreError::Serialize { path: path.to_path_buf(), source })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| ConfigStoreError::Write { path: parent.to_path_buf(), source })?;
    }

    fs::write(path, encoded)
        .map_err(|source| ConfigStoreError::Write { path: path.to_path_buf(), source })?;

    Ok(path.to_path_buf())
}

#[derive(Debug, Error)]
pub enum ConfigStoreError {
    #[error("failed to read config at {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("failed to parse config at {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
    #[error("failed to serialize config at {path}: {source}")]
    Serialize { path: PathBuf, source: toml::ser::Error },
    #[error("failed to write config at {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
    #[error("invalid config at {path}: {message}")]
    InvalidConfig { path: PathBuf, message: String },
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use uuid::Uuid;

    use super::{ConfigStoreError, FileConfigStore};
    use crate::domain::configuration::{ConfigFile, ModelRoute, RoutingConfig, RuntimeKind};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvRestore<'a> {
        old_xdg: Option<OsString>,
        old_home: Option<OsString>,
        _lock: MutexGuard<'a, ()>,
    }

    impl Drop for EnvRestore<'_> {
        fn drop(&mut self) {
            unsafe {
                match &self.old_xdg {
                    Some(value) => env::set_var("XDG_CONFIG_HOME", value),
                    None => env::remove_var("XDG_CONFIG_HOME"),
                }
                match &self.old_home {
                    Some(value) => env::set_var("HOME", value),
                    None => env::remove_var("HOME"),
                }
            }
        }
    }

    fn with_config_env<T>(
        xdg_home: Option<&Path>,
        home: Option<&Path>,
        action: impl FnOnce() -> T,
    ) -> T {
        let lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let restore = EnvRestore {
            old_xdg: env::var_os("XDG_CONFIG_HOME"),
            old_home: env::var_os("HOME"),
            _lock: lock,
        };

        unsafe {
            match xdg_home {
                Some(path) => env::set_var("XDG_CONFIG_HOME", path),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
            match home {
                Some(path) => env::set_var("HOME", path),
                None => env::remove_var("HOME"),
            }
        }

        let result = action();
        drop(restore);
        result
    }

    fn valid_config() -> ConfigFile {
        let mut cfg = ConfigFile::default();
        cfg.routing.planning =
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5-codex".to_string() });
        cfg
    }

    #[test]
    fn local_round_trip_works() {
        let workspace = std::env::temp_dir().join(format!("boundline-config-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();

        let store = FileConfigStore::for_workspace(&workspace);
        let cfg = valid_config();

        let path = store.save_local(&cfg).unwrap();
        assert!(path.ends_with(".boundline/config.toml"));

        let loaded = store.load_local().unwrap().unwrap();
        assert_eq!(loaded.routing.planning.unwrap().model, "gpt-5-codex");
    }

    #[test]
    fn local_routing_returns_none_when_missing() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-config-missing-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        let store = FileConfigStore::for_workspace(&workspace);
        assert!(store.local_routing().unwrap().is_none());
    }

    #[test]
    fn invalid_routing_is_rejected_before_write() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-config-invalid-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();

        let store = FileConfigStore::for_workspace(&workspace);
        let cfg = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                planning: Some(ModelRoute { runtime: RuntimeKind::Claude, model: " ".to_string() }),
                ..RoutingConfig::default()
            },
            canon: None,
        };

        let error = store.save_local(&cfg).unwrap_err();
        assert!(error.to_string().contains("invalid config"));
    }

    #[test]
    fn global_config_path_prefers_xdg_config_home() {
        let xdg_home = std::env::temp_dir().join(format!("boundline-xdg-{}", Uuid::new_v4()));
        let home = std::env::temp_dir().join(format!("boundline-home-{}", Uuid::new_v4()));

        with_config_env(Some(&xdg_home), Some(&home), || {
            assert_eq!(
                FileConfigStore::global_config_path(),
                xdg_home.join("boundline/config.toml")
            );
        });
    }

    #[test]
    fn malformed_global_config_reports_parse_error() {
        let xdg_home =
            std::env::temp_dir().join(format!("boundline-global-parse-{}", Uuid::new_v4()));
        let path = xdg_home.join("boundline/config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "not = [valid toml").unwrap();

        with_config_env(Some(&xdg_home), None, || {
            let error = FileConfigStore::load_global().unwrap_err();
            assert!(matches!(error, ConfigStoreError::Parse { .. }));
        });
    }

    #[test]
    fn invalid_global_config_reports_validation_error() {
        let xdg_home =
            std::env::temp_dir().join(format!("boundline-global-invalid-{}", Uuid::new_v4()));
        let path = xdg_home.join("boundline/config.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let cfg = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                planning: Some(ModelRoute { runtime: RuntimeKind::Claude, model: " ".to_string() }),
                ..RoutingConfig::default()
            },
            canon: None,
        };
        fs::write(&path, toml::to_string_pretty(&cfg).unwrap()).unwrap();

        with_config_env(Some(&xdg_home), None, || {
            let error = FileConfigStore::load_global().unwrap_err();
            assert!(matches!(error, ConfigStoreError::InvalidConfig { .. }));
        });
    }

    #[test]
    fn global_routing_reads_saved_xdg_config() {
        let xdg_home =
            std::env::temp_dir().join(format!("boundline-global-routing-{}", Uuid::new_v4()));

        with_config_env(Some(&xdg_home), None, || {
            FileConfigStore::save_global(&valid_config()).unwrap();
            let routing = FileConfigStore::global_routing().unwrap().unwrap();
            assert_eq!(routing.planning.unwrap().model, "gpt-5-codex");
        });
    }

    #[test]
    fn with_config_env_none_xdg_home_covers_remove_var_branch() {
        // Passing xdg_home=None exercises the None arm inside with_config_env
        // (line 163: None => env::remove_var("XDG_CONFIG_HOME"))
        // and the Drop impl's None arm for old_xdg when XDG_CONFIG_HOME was
        // not previously set.
        let home =
            std::env::temp_dir().join(format!("boundline-config-none-xdg-{}", Uuid::new_v4()));
        with_config_env(None, Some(home.as_path()), || {
            // When xdg_home is None, XDG_CONFIG_HOME is removed; the global
            // config path falls back to HOME-based resolution.
            assert!(
                FileConfigStore::global_config_path()
                    .to_string_lossy()
                    .contains(".config/boundline"),
                "expected home-based path"
            );
        });
    }

    #[test]
    fn save_local_reports_parent_write_error() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-config-write-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join(".boundline"), "not a directory").unwrap();

        let store = FileConfigStore::for_workspace(&workspace);
        let error = store.save_local(&valid_config()).unwrap_err();
        assert!(matches!(error, ConfigStoreError::Write { .. }));
    }
}
