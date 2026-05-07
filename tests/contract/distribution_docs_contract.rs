use std::fs;
use std::path::Path;

#[test]
fn docs_split_quick_path_and_advanced_architecture_explicitly() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(repo_root.join("README.md")).unwrap();
    let getting_started = fs::read_to_string(repo_root.join("docs/getting-started.md")).unwrap();
    let architecture = fs::read_to_string(repo_root.join("docs/architecture.md")).unwrap();
    let assistant = fs::read_to_string(repo_root.join("assistant/README.md")).unwrap();

    assert!(readme.contains("## Quick Path Brutale"));
    assert!(readme.contains("boundline doctor --install"));
    assert!(readme.contains("planning=copilot:gpt-5.4"));
    assert!(readme.contains("docs/architecture.md"));

    assert!(getting_started.contains("## Quick Path Brutale"));
    assert!(getting_started.contains("## When Canon Matters"));
    assert!(getting_started.contains("boundline doctor --install"));
    assert!(getting_started.contains("boundline config show --workspace <workspace>"));

    assert!(architecture.contains("## Boundline Versus Canon"));
    assert!(architecture.contains("## Distribution And Update Model"));

    assert!(assistant.contains("boundline doctor --install"));
    assert!(assistant.contains("route_setup"));
    assert!(assistant.contains("docs/architecture.md"));
    assert!(assistant.contains("Canon is the optional governed companion runtime"));
}
