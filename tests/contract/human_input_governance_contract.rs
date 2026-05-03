use std::path::PathBuf;

use boundline::cli::{Cli, DeveloperCommand};
use boundline::domain::governance::GovernanceRuntimeKind;
use clap::Parser;

#[test]
fn capture_accepts_human_governance_intent_flags() {
    let cli = Cli::try_parse_from([
        "boundline",
        "capture",
        "--workspace",
        "/tmp/ws",
        "--goal",
        "Fix the failing checkout flow",
        "--governance",
        "canon",
        "--risk",
        "high",
        "--zone",
        "payments",
        "--owner",
        "platform",
    ])
    .unwrap();

    match cli.command {
        DeveloperCommand::Capture {
            workspace, goal, brief, governance, risk, zone, owner, ..
        } => {
            assert_eq!(workspace, Some(PathBuf::from("/tmp/ws")));
            assert_eq!(goal.as_deref(), Some("Fix the failing checkout flow"));
            assert!(brief.is_empty());
            assert_eq!(governance, Some(GovernanceRuntimeKind::Canon));
            assert_eq!(risk.as_deref(), Some("high"));
            assert_eq!(zone.as_deref(), Some("payments"));
            assert_eq!(owner.as_deref(), Some("platform"));
        }
        other => panic!("expected Capture, got {other:?}"),
    }
}

#[test]
fn run_accepts_local_governance_without_business_fields() {
    let cli = Cli::try_parse_from([
        "boundline",
        "run",
        "--workspace",
        "/tmp/ws",
        "--goal",
        "Fix the failing checkout flow",
        "--governance",
        "local",
    ])
    .unwrap();

    match cli.command {
        DeveloperCommand::Run { workspace, goal, brief, governance, risk, zone, owner, .. } => {
            assert_eq!(workspace, Some(PathBuf::from("/tmp/ws")));
            assert_eq!(goal.as_deref(), Some("Fix the failing checkout flow"));
            assert!(brief.is_empty());
            assert_eq!(governance, Some(GovernanceRuntimeKind::Local));
            assert!(risk.is_none());
            assert!(zone.is_none());
            assert!(owner.is_none());
        }
        other => panic!("expected Run, got {other:?}"),
    }
}
