use std::path::PathBuf;

use clap::Parser;
use synod::cli::{Cli, DeveloperCommand};
use synod::domain::configuration::{InitTemplate, RuntimeKind};

#[test]
fn init_accepts_template_and_assistant_runtimes() {
    let cli = Cli::try_parse_from([
        "synod",
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
        DeveloperCommand::Init { workspace, template, assistant, force } => {
            assert_eq!(workspace, PathBuf::from("/tmp/ws"));
            assert_eq!(template, Some(InitTemplate::Delivery));
            assert_eq!(assistant, vec![RuntimeKind::Codex, RuntimeKind::Gemini]);
            assert!(force);
        }
        other => panic!("expected Init, got {other:?}"),
    }
}
