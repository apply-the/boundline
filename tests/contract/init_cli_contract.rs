use std::path::PathBuf;

use boundline::cli::{Cli, DeveloperCommand};
use boundline::domain::configuration::{InitTemplate, RuntimeKind};
use boundline::domain::domain_templates::DomainFamily;
use clap::Parser;

#[test]
fn init_accepts_template_and_assistant_runtimes() {
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
        "gemini",
        "--force",
    ])
    .unwrap();

    match cli.command {
        DeveloperCommand::Init {
            workspace,
            template,
            assistant,
            domain,
            domain_standard,
            context_binding,
            required_context_binding,
            force,
        } => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(template, Some(InitTemplate::Delivery));
            assert_eq!(assistant, vec![RuntimeKind::Codex, RuntimeKind::Gemini]);
            assert!(domain.is_empty());
            assert!(domain_standard.is_empty());
            assert!(context_binding.is_empty());
            assert!(required_context_binding.is_empty());
            assert!(force);
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
        DeveloperCommand::Init {
            workspace,
            template,
            assistant,
            domain,
            domain_standard,
            context_binding,
            required_context_binding,
            force,
        } => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(template, None);
            assert!(assistant.is_empty());
            assert_eq!(domain, vec![DomainFamily::Systems, DomainFamily::React]);
            assert_eq!(domain_standard, vec!["react=follow the shared UI system".to_string()]);
            assert_eq!(context_binding, vec!["react|design_system|mcp:design-system".to_string()]);
            assert_eq!(
                required_context_binding,
                vec!["react|design_reference|design/reference.md".to_string()]
            );
            assert!(!force);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}

#[test]
fn init_accepts_workspace_without_template() {
    let cli = Cli::try_parse_from(["boundline", "init", "--workspace", "/tmp/ws"]).unwrap();

    match cli.command {
        DeveloperCommand::Init {
            workspace,
            template,
            assistant,
            domain,
            domain_standard,
            context_binding,
            required_context_binding,
            force,
        } => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(template, None);
            assert!(assistant.is_empty());
            assert!(domain.is_empty());
            assert!(domain_standard.is_empty());
            assert!(context_binding.is_empty());
            assert!(required_context_binding.is_empty());
            assert!(!force);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}
