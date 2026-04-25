use std::process::{Command, Output};

fn run_synod(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn run_demo_subcommand_is_advertised_in_top_level_help() {
    let output = run_synod(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let text = text(&output);
    assert!(text.contains("run-demo"), "top-level --help missing run-demo: {text}");
}

#[test]
fn run_demo_help_advertises_only_the_workspace_flag() {
    let output = run_synod(&["run-demo", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    let text = text(&output);
    assert!(text.contains("--workspace"), "missing --workspace flag in: {text}");
    assert!(!text.contains("--goal"), "run-demo must not accept --goal: {text}");
    assert!(!text.contains("--profile"), "run-demo must not accept --profile: {text}");
    assert!(!text.contains("--trace"), "run-demo must not accept --trace: {text}");
}
