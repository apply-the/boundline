use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn cluster_scope_config_is_used_for_effective_resolution_until_workspace_overrides_it() {
    let primary = temp_fixture_workspace("boundline-cluster-config-primary");
    let secondary = temp_fixture_workspace("boundline-cluster-config-secondary");

    let init = run_boundline_in(
        &primary,
        &[
            "cluster",
            "init",
            "--workspace",
            primary.to_string_lossy().as_ref(),
            "--cluster-id",
            "delivery-a",
            "--member",
            primary.to_string_lossy().as_ref(),
            "--member",
            secondary.to_string_lossy().as_ref(),
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set_cluster = run_boundline_in(
        &primary,
        &[
            "config",
            "set",
            "--cluster",
            primary.to_string_lossy().as_ref(),
            "--scope",
            "cluster",
            "--slot",
            "planning",
            "--runtime",
            "codex",
            "--model",
            "gpt-5-codex",
        ],
    );
    assert_eq!(set_cluster.status.code(), Some(0), "{}", terminal_text(&set_cluster));

    let show_cluster = run_boundline_in(
        &secondary,
        &[
            "config",
            "show",
            "--workspace",
            secondary.to_string_lossy().as_ref(),
            "--cluster",
            primary.to_string_lossy().as_ref(),
            "--scope",
            "effective",
        ],
    );
    let show_cluster_text = terminal_text(&show_cluster);
    assert_eq!(show_cluster.status.code(), Some(0), "{show_cluster_text}");
    assert!(
        show_cluster_text.contains("planning: codex:gpt-5-codex [cluster]"),
        "{show_cluster_text}"
    );

    let set_workspace = run_boundline_in(
        &secondary,
        &[
            "config",
            "set",
            "--workspace",
            secondary.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "planning",
            "--runtime",
            "copilot",
            "--model",
            "gpt-5.4",
        ],
    );
    assert_eq!(set_workspace.status.code(), Some(0), "{}", terminal_text(&set_workspace));

    let show_workspace = run_boundline_in(
        &secondary,
        &[
            "config",
            "show",
            "--workspace",
            secondary.to_string_lossy().as_ref(),
            "--cluster",
            primary.to_string_lossy().as_ref(),
            "--scope",
            "effective",
        ],
    );
    let show_workspace_text = terminal_text(&show_workspace);
    assert_eq!(show_workspace.status.code(), Some(0), "{show_workspace_text}");
    assert!(
        show_workspace_text.contains("planning: copilot:gpt-5.4 [workspace]"),
        "{show_workspace_text}"
    );
}
