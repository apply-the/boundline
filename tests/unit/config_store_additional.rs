use std::fs;

use boundline::adapters::config_store::FileConfigStore;
use uuid::Uuid;

#[test]
fn local_config_path_is_workspace_scoped() {
    let workspace = std::env::temp_dir().join(format!("boundline-config-path-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let store = FileConfigStore::for_workspace(&workspace);
    let local_path = store.local_config_path();

    assert!(local_path.starts_with(&workspace));
    assert!(local_path.ends_with(".boundline/config.toml"));
}

#[test]
fn global_config_path_uses_boundline_suffix() {
    let path = FileConfigStore::global_config_path();
    assert!(path.ends_with("boundline/config.toml"));
}
