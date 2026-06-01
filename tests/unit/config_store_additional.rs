use std::error::Error;
use std::fs;

use boundline::adapters::config_store::FileConfigStore;
use boundline::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, ConfigFile, PersistedAdapterConfiguration,
    RoutingConfig,
};
use boundline::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    StoredAdapterConfigValueState,
};
use uuid::Uuid;

const SAMPLE_ADAPTER_ID: &str = "speckit";
const SAMPLE_ADAPTER_DISPLAY_NAME: &str = "Speckit";
const SAMPLE_ADAPTER_COMMAND: &str = "boundline-adapter-speckit";
const SAMPLE_SCHEMA_FINGERPRINT: &str = "schema-v1";
const SAMPLE_FIELD_KEY: &str = "template_repo";
const SAMPLE_FIELD_PATH: &str = "../boundline-framework-template";

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

#[test]
fn local_adapter_round_trip_persists_top_level_adapter_block() -> Result<(), Box<dyn Error>> {
    let workspace =
        std::env::temp_dir().join(format!("boundline-config-adapter-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)?;

    let store = FileConfigStore::for_workspace(&workspace);
    let adapter = sample_persisted_adapter_configuration();
    store.save_local(&ConfigFile {
        version: 1,
        routing: RoutingConfig::default(),
        canon: None,
        adapter: Some(adapter.clone()),
    })?;

    let loaded =
        store.local_adapter()?.ok_or("expected persisted adapter selection in local config")?;
    assert_eq!(loaded, adapter);

    let rendered = fs::read_to_string(store.local_config_path())?;
    assert!(rendered.contains("[adapter]"));
    assert!(rendered.contains("[[adapter.values]]"));
    assert!(rendered.contains("selection_mode = \"known_profile\""));
    assert!(rendered.contains("compatibility_line = \"framework-adapter-v1\""));

    Ok(())
}

fn sample_persisted_adapter_configuration() -> PersistedAdapterConfiguration {
    PersistedAdapterConfiguration {
        selection: AdapterSelectionRecord {
            selection_mode: AdapterSelectionMode::KnownProfile,
            adapter_id: SAMPLE_ADAPTER_ID.to_string(),
            display_name: SAMPLE_ADAPTER_DISPLAY_NAME.to_string(),
            command: SAMPLE_ADAPTER_COMMAND.to_string(),
            args: Vec::new(),
            registration_source: AdapterRegistrationSource::AdapterAdd,
            discovery_state: AdapterDiscoveryState::ExplicitCommand,
            compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
            updated_at: 42,
        },
        schema_fingerprint: SAMPLE_SCHEMA_FINGERPRINT.to_string(),
        completeness_state: AdapterConfigCompletenessState::Complete,
        interactive_resolution: true,
        last_validated_at: Some(42),
        value_count: 1,
        values: vec![AdapterConfigValueRecord {
            field_key: SAMPLE_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::Path,
            secret: false,
            string_value: None,
            path_value: Some(SAMPLE_FIELD_PATH.to_string()),
            bool_value: None,
            int_value: None,
            value_source: AdapterValueSource::KnownProfileDefault,
            resolution_state: StoredAdapterConfigValueState::Present,
        }],
    }
}
