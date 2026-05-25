use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::config_store::FileConfigStore;

pub const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
pub const OPENAI_BASE_URL_ENV: &str = "OPENAI_BASE_URL";
pub const DEEPSEEK_API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
pub const DEEPSEEK_BASE_URL_ENV: &str = "DEEPSEEK_BASE_URL";
pub const GROK_API_KEY_ENV: &str = "GROK_API_KEY";
pub const GROK_BASE_URL_ENV: &str = "GROK_BASE_URL";
pub const GROQ_API_KEY_ENV: &str = "GROQ_API_KEY";
pub const GROQ_BASE_URL_ENV: &str = "GROQ_BASE_URL";
pub const OLLAMA_BASE_URL_ENV: &str = "OLLAMA_BASE_URL";
pub const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
pub const ANTHROPIC_BASE_URL_ENV: &str = "ANTHROPIC_BASE_URL";
pub const GEMINI_API_KEY_ENV: &str = "GEMINI_API_KEY";
pub const GITHUB_MODELS_TOKEN_ENV: &str = "GITHUB_MODELS_TOKEN";
pub const GITHUB_MODELS_BASE_URL_ENV: &str = "GITHUB_MODELS_BASE_URL";
pub const GITHUB_MODELS_ORG_ENV: &str = "GITHUB_MODELS_ORG";
pub const COPILOT_GITHUB_TOKEN_ENV: &str = "COPILOT_GITHUB_TOKEN";
pub const GH_TOKEN_ENV: &str = "GH_TOKEN";
pub const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";
// Legacy app-specific alias for a GitHub token. Templates should prefer COPILOT_GITHUB_TOKEN.
pub const COPILOT_API_KEY_ENV: &str = "COPILOT_API_KEY";

const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_GROK_BASE_URL: &str = "https://api.x.ai/v1";
const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434/v1";
const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderEnvTemplateScope {
    Global,
    Workspace,
}

impl ProviderEnvTemplateScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Workspace => "workspace",
        }
    }
}

