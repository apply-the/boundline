use std::fs;

use crate::workspace_fixture::{
    TempGitWorkspace, run_boundline_in, run_boundline_in_with_env, supported_canon_path,
    terminal_text,
};

fn empty_workspace(prefix: &str) -> TempGitWorkspace {
    TempGitWorkspace::with_initializer(prefix, |workspace| {
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"boundline-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
    })
}

#[test]
fn config_set_show_and_unset_workspace_slot() {
    let workspace = empty_workspace("boundline-config-workspace");

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--template",
            "change",
            "--assistant",
            "copilot",
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set = run_boundline_in(
        &workspace,
        &[
            "config",
            "set",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "planning",
            "--runtime",
            "codex",
            "--model",
            "o4-mini",
        ],
    );
    let set_text = terminal_text(&set);
    assert_eq!(set.status.code(), Some(0), "{set_text}");
    assert!(set_text.contains("config: updated workspace config"), "{set_text}");

    let show = run_boundline_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
        ],
    );
    let show_text = terminal_text(&show);
    assert_eq!(show.status.code(), Some(0), "{show_text}");
    assert!(show_text.contains("planning: codex:o4-mini"), "{show_text}");

    let unset = run_boundline_in(
        &workspace,
        &[
            "config",
            "unset",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "planning",
        ],
    );
    let unset_text = terminal_text(&unset);
    assert_eq!(unset.status.code(), Some(0), "{unset_text}");

    let show_after = run_boundline_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
        ],
    );
    let show_after_text = terminal_text(&show_after);
    assert_eq!(show_after.status.code(), Some(0), "{show_after_text}");
    assert!(show_after_text.contains("planning: <unset>"), "{show_after_text}");
}

#[test]
fn config_set_canon_updates_workspace_mode_selection() {
    let workspace = empty_workspace("boundline-config-canon");
    let canon_path = supported_canon_path();

    let init = run_boundline_in_with_env(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--canon-mode-selection",
            "auto-confirm",
        ],
        &[("PATH", canon_path.as_str())],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set = run_boundline_in(
        &workspace,
        &[
            "config",
            "set-canon",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--mode-selection",
            "auto",
        ],
    );
    let set_text = terminal_text(&set);
    assert_eq!(set.status.code(), Some(0), "{set_text}");
    assert!(set_text.contains("config: updated Canon preferences"), "{set_text}");

    let show = run_boundline_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
        ],
    );
    let show_text = terminal_text(&show);
    assert_eq!(show.status.code(), Some(0), "{show_text}");
    assert!(show_text.contains("canon:"), "{show_text}");
    assert!(show_text.contains("mode_selection: auto"), "{show_text}");
}

#[test]
fn config_show_effective_surfaces_assistant_bindings() {
    let workspace = empty_workspace("boundline-config-effective");

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--template",
            "change",
            "--assistant",
            "copilot",
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set = run_boundline_in(
        &workspace,
        &[
            "config",
            "set",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "planning",
            "--runtime",
            "codex",
            "--model",
            "o4-mini",
        ],
    );
    assert_eq!(set.status.code(), Some(0), "{}", terminal_text(&set));

    let show = run_boundline_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "effective",
        ],
    );
    let show_text = terminal_text(&show);
    assert_eq!(show.status.code(), Some(0), "{show_text}");
    assert!(
        show_text.contains(
            "effective_routing: planning=codex/o4-mini [workspace], implementation=copilot/gpt-4.1 [workspace]"
        ),
        "{show_text}"
    );
    assert!(
        show_text.contains(
            "assistant_bindings: planning=codex, implementation=copilot, verification=copilot, review=copilot, adjudication=codex"
        ),
        "{show_text}"
    );
}

#[test]
fn config_show_effective_surfaces_capability_and_effort_projection() {
    let workspace = empty_workspace("boundline-config-capability-effort");

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--template",
            "change",
            "--assistant",
            "copilot",
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set_route = run_boundline_in(
        &workspace,
        &[
            "config",
            "set",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "implementation",
            "--runtime",
            "claude",
            "--model",
            "sonnet-4",
        ],
    );
    assert_eq!(set_route.status.code(), Some(0), "{}", terminal_text(&set_route));

    let set_capability = run_boundline_in(
        &workspace,
        &[
            "config",
            "set-capability",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--runtime",
            "claude",
            "--continuation",
            "unsupported",
            "--resume",
            "unsupported",
            "--validation",
            "supported",
            "--handoff-target",
            "unsupported",
            "--escalation-context",
            "supported",
            "--notes",
            "requires a handoff for bounded continuation",
        ],
    );
    assert_eq!(set_capability.status.code(), Some(0), "{}", terminal_text(&set_capability));

    let set_effort = run_boundline_in(
        &workspace,
        &[
            "config",
            "set-effort",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--slot",
            "implementation",
            "--level",
            "high",
            "--fallback",
            "preserve",
            "--rationale",
            "keep implementation on the highest-effort bounded path",
        ],
    );
    assert_eq!(set_effort.status.code(), Some(0), "{}", terminal_text(&set_effort));

    let show = run_boundline_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "effective",
        ],
    );
    let show_text = terminal_text(&show);
    assert_eq!(show.status.code(), Some(0), "{show_text}");
    assert!(
        show_text.contains(
            "effective_routing: planning=copilot/gpt-4.1 [workspace], implementation=claude/sonnet-4 [workspace]"
        ),
        "{show_text}"
    );
    assert!(show_text.contains("runtime_capabilities:"), "{show_text}");
    assert!(
        show_text.contains(
            "- claude: continuation=unsupported, resume=unsupported, validation=supported, handoff_target=unsupported, escalation_context=supported, notes=requires a handoff for bounded continuation [workspace]"
        ),
        "{show_text}"
    );
    assert!(show_text.contains("slot_effort_policies:"), "{show_text}");
    assert!(
        show_text.contains(
            "- implementation: level=high, fallback=preserve, rationale=keep implementation on the highest-effort bounded path [workspace]"
        ),
        "{show_text}"
    );
}

#[test]
fn config_domain_commands_surface_effective_domain_templates() {
    let workspace = empty_workspace("boundline-config-domain");

    let init = run_boundline_in(
        &workspace,
        &[
            "init",
            "--non-interactive",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--template",
            "change",
            "--assistant",
            "copilot",
        ],
    );
    assert_eq!(init.status.code(), Some(0), "{}", terminal_text(&init));

    let set_domain = run_boundline_in(
        &workspace,
        &[
            "config",
            "set-domain",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--family",
            "react",
            "--enable",
            "--standards",
            "follow the shared ui system",
        ],
    );
    assert_eq!(set_domain.status.code(), Some(0), "{}", terminal_text(&set_domain));

    let bind_context = run_boundline_in(
        &workspace,
        &[
            "config",
            "bind-context",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "workspace",
            "--family",
            "react",
            "--kind",
            "design-system",
            "--reference",
            "mcp:design-system",
            "--required",
        ],
    );
    assert_eq!(bind_context.status.code(), Some(0), "{}", terminal_text(&bind_context));

    let show = run_boundline_in(
        &workspace,
        &[
            "config",
            "show",
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--scope",
            "effective",
        ],
    );
    let show_text = terminal_text(&show);
    assert_eq!(show.status.code(), Some(0), "{show_text}");
    assert!(show_text.contains("domain_templates:"), "{show_text}");
    assert!(show_text.contains("- react: enabled=true [workspace]"), "{show_text}");
    assert!(
        show_text.contains("design_system mcp:design-system (required) [workspace]"),
        "{show_text}"
    );
}
