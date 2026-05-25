use std::path::PathBuf;
use std::process::Command;

use boundline::cli::{Cli, DeveloperCommand};
use boundline::domain::configuration::{AssistantHostKind, InitConfigScope, InitTemplate};
use boundline::domain::domain_templates::DomainFamily;
use clap::{CommandFactory, Parser};

#[test]
fn init_accepts_template_and_assistant_hosts() {
    let cli = Cli::try_parse_from([
        "boundline",
        "init",
        "--workspace",
        "/tmp/ws",
        "--template",
        "delivery",
        "--assistant",
        "codex",
        "--assistant",
        "antigravity",
        "--force",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Init {
            workspace,
            scope,
            non_interactive,
            template,
            assistant,
            route,
            domain,
            domain_standard,
            context_binding,
            required_context_binding,
            canon_mode_selection,
            risk,
            zone,
            owner,
            export_docs,
            refresh,
            diff,
            to,
            force,
        }) => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(scope, InitConfigScope::Workspace);
            assert!(!non_interactive);
            assert_eq!(template, Some(InitTemplate::Delivery));
            assert_eq!(assistant, vec![AssistantHostKind::Codex, AssistantHostKind::Antigravity]);
            assert!(route.is_empty());
            assert!(domain.is_empty());
            assert!(domain_standard.is_empty());
            assert!(context_binding.is_empty());
            assert!(required_context_binding.is_empty());
            assert_eq!(canon_mode_selection, None);
            assert_eq!(risk, None);
            assert_eq!(zone, None);
            assert_eq!(owner, None);
            assert!(!export_docs);
            assert!(!refresh);
            assert!(!diff);
            assert_eq!(to, None);
            assert!(force);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}

#[test]
fn init_accepts_canon_preferences_and_model_routes() {
    let cli = Cli::try_parse_from([
        "boundline",
        "init",
        "--workspace",
        "/tmp/ws",
        "--non-interactive",
        "--canon-mode-selection",
        "auto-confirm",
        "--assistant",
        "copilot",
        "--route",
        "planning=copilot:gpt-4o",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Init {
            non_interactive,
            assistant,
            route,
            canon_mode_selection,
            ..
        }) => {
            assert!(non_interactive);
            assert_eq!(assistant, vec![AssistantHostKind::Copilot]);
            assert_eq!(route, vec!["planning=copilot:gpt-4o".to_string()]);
            assert_eq!(canon_mode_selection.unwrap().to_string(), "auto-confirm");
        }
        other => panic!("expected Init, got {other:?}"),
    }
}

#[test]
fn init_accepts_domain_templates_standards_and_bindings() {
    let cli = Cli::try_parse_from([
        "boundline",
        "init",
        "--workspace",
        "/tmp/ws",
        "--domain",
        "systems",
        "--domain",
        "react",
        "--domain-standard",
        "react=follow the shared UI system",
        "--context-binding",
        "react|design_system|mcp:design-system",
        "--required-context-binding",
        "react|design_reference|design/reference.md",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Init {
            workspace,
            scope,
            non_interactive,
            template,
            assistant,
            route,
            domain,
            domain_standard,
            context_binding,
            required_context_binding,
            canon_mode_selection,
            risk,
            zone,
            owner,
            export_docs,
            refresh,
            diff,
            to,
            force,
        }) => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(scope, InitConfigScope::Workspace);
            assert!(!non_interactive);
            assert_eq!(template, None);
            assert!(assistant.is_empty());
            assert!(route.is_empty());
            assert_eq!(domain, vec![DomainFamily::Systems, DomainFamily::React]);
            assert_eq!(domain_standard, vec!["react=follow the shared UI system".to_string()]);
            assert_eq!(context_binding, vec!["react|design_system|mcp:design-system".to_string()]);
            assert_eq!(
                required_context_binding,
                vec!["react|design_reference|design/reference.md".to_string()]
            );
            assert_eq!(canon_mode_selection, None);
            assert_eq!(risk, None);
            assert_eq!(zone, None);
            assert_eq!(owner, None);
            assert!(!export_docs);
            assert!(!refresh);
            assert!(!diff);
            assert_eq!(to, None);
            assert!(!force);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}

#[test]
fn init_accepts_workspace_without_template() {
    let cli = Cli::try_parse_from(["boundline", "init", "--workspace", "/tmp/ws"]).unwrap();

    match cli.command {
        Some(DeveloperCommand::Init {
            workspace,
            scope,
            non_interactive,
            template,
            assistant,
            route,
            domain,
            domain_standard,
            context_binding,
            required_context_binding,
            canon_mode_selection,
            risk,
            zone,
            owner,
            export_docs,
            refresh,
            diff,
            to,
            force,
        }) => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(scope, InitConfigScope::Workspace);
            assert!(!non_interactive);
            assert_eq!(template, None);
            assert!(assistant.is_empty());
            assert!(route.is_empty());
            assert!(domain.is_empty());
            assert!(domain_standard.is_empty());
            assert!(context_binding.is_empty());
            assert!(required_context_binding.is_empty());
            assert_eq!(canon_mode_selection, None);
            assert_eq!(risk, None);
            assert_eq!(zone, None);
            assert_eq!(owner, None);
            assert!(!export_docs);
            assert!(!refresh);
            assert!(!diff);
            assert_eq!(to, None);
            assert!(!force);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}

#[test]
fn init_accepts_docs_export_refresh_diff_and_custom_root() {
    let cli = Cli::try_parse_from([
        "boundline",
        "init",
        "--workspace",
        "/tmp/ws",
        "--export-docs",
        "--refresh",
        "--to",
        "docs/reference/boundline",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Init { export_docs, refresh, diff, to, .. }) => {
            assert!(export_docs);
            assert!(refresh);
            assert!(!diff);
            assert_eq!(to, Some(PathBuf::from("docs/reference/boundline")));
        }
        other => panic!("expected Init, got {other:?}"),
    }

    let diff_cli = Cli::try_parse_from([
        "boundline",
        "init",
        "--workspace",
        "/tmp/ws",
        "--export-docs",
        "--diff",
    ])
    .unwrap();

    match diff_cli.command {
        Some(DeveloperCommand::Init { export_docs, refresh, diff, to, .. }) => {
            assert!(export_docs);
            assert!(!refresh);
            assert!(diff);
            assert_eq!(to, None);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}

#[test]
fn init_help_explains_supported_assistants_route_shape_and_defaults() {
    let mut command = Cli::command();
    let init = command.find_subcommand_mut("init").expect("init subcommand should exist");
    let mut help = Vec::new();
    init.write_long_help(&mut help).unwrap();
    let help = String::from_utf8(help).unwrap();

    assert!(
        help.contains("install-global defaults, workspace files, and default routing"),
        "{help}"
    );
    assert!(help.contains("claude, codex, copilot, antigravity"), "{help}");
    assert!(help.contains("SLOT=RUNTIME:MODEL"), "{help}");
    assert!(help.contains("planning, implementation, verification, review"), "{help}");
    assert!(help.contains("planning=copilot:gpt-4o"), "{help}");
    assert!(help.contains("--export-docs"), "{help}");
    assert!(help.contains("--refresh"), "{help}");
    assert!(help.contains("--diff"), "{help}");
    assert!(help.contains("--to <PATH>"), "{help}");
    assert!(help.contains("create-only by default"), "{help}");
    assert!(
        help.contains("leave guided routes blank to let selected assistants seed defaults"),
        "{help}"
    );
}

#[test]
fn version_flags_work_without_a_subcommand() {
    let output =
        Command::new(env!("CARGO_BIN_EXE_boundline")).args(["--version"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")), "{stdout}");
}
