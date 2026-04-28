use std::fs;

use synod::adapters::config_store::FileConfigStore;
use uuid::Uuid;

#[test]
fn local_config_path_is_workspace_scoped() {
    let workspace = std::env::temp_dir().join(format!("synod-config-path-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let store = FileConfigStore::for_workspace(&workspace);
    let local_path = store.local_config_path();

    assert!(local_path.starts_with(&workspace));
    assert!(local_path.ends_with(".synod/config.toml"));
}

#[test]
fn global_config_path_uses_synod_suffix() {
    let path = FileConfigStore::global_config_path();
    assert!(path.ends_with("synod/config.toml"));
}
