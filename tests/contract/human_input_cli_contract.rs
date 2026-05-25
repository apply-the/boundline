use std::path::PathBuf;

use boundline::cli::{Cli, DeveloperCommand};
use clap::Parser;

#[test]
fn goal_accepts_brief_only_invocation_with_multiple_briefs() {
    let cli = Cli::try_parse_from([
        "boundline",
        "goal",
        "--workspace",
        "/tmp/ws",
        "--brief",
        "docs/brief.md",
        "--brief",
        "docs/extra.md",
    ])
    .unwrap();
    match cli.command {
        Some(DeveloperCommand::Goal { workspace, goal, brief, .. }) => {
            assert_eq!(workspace, Some(PathBuf::from("/tmp/ws")));
            assert!(goal.is_none(), "goal should be optional when --brief is provided");
            assert_eq!(brief, vec![PathBuf::from("docs/brief.md"), PathBuf::from("docs/extra.md")]);
        }
        other => panic!("expected Goal, got {other:?}"),
    }
}

#[test]
fn goal_accepts_goal_with_a_single_brief() {
    let cli = Cli::try_parse_from([
        "boundline",
        "goal",
        "--workspace",
        ".",
        "--goal",
        "Fix the bug",
        "--brief",
        "brief.md",
    ])
    .unwrap();
    match cli.command {
        Some(DeveloperCommand::Goal { goal, brief, .. }) => {
            assert_eq!(goal.as_deref(), Some("Fix the bug"));
            assert_eq!(brief, vec![PathBuf::from("brief.md")]);
        }
        other => panic!("expected Goal, got {other:?}"),
    }
}

#[test]
fn run_accepts_brief_only_invocation() {
    let cli = Cli::try_parse_from(["boundline", "run", "--workspace", ".", "--brief", "brief.md"])
        .unwrap();
    match cli.command {
        Some(DeveloperCommand::Run { goal, brief, .. }) => {
            assert!(goal.is_none());
            assert_eq!(brief, vec![PathBuf::from("brief.md")]);
        }
        other => panic!("expected Run, got {other:?}"),
    }
}

#[test]
fn goal_without_goal_or_brief_still_parses_clap() {
    // Validation is performed at session level; clap itself accepts the bare invocation
    // because both --goal and --brief are optional.
    let cli = Cli::try_parse_from(["boundline", "goal", "--workspace", "."]).unwrap();
    match cli.command {
        Some(DeveloperCommand::Goal { goal, brief, .. }) => {
            assert!(goal.is_none());
            assert!(brief.is_empty());
        }
        other => panic!("expected Goal, got {other:?}"),
    }
}
