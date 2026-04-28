use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::cluster::ClusterConfigFile;

const CLUSTER_CONFIG_RELATIVE: &str = ".synod/cluster.toml";

#[derive(Debug, Clone)]
pub struct FileClusterStore {
    workspace: PathBuf,
}

impl FileClusterStore {
    pub fn for_workspace(workspace: &Path) -> Self {
        Self { workspace: workspace.to_path_buf() }
    }

    pub fn cluster_config_path(&self) -> PathBuf {
        self.workspace.join(CLUSTER_CONFIG_RELATIVE)
    }

    pub fn load(&self) -> Result<Option<ClusterConfigFile>, ClusterStoreError> {
        load_from_path(&self.cluster_config_path())
    }

    pub fn save(&self, config: &ClusterConfigFile) -> Result<PathBuf, ClusterStoreError> {
        save_to_path(&self.cluster_config_path(), config)
    }
}

fn load_from_path(path: &Path) -> Result<Option<ClusterConfigFile>, ClusterStoreError> {
    if !path.is_file() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)
        .map_err(|source| ClusterStoreError::Read { path: path.to_path_buf(), source })?;

    let parsed = toml::from_str::<ClusterConfigFile>(&contents)
        .map_err(|source| ClusterStoreError::Parse { path: path.to_path_buf(), source })?;

    parsed.validate().map_err(|source| ClusterStoreError::InvalidConfig {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;

    Ok(Some(parsed))
}

fn save_to_path(path: &Path, config: &ClusterConfigFile) -> Result<PathBuf, ClusterStoreError> {
    config.validate().map_err(|source| ClusterStoreError::InvalidConfig {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;

    let encoded = toml::to_string_pretty(config)
        .map_err(|source| ClusterStoreError::Serialize { path: path.to_path_buf(), source })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| ClusterStoreError::Write { path: parent.to_path_buf(), source })?;
    }

    fs::write(path, encoded)
        .map_err(|source| ClusterStoreError::Write { path: path.to_path_buf(), source })?;

    Ok(path.to_path_buf())
}

#[derive(Debug, Error)]
pub enum ClusterStoreError {
    #[error("failed to read cluster config at {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("failed to parse cluster config at {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
    #[error("failed to serialize cluster config at {path}: {source}")]
    Serialize { path: PathBuf, source: toml::ser::Error },
    #[error("failed to write cluster config at {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
    #[error("invalid cluster config at {path}: {message}")]
    InvalidConfig { path: PathBuf, message: String },
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::FileClusterStore;
    use crate::domain::cluster::{
        ClusterConfigFile, ClusterMemberRegistration, ClusterMemberRole, WorkspaceCluster,
    };

    #[test]
    fn cluster_config_round_trip_works() {
        let workspace = std::env::temp_dir().join(format!("synod-cluster-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();

        let store = FileClusterStore::for_workspace(&workspace);
        let config = ClusterConfigFile {
            version: 1,
            cluster: WorkspaceCluster {
                cluster_id: "delivery-a".to_string(),
                primary_workspace_ref: workspace.to_string_lossy().into_owned(),
                members: vec![
                    ClusterMemberRegistration {
                        workspace_ref: workspace.to_string_lossy().into_owned(),
                        display_name: None,
                        role: ClusterMemberRole::Primary,
                    },
                    ClusterMemberRegistration {
                        workspace_ref: "/tmp/other".to_string(),
                        display_name: None,
                        role: ClusterMemberRole::Member,
                    },
                ],
                created_at: 1,
                updated_at: 1,
            },
            ..ClusterConfigFile::default()
        };

        let path = store.save(&config).unwrap();
        assert!(path.ends_with(".synod/cluster.toml"));

        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.cluster.cluster_id, "delivery-a");
    }
}
