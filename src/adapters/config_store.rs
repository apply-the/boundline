use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::configuration::{
    ConfigFile, PersistedAdapterConfiguration, PersistedCapabilityProviderConfiguration,
    RoutingConfig,
};

const GLOBAL_CONFIG_DIR_NAME: &str = "boundline";
const GLOBAL_CONFIG_FILE_NAME: &str = "config.toml";
const LOCAL_CONFIG_RELATIVE: &str = ".boundline/config.toml";
const GLOBAL_ENV_FILE_NAME: &str = "providers.env";
const GLOBAL_ENV_TEMPLATE_FILE_NAME: &str = "providers.env.template";
const LOCAL_ENV_FILE_NAME: &str = ".env";
const LOCAL_ENV_LOCAL_FILE_NAME: &str = ".env.local";
const LOCAL_ENV_TEMPLATE_FILE_NAME: &str = ".env.template";

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

    pub fn local_env_path(&self) -> PathBuf {
        self.workspace.join(LOCAL_ENV_FILE_NAME)
    }

    pub fn local_env_local_path(&self) -> PathBuf {
        self.workspace.join(LOCAL_ENV_LOCAL_FILE_NAME)
    }

    pub fn local_env_template_path(&self) -> PathBuf {
        self.workspace.join(LOCAL_ENV_TEMPLATE_FILE_NAME)
    }

    pub fn global_config_dir() -> PathBuf {
        if let Some(xdg_home) = env::var_os("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg_home).join(GLOBAL_CONFIG_DIR_NAME);
        }

        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        home.join(".config").join(GLOBAL_CONFIG_DIR_NAME)
    }

    pub fn global_config_path() -> PathBuf {
        Self::global_config_dir().join(GLOBAL_CONFIG_FILE_NAME)
    }

    pub fn global_env_path() -> PathBuf {
        Self::global_config_dir().join(GLOBAL_ENV_FILE_NAME)
    }

    pub fn global_env_template_path() -> PathBuf {
        Self::global_config_dir().join(GLOBAL_ENV_TEMPLATE_FILE_NAME)
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

    pub fn local_adapter(&self) -> Result<Option<PersistedAdapterConfiguration>, ConfigStoreError> {
        Ok(self.load_local()?.and_then(|cfg| cfg.adapter))
    }

    pub fn global_routing() -> Result<Option<RoutingConfig>, ConfigStoreError> {
        Ok(Self::load_global()?.map(|cfg| cfg.routing))
    }

    pub fn global_adapter() -> Result<Option<PersistedAdapterConfiguration>, ConfigStoreError> {
        Ok(Self::load_global()?.and_then(|cfg| cfg.adapter))
    }

    pub fn local_capability_provider(
        &self,
    ) -> Result<Option<PersistedCapabilityProviderConfiguration>, ConfigStoreError> {
        Ok(self.load_local()?.and_then(|cfg| cfg.capability_provider))
    }

    pub fn global_capability_provider()
    -> Result<Option<PersistedCapabilityProviderConfiguration>, ConfigStoreError> {
        Ok(Self::load_global()?.and_then(|cfg| cfg.capability_provider))
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
    use std::sync::{Mutex, MutexGuard};

    use uuid::Uuid;

    use super::{ConfigStoreError, FileConfigStore};
    use crate::domain::capability_provider::{
        CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
        CapabilityProviderRegistration, CapabilityProviderRegistrationSource,
        CommandProviderTransport, ProviderTransportDescriptor,
    };
    use crate::domain::configuration::{
        ConfigFile, ModelRoute, PersistedAdapterConfiguration,
        PersistedCapabilityProviderConfiguration, RoutingConfig, RuntimeKind,
    };
    use crate::domain::framework_adapter::{
        AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
        AdapterSelectionMode,
    };

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
        let lock = super::super::SHARED_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() });
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
        assert_eq!(loaded.routing.planning.unwrap().model, "o4-mini");
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
            adapter: None,
            capability_provider: None,
        };

        let error = store.save_local(&cfg).unwrap_err();
        assert!(error.to_string().contains("invalid config"));
    }

    #[test]
    fn global_config_path_prefers_xdg_config_home() {
        let xdg_home = std::env::temp_dir().join(format!("boundline-xdg-{}", Uuid::new_v4()));
        let home = std::env::temp_dir().join(format!("boundline-home-{}", Uuid::new_v4()));

        with_config_env(Some(&xdg_home), Some(&home), || {
            assert_eq!(FileConfigStore::global_config_dir(), xdg_home.join("boundline"));
            assert_eq!(
                FileConfigStore::global_config_path(),
                xdg_home.join("boundline/config.toml")
            );
            assert_eq!(
                FileConfigStore::global_env_path(),
                xdg_home.join("boundline/providers.env")
            );
            assert_eq!(
                FileConfigStore::global_env_template_path(),
                xdg_home.join("boundline/providers.env.template")
            );
        });
    }

    #[test]
    fn global_provider_and_adapter_accessors_round_trip() {
        let xdg_home =
            std::env::temp_dir().join(format!("boundline-global-provider-{}", Uuid::new_v4()));
        let home =
            std::env::temp_dir().join(format!("boundline-global-provider-home-{}", Uuid::new_v4()));

        with_config_env(Some(&xdg_home), Some(&home), || {
            let config = ConfigFile {
                version: 1,
                routing: valid_config().routing,
                canon: None,
                adapter: Some(sample_adapter_configuration()),
                capability_provider: Some(sample_capability_provider_configuration()),
            };

            let saved_path = FileConfigStore::save_global(&config)
                .map_err(|error| error.to_string())
                .expect("global config should save");
            let adapter = FileConfigStore::global_adapter()
                .map_err(|error| error.to_string())
                .expect("global adapter should load");
            let provider = FileConfigStore::global_capability_provider()
                .map_err(|error| error.to_string())
                .expect("global provider should load");

            assert_eq!(saved_path, FileConfigStore::global_config_path());
            assert_eq!(adapter, config.adapter);
            assert_eq!(provider, config.capability_provider);
        });
    }

    #[test]
    fn local_env_paths_resolve_from_workspace_root() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-config-env-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();

        let store = FileConfigStore::for_workspace(&workspace);
        assert_eq!(store.local_env_path(), workspace.join(".env"));
        assert_eq!(store.local_env_local_path(), workspace.join(".env.local"));
        assert_eq!(store.local_env_template_path(), workspace.join(".env.template"));
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

    fn sample_adapter_configuration() -> PersistedAdapterConfiguration {
        PersistedAdapterConfiguration {
            selection: crate::domain::configuration::AdapterSelectionRecord {
                selection_mode: AdapterSelectionMode::KnownProfile,
                adapter_id: "speckit".to_string(),
                display_name: "Speckit".to_string(),
                command: "boundline-adapter-speckit".to_string(),
                args: Vec::new(),
                registration_source: AdapterRegistrationSource::AdapterAdd,
                discovery_state: AdapterDiscoveryState::ExplicitCommand,
                compatibility_line: "framework-adapter-v1".to_string(),
                updated_at: 1,
            },
            schema_fingerprint: "schema-v1".to_string(),
            completeness_state: AdapterConfigCompletenessState::Complete,
            interactive_resolution: false,
            last_validated_at: Some(1),
            value_count: 0,
            values: Vec::new(),
        }
    }

    fn sample_capability_provider_configuration() -> PersistedCapabilityProviderConfiguration {
        PersistedCapabilityProviderConfiguration {
            registrations: vec![CapabilityProviderRegistration {
                provider_id: "demo-provider".to_string(),
                display_name: "Demo Provider".to_string(),
                transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
                    command_ref: "/bin/echo".to_string(),
                    args: Vec::new(),
                    working_directory_ref: None,
                    environment_ref_names: Vec::new(),
                }),
                registration_source: CapabilityProviderRegistrationSource::OperatorCli,
                discovery_state: CapabilityProviderDiscoveryState::Explicit,
                activation_state: CapabilityProviderActivationState::Active,
                config_refs: vec!["token=config/token".to_string()],
                secret_handle_refs: vec!["provider-secret".to_string()],
                setup_requirements: Vec::new(),
                capability_ids: vec!["capability.demo".to_string()],
                active_profile_id: None,
            }],
            active_provider_id: Some("demo-provider".to_string()),
            last_validated_at: Some(1),
        }
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
            adapter: None,
            capability_provider: None,
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
            assert_eq!(routing.planning.unwrap().model, "o4-mini");
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
