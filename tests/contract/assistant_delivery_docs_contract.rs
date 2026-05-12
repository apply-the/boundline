use std::fs;

#[test]
fn assistant_docs_distinguish_global_repo_local_and_cli_state_authority() {
    let guide = fs::read_to_string("docs/guides/assistant-plugin-packages.md").unwrap();
    let assistant = fs::read_to_string("assistant/README.md").unwrap();

    for text in [&guide, &assistant] {
        assert!(
            text.contains("Global assistant package") || text.contains("global bootstrap"),
            "{text}"
        );
        assert!(
            text.contains("Repo-local assistant package") || text.contains("repo-local"),
            "{text}"
        );
        assert!(text.contains("CLI runtime") || text.contains("CLI remains"), "{text}");
        assert!(text.contains(".boundline/session.json"), "{text}");
        assert!(text.contains("chat history is not authoritative"), "{text}");
    }
}

#[test]
fn readme_documents_chat_cli_and_chat_to_runtime_mapping() {
    let readme = fs::read_to_string("README.md").unwrap();

    assert!(readme.contains("## Use Boundline from chat"), "{readme}");
    assert!(readme.contains("## Use Boundline from CLI"), "{readme}");
    assert!(readme.contains("## How chat commands map to CLI/runtime state"), "{readme}");
    assert!(readme.contains("/boundline:init"), "{readme}");
    assert!(readme.contains("/boundline:continue"), "{readme}");
    assert!(readme.contains("boundline assistant install --host"), "{readme}");
}