impl fmt::Display for ProviderEnvTemplateScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedEnvFile {
    pub path: PathBuf,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvLayerReport {
    pub loaded_files: Vec<LoadedEnvFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEnvironmentStatus {
    pub global_env_path: PathBuf,
    pub global_env_template_path: PathBuf,
    pub workspace_env_path: Option<PathBuf>,
    pub workspace_env_local_path: Option<PathBuf>,
    pub workspace_env_template_path: Option<PathBuf>,
    pub global_env_present: bool,
    pub global_env_template_present: bool,
    pub workspace_env_present: bool,
    pub workspace_env_local_present: bool,
    pub workspace_env_template_present: bool,
    pub process_keys_present: Vec<String>,
}

#[derive(Debug, Error)]
pub enum EnvLayerError {
    #[error("failed to read env file {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("failed to parse env file {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

pub fn load_provider_environment(
    workspace: Option<&Path>,
) -> Result<EnvLayerReport, EnvLayerError> {
    let inherited =
        env::vars_os().filter_map(|(key, _)| key.into_string().ok()).collect::<BTreeSet<_>>();
    let mut loaded_files = Vec::new();

    for path in provider_env_files(workspace) {
        if !path.is_file() {
            continue;
        }

        let entries = dotenvy::from_path_iter(&path).map_err(|error| EnvLayerError::Parse {
            path: path.clone(),
            message: error.to_string(),
        })?;

        let mut keys = Vec::new();
        for entry in entries {
            let (key, value) = entry.map_err(|error| EnvLayerError::Parse {
                path: path.clone(),
                message: error.to_string(),
            })?;
            if inherited.contains(&key) {
                continue;
            }

            unsafe {
                env::set_var(&key, &value);
            }
            if !keys.iter().any(|existing| existing == &key) {
                keys.push(key);
            }
        }
        loaded_files.push(LoadedEnvFile { path, keys });
    }

    Ok(EnvLayerReport { loaded_files })
}

pub fn provider_environment_status(workspace: Option<&Path>) -> ProviderEnvironmentStatus {
    let global_env_path = FileConfigStore::global_env_path();
    let global_env_template_path = FileConfigStore::global_env_template_path();
    let store = workspace.map(FileConfigStore::for_workspace);
    let workspace_env_path = store.as_ref().map(FileConfigStore::local_env_path);
    let workspace_env_local_path = store.as_ref().map(FileConfigStore::local_env_local_path);
    let workspace_env_template_path = store.as_ref().map(FileConfigStore::local_env_template_path);

    ProviderEnvironmentStatus {
        global_env_present: global_env_path.is_file(),
        global_env_template_present: global_env_template_path.is_file(),
        workspace_env_present: workspace_env_path.as_ref().is_some_and(|path| path.is_file()),
        workspace_env_local_present: workspace_env_local_path
            .as_ref()
            .is_some_and(|path| path.is_file()),
        workspace_env_template_present: workspace_env_template_path
            .as_ref()
            .is_some_and(|path| path.is_file()),
        process_keys_present: provider_process_keys_present(),
        global_env_path,
        global_env_template_path,
        workspace_env_path,
        workspace_env_local_path,
        workspace_env_template_path,
    }
}

pub fn global_env_template_path() -> PathBuf {
    FileConfigStore::global_env_template_path()
}

pub fn workspace_env_template_path(workspace: &Path) -> PathBuf {
    FileConfigStore::for_workspace(workspace).local_env_template_path()
}

pub fn render_provider_env_template(scope: ProviderEnvTemplateScope) -> String {
    let mut lines = vec![
        format!("# Boundline provider environment template ({scope})"),
        match scope {
            ProviderEnvTemplateScope::Global => {
                "# Rename or copy this file to providers.env under the global Boundline config directory."
                    .to_string()
            }
            ProviderEnvTemplateScope::Workspace => {
                "# Copy values into .env or .env.local to override install-wide provider defaults for this repository."
                    .to_string()
            }
        },
        "# Precedence: process env > workspace .env.local > workspace .env > global providers.env."
            .to_string(),
        "# Route and model choices stay in Boundline config TOML; put secrets and endpoints here only."
            .to_string(),
        String::new(),
        "# OpenAI-compatible runtimes".to_string(),
        format!("{OPENAI_API_KEY_ENV}="),
        format!("{OPENAI_BASE_URL_ENV}={DEFAULT_OPENAI_BASE_URL}"),
        format!("{DEEPSEEK_API_KEY_ENV}="),
        format!("{DEEPSEEK_BASE_URL_ENV}={DEFAULT_DEEPSEEK_BASE_URL}"),
        format!("{GROK_API_KEY_ENV}="),
        format!("{GROK_BASE_URL_ENV}={DEFAULT_GROK_BASE_URL}"),
        format!("{GROQ_API_KEY_ENV}="),
        format!("{GROQ_BASE_URL_ENV}={DEFAULT_GROQ_BASE_URL}"),
        format!("{OLLAMA_BASE_URL_ENV}={DEFAULT_OLLAMA_BASE_URL}"),
        String::new(),
        "# Claude".to_string(),
        format!("{ANTHROPIC_API_KEY_ENV}="),
        format!("{ANTHROPIC_BASE_URL_ENV}={DEFAULT_ANTHROPIC_BASE_URL}"),
        String::new(),
        "# Gemini".to_string(),
        format!("{GEMINI_API_KEY_ENV}="),
        String::new(),
        "# GitHub Models official inference API".to_string(),
        "# Use a fine-grained PAT, GitHub App token, or Actions token with models:read."
            .to_string(),
        format!("{GITHUB_MODELS_TOKEN_ENV}="),
        "# Optional organization attribution for enterprise usage.".to_string(),
        format!("{GITHUB_MODELS_ORG_ENV}="),
        "# Boundline also honors GITHUB_TOKEN for GitHub Models requests.".to_string(),
        String::new(),
        "# Copilot official runtime (GitHub token auth)".to_string(),
        "# Boundline uses the official GitHub Copilot endpoint internally."
            .to_string(),
        "# GitHub user/OAuth token used only for Copilot token exchange."
            .to_string(),
        "# Boundline exchanges it for a short-lived Copilot API token before chat requests."
            .to_string(),
        format!("{COPILOT_GITHUB_TOKEN_ENV}="),
        "# Boundline also honors GH_TOKEN and GITHUB_TOKEN if they are already present."
            .to_string(),
        String::new(),
    ];
    lines.push(String::new());
    lines.join("\n")
}

pub fn provider_env_files(workspace: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = vec![FileConfigStore::global_env_path()];
    if let Some(workspace) = workspace {
        let store = FileConfigStore::for_workspace(workspace);
        paths.push(store.local_env_path());
        paths.push(store.local_env_local_path());
    }
    paths
}

fn provider_process_keys_present() -> Vec<String> {
    provider_key_catalog()
        .into_iter()
        .filter(|key| env::var_os(key).is_some())
        .map(str::to_string)
        .collect()
}

fn provider_key_catalog() -> [&'static str; 19] {
    [
        OPENAI_API_KEY_ENV,
        OPENAI_BASE_URL_ENV,
        DEEPSEEK_API_KEY_ENV,
        DEEPSEEK_BASE_URL_ENV,
        GROK_API_KEY_ENV,
        GROK_BASE_URL_ENV,
        GROQ_API_KEY_ENV,
        GROQ_BASE_URL_ENV,
        OLLAMA_BASE_URL_ENV,
        ANTHROPIC_API_KEY_ENV,
        ANTHROPIC_BASE_URL_ENV,
        GEMINI_API_KEY_ENV,
        GITHUB_MODELS_TOKEN_ENV,
        GITHUB_MODELS_BASE_URL_ENV,
        GITHUB_MODELS_ORG_ENV,
        COPILOT_GITHUB_TOKEN_ENV,
        GH_TOKEN_ENV,
        GITHUB_TOKEN_ENV,
        COPILOT_API_KEY_ENV,
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use uuid::Uuid;

    use super::{
        ANTHROPIC_API_KEY_ENV, ANTHROPIC_BASE_URL_ENV, COPILOT_GITHUB_TOKEN_ENV, EnvLayerError,
        GEMINI_API_KEY_ENV, GITHUB_MODELS_ORG_ENV, GITHUB_MODELS_TOKEN_ENV, OPENAI_API_KEY_ENV,
        ProviderEnvTemplateScope, load_provider_environment, provider_environment_status,
        render_provider_env_template,
    };

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvRestore<'a> {
        saved: BTreeMap<&'static str, Option<OsString>>,
        old_xdg: Option<OsString>,
        old_home: Option<OsString>,
        _lock: MutexGuard<'a, ()>,
    }

    impl Drop for EnvRestore<'_> {
        fn drop(&mut self) {
            unsafe {
                for (key, value) in &self.saved {
                    match value {
                        Some(value) => env::set_var(key, value),
                        None => env::remove_var(key),
                    }
                }
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

    fn with_env_test<T>(
        tracked_keys: &[&'static str],
        xdg_home: Option<&Path>,
        home: Option<&Path>,
        action: impl FnOnce() -> T,
    ) -> T {
        let lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let saved =
            tracked_keys.iter().map(|key| (*key, env::var_os(key))).collect::<BTreeMap<_, _>>();
        let restore = EnvRestore {
            saved,
            old_xdg: env::var_os("XDG_CONFIG_HOME"),
            old_home: env::var_os("HOME"),
            _lock: lock,
        };

        unsafe {
            for key in tracked_keys {
                env::remove_var(key);
            }
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

    fn temp_workspace(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn load_provider_environment_honors_process_env_and_workspace_overrides() {
        let tracked_keys = [OPENAI_API_KEY_ENV, GEMINI_API_KEY_ENV, ANTHROPIC_API_KEY_ENV];
        let workspace = temp_workspace("boundline-env-workspace");
        let xdg_home = temp_workspace("boundline-env-xdg");
        let global_env = xdg_home.join("boundline/providers.env");
        fs::create_dir_all(global_env.parent().unwrap()).unwrap();
        fs::write(
            &global_env,
            format!("{OPENAI_API_KEY_ENV}=global-openai\n{GEMINI_API_KEY_ENV}=global-gemini\n"),
        )
        .unwrap();
        fs::write(
            workspace.join(".env"),
            format!(
                "{OPENAI_API_KEY_ENV}=workspace-openai\n{ANTHROPIC_API_KEY_ENV}=workspace-claude\n"
            ),
        )
        .unwrap();
        fs::write(
            workspace.join(".env.local"),
            format!("{GEMINI_API_KEY_ENV}=workspace-local-gemini\n"),
        )
        .unwrap();

        with_env_test(&tracked_keys, Some(&xdg_home), None, || {
            unsafe {
                env::set_var(OPENAI_API_KEY_ENV, "process-openai");
            }

            let report = load_provider_environment(Some(&workspace)).unwrap();
            assert_eq!(report.loaded_files.len(), 3);
            assert_eq!(env::var(OPENAI_API_KEY_ENV).unwrap(), "process-openai");
            assert_eq!(env::var(GEMINI_API_KEY_ENV).unwrap(), "workspace-local-gemini");
            assert_eq!(env::var(ANTHROPIC_API_KEY_ENV).unwrap(), "workspace-claude");
        });
    }

    #[test]
    fn load_provider_environment_reports_parse_errors() {
        let workspace = temp_workspace("boundline-env-invalid");
        fs::write(workspace.join(".env"), "not valid\n").unwrap();

        let error = load_provider_environment(Some(&workspace)).unwrap_err();
        assert!(matches!(error, EnvLayerError::Parse { .. }));
        assert!(error.to_string().contains("Error parsing line"));
    }

    #[test]
    fn provider_environment_status_reports_paths_and_presence() {
        let workspace = temp_workspace("boundline-env-status");
        let xdg_home = temp_workspace("boundline-env-status-xdg");

        with_env_test(&[OPENAI_API_KEY_ENV], Some(&xdg_home), None, || {
            unsafe {
                env::set_var(OPENAI_API_KEY_ENV, "process-openai");
            }
            let global_template = xdg_home.join("boundline/providers.env.template");
            fs::create_dir_all(global_template.parent().unwrap()).unwrap();
            fs::write(&global_template, "template\n").unwrap();
            fs::write(workspace.join(".env.template"), "workspace template\n").unwrap();

            let status = provider_environment_status(Some(&workspace));
            assert!(status.global_env_template_present);
            assert!(status.workspace_env_template_present);
            assert!(status.process_keys_present.iter().any(|key| key == OPENAI_API_KEY_ENV));
        });
    }

    #[test]
    fn render_provider_env_template_mentions_precedence_and_provider_keys() {
        let global = render_provider_env_template(ProviderEnvTemplateScope::Global);
        assert!(global.contains(
            "process env > workspace .env.local > workspace .env > global providers.env"
        ));
        assert!(global.contains(OPENAI_API_KEY_ENV));
        assert!(global.contains(ANTHROPIC_API_KEY_ENV));
        assert!(global.contains(ANTHROPIC_BASE_URL_ENV));
        assert!(global.contains(GEMINI_API_KEY_ENV));
        assert!(global.contains(GITHUB_MODELS_TOKEN_ENV));
        assert!(global.contains(GITHUB_MODELS_ORG_ENV));
        assert!(global.contains(COPILOT_GITHUB_TOKEN_ENV));
        assert!(!global.contains("COPILOT_BASE_URL"));
        assert!(global.contains("https://api.deepseek.com"));
        assert!(!global.contains("https://api.deepseek.com/v1"));

        let workspace = render_provider_env_template(ProviderEnvTemplateScope::Workspace);
        assert!(workspace.contains("override install-wide provider defaults"));
    }
}
