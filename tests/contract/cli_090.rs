//! Contract tests for the Boundline 0.90 command classification and routing surface.

use std::{
    error::Error,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use boundline::cli::{
    Cli, CommandClassification, DeveloperCommand, PreviewCommand, command_names,
    preview_command_names, stable_command_names, stable_completion_command_names,
};
use clap::{CommandFactory, Parser};

type TestResult = Result<(), Box<dyn Error>>;

const STABLE_COMMANDS: [&str; 15] = [
    "init",
    "goal",
    "plan",
    "run",
    "status",
    "inspect",
    "doctor",
    "config",
    "models",
    "provider",
    "adapter",
    "index",
    "session",
    "assistant",
    "update",
];
const PENDING_ROOT_COMMANDS: [&str; 4] = ["approve", "recover", "rpc", "serve"];
const REMOVED_COMMANDS: [(&str, &str); 7] = [
    ("orchestrate", "Use `boundline run --until"),
    ("step", "Use `boundline run --one-step`"),
    ("continue", "Use `boundline run --resume`"),
    ("next", "structured `next_actions` from `boundline status`"),
    ("probe", "Use `boundline doctor` or `boundline status`"),
    ("help-next", "Use `boundline doctor` or `boundline status`"),
    ("govern", "Use `boundline plan` or `boundline run`"),
];

fn ensure(condition: bool, message: impl Into<String>) -> TestResult {
    if condition { Ok(()) } else { Err(io::Error::other(message.into()).into()) }
}

fn root_parser_names() -> Vec<String> {
    Cli::command().get_subcommands().map(|command| command.get_name().to_string()).collect()
}

fn rendered_command_names(help: &str) -> Vec<&str> {
    help.lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .collect()
}

fn binary_output<I, S>(workspace: &Path, args: I) -> Result<Output, io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .env("BOUNDLINE_HOME", workspace.join("boundline-home"))
        .output()
}

fn temp_workspace(label: &str) -> Result<PathBuf, io::Error> {
    let path = std::env::temp_dir().join(format!("{label}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

#[test]
fn stable_inventory_is_exact_ordered_and_shared_by_both_cli_surfaces() -> TestResult {
    ensure(stable_command_names() == STABLE_COMMANDS, "stable inventory changed")?;
    ensure(
        stable_completion_command_names() == STABLE_COMMANDS,
        "stable completion inventory changed",
    )?;
    ensure(
        boundline_cli::cli::stable_command_names() == stable_command_names(),
        "root and boundline-cli stable inventories diverged",
    )?;
    ensure(
        command_names(CommandClassification::StableOperational) == STABLE_COMMANDS,
        "runtime stable classification changed",
    )
}

#[test]
fn stable_root_help_contains_only_the_stable_inventory() -> TestResult {
    let help = Cli::command().render_long_help().to_string();
    ensure(
        rendered_command_names(&help) == STABLE_COMMANDS,
        format!("rendered stable Commands section changed: {:?}", rendered_command_names(&help)),
    )?;
    let mut expected_registered =
        STABLE_COMMANDS.iter().map(ToString::to_string).collect::<Vec<_>>();
    expected_registered.push("preview".to_string());
    let mut registered = root_parser_names();
    registered.sort_by_key(|name| {
        expected_registered
            .iter()
            .position(|expected| expected == name)
            .unwrap_or(expected_registered.len())
    });
    ensure(
        registered == expected_registered,
        format!("registered root surface contains an unclassified command: {registered:?}"),
    )?;
    let positions = STABLE_COMMANDS
        .iter()
        .map(|name| {
            help.find(&format!("  {name}"))
                .ok_or_else(|| io::Error::other(format!("stable help omitted {name}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    ensure(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "stable help ordering is nondeterministic",
    )?;
    for name in preview_command_names()
        .iter()
        .chain(command_names(CommandClassification::Internal).iter())
        .chain(PENDING_ROOT_COMMANDS.iter())
        .chain(REMOVED_COMMANDS.iter().map(|(name, _)| name))
    {
        ensure(
            !help.contains(&format!("  {name}")),
            format!("non-stable command {name} leaked into stable help"),
        )?;
    }
    ensure(
        Cli::command().render_long_help().to_string() == help,
        "repeated stable help generation changed",
    )?;
    ensure(
        Cli::try_parse_from(["boundline", "--help"])
            .is_err_and(|error| error.kind() == clap::error::ErrorKind::DisplayHelp),
        "--help flag stopped rendering help",
    )?;
    ensure(
        Cli::try_parse_from(["boundline", "help"]).is_err(),
        "generated root help subcommand remains registered",
    )
}

#[test]
fn pending_commands_are_not_registered_or_completed() -> TestResult {
    let roots = root_parser_names();
    for name in PENDING_ROOT_COMMANDS {
        ensure(!roots.iter().any(|root| root == name), format!("pending root {name} registered"))?;
        ensure(
            !stable_completion_command_names().contains(&name),
            format!("pending root {name} leaked into completion metadata"),
        )?;
        ensure(
            Cli::try_parse_from(["boundline", name]).is_err(),
            format!("pending root {name} parsed"),
        )?;
    }

    for nested in ["abort", "cleanup"] {
        ensure(
            Cli::try_parse_from(["boundline", "session", nested, "session-1"]).is_err(),
            format!("pending session command {nested} parsed"),
        )?;
    }
    let session_help = Cli::command()
        .find_subcommand("session")
        .ok_or_else(|| io::Error::other("session parser missing"))?
        .clone()
        .render_long_help()
        .to_string();
    ensure(!session_help.contains("abort"), "session abort leaked into help")?;
    ensure(!session_help.contains("cleanup"), "session cleanup leaked into help")
}

#[test]
fn removed_commands_are_unregistered_and_emit_side_effect_free_guidance() -> TestResult {
    for (name, replacement) in REMOVED_COMMANDS {
        ensure(
            Cli::try_parse_from(["boundline", name]).is_err(),
            format!("removed command {name} remains registered"),
        )?;
        let workspace = temp_workspace(&format!("boundline-m1c-removed-{name}"))?;
        let before = fs::read_dir(&workspace)?.count();
        let output = binary_output(&workspace, [name])?;
        let after = fs::read_dir(&workspace)?.count();
        let stderr = text(&output.stderr);
        ensure(!output.status.success(), format!("removed command {name} succeeded"))?;
        ensure(before == after, format!("removed command {name} changed workspace state"))?;
        ensure(
            stderr.contains(&format!("`{name}` was removed in Boundline 0.90.")),
            format!("removed command {name} omitted deterministic diagnostic: {stderr}"),
        )?;
        ensure(
            stderr.contains(replacement),
            format!("removed command {name} omitted migration target: {stderr}"),
        )?;
        if name == "govern" {
            ensure(!stderr.contains("approve"), "govern diagnostic promoted approve early")?;
        }
        fs::remove_dir_all(workspace)?;
    }
    Ok(())
}

#[test]
fn ordinary_unknown_command_retains_the_clap_unknown_diagnostic() -> TestResult {
    let workspace = temp_workspace("boundline-m1c-unknown")?;
    let output = binary_output(&workspace, ["definitely-not-a-command"])?;
    let stderr = text(&output.stderr);
    ensure(!output.status.success(), "unknown command succeeded")?;
    ensure(stderr.contains("unrecognized subcommand"), format!("unexpected diagnostic: {stderr}"))?;
    ensure(
        !stderr.contains("was removed in Boundline 0.90"),
        "unknown command got migration help",
    )?;
    fs::remove_dir_all(workspace)?;
    Ok(())
}

#[test]
fn preview_inventory_is_gateway_only_and_visibly_unstable() -> TestResult {
    ensure(
        command_names(CommandClassification::Preview) == preview_command_names()[..5],
        "preview inventory changed",
    )?;
    ensure(
        command_names(CommandClassification::PreviewTransitional) == preview_command_names()[5..],
        "transitional preview inventory changed",
    )?;

    for name in preview_command_names() {
        ensure(
            Cli::try_parse_from(["boundline", name]).is_err(),
            format!("preview command {name} retained a top-level spelling"),
        )?;
        let help_result = Cli::try_parse_from(["boundline", "preview", name, "--help"]);
        ensure(
            help_result
                .as_ref()
                .is_err_and(|error| error.kind() == clap::error::ErrorKind::DisplayHelp),
            format!("preview command {name} did not expose explicit preview help"),
        )?;
    }

    let help = Cli::command()
        .find_subcommand("preview")
        .ok_or_else(|| io::Error::other("preview gateway missing"))?
        .clone()
        .render_long_help()
        .to_string();
    ensure(help.contains("PREVIEW"), "preview help omitted the compatibility warning")?;
    ensure(help.contains("outside the stable compatibility promise"), "preview warning unclear")
}

#[test]
fn preview_commands_parse_to_existing_typed_variants() -> TestResult {
    let cases = [
        vec!["boundline", "preview", "flow", "bug-fix"],
        vec!["boundline", "preview", "workflow", "list"],
        vec!["boundline", "preview", "cluster", "status", "--workspace", "."],
        vec!["boundline", "preview", "council", "adjudicate"],
        vec!["boundline", "preview", "evals", "run"],
        vec![
            "boundline",
            "preview",
            "override",
            "--guardian-id",
            "g",
            "--control-id",
            "c",
            "--level",
            "session",
            "--reason",
            "reviewed",
        ],
        vec!["boundline", "preview", "trace", "compact"],
    ];
    for args in cases {
        let cli = Cli::try_parse_from(args)?;
        let Some(DeveloperCommand::Preview { command }) = cli.command else {
            return Err(io::Error::other("preview command did not parse through gateway").into());
        };
        ensure(
            matches!(
                command,
                PreviewCommand::Flow { .. }
                    | PreviewCommand::Workflow { .. }
                    | PreviewCommand::Cluster { .. }
                    | PreviewCommand::Council { .. }
                    | PreviewCommand::Evals { .. }
                    | PreviewCommand::Override { .. }
                    | PreviewCommand::Trace { .. }
            ),
            "unexpected preview variant",
        )?;
    }
    Ok(())
}

#[test]
fn internal_commands_are_runtime_classified_but_not_publicly_registered() -> TestResult {
    ensure(
        command_names(CommandClassification::Internal).len() == 2,
        "internal inventory cardinality changed",
    )?;
    for name in command_names(CommandClassification::Internal) {
        ensure(
            Cli::try_parse_from(["boundline", name]).is_err(),
            format!("internal command {name} remains public"),
        )?;
        ensure(
            !stable_completion_command_names().contains(&name),
            format!("internal command {name} leaked into completions"),
        )?;
    }
    Ok(())
}

#[test]
fn removed_commands_are_runtime_classified_without_parser_aliases() -> TestResult {
    let expected: Vec<&str> = REMOVED_COMMANDS.iter().map(|(name, _)| *name).collect();
    ensure(command_names(CommandClassification::Removed) == expected, "removed inventory changed")?;
    let roots = root_parser_names();
    for name in expected {
        ensure(!roots.iter().any(|root| root == name), format!("removed root {name} registered"))?;
    }
    Ok(())
}

#[test]
fn run_replacements_parse_without_placeholder_execution_routes() -> TestResult {
    let until = Cli::try_parse_from(["boundline", "run", "--until"])?;
    ensure(
        matches!(
            until.command,
            Some(DeveloperCommand::Run {
                route: boundline::cli::RunRouteArgs {
                    until: Some(_),
                    one_step: false,
                    resume_session: false,
                    ..
                },
                ..
            })
        ),
        "run --until did not select orchestration routing",
    )?;
    let one_step = Cli::try_parse_from(["boundline", "run", "--one-step"])?;
    ensure(
        matches!(
            one_step.command,
            Some(DeveloperCommand::Run {
                route: boundline::cli::RunRouteArgs {
                    until: None,
                    one_step: true,
                    resume_session: false,
                    ..
                },
                ..
            })
        ),
        "run --one-step did not select step routing",
    )?;
    let resume = Cli::try_parse_from(["boundline", "run", "--resume"])?;
    ensure(
        matches!(
            resume.command,
            Some(DeveloperCommand::Run {
                route: boundline::cli::RunRouteArgs {
                    until: None,
                    one_step: false,
                    resume_session: true,
                    ..
                },
                ..
            })
        ),
        "bare run --resume did not select continuation routing",
    )?;
    ensure(
        Cli::try_parse_from(["boundline", "run", "--resume", "legacy-run-id"]).is_err(),
        "placeholder run --resume <ID> route remains public",
    )?;
    ensure(
        Cli::try_parse_from(["boundline", "run", "--plan", "plan.json"]).is_err(),
        "placeholder run --plan route remains public",
    )?;

    let incompatible_common = [
        ["--goal", "changed"],
        ["--brief", "brief.md"],
        ["--governance", "local"],
        ["--risk", "high"],
        ["--zone", "red"],
        ["--owner", "owner"],
        ["--mode", "architecture"],
        ["--compatibility", ""],
        ["--no-canon", ""],
    ];
    for route in ["--one-step", "--resume"] {
        for option in incompatible_common {
            let mut args = vec!["boundline", "run", route, option[0]];
            if !option[1].is_empty() {
                args.push(option[1]);
            }
            ensure(
                Cli::try_parse_from(args).is_err(),
                format!("{route} accepted incompatible option {}", option[0]),
            )?;
        }
    }
    for option in [["--mode", "architecture"], ["--compatibility", ""]] {
        let mut args = vec!["boundline", "run", "--until", option[0]];
        if !option[1].is_empty() {
            args.push(option[1]);
        }
        ensure(
            Cli::try_parse_from(args).is_err(),
            format!("--until accepted incompatible option {}", option[0]),
        )?;
    }

    let workspace = temp_workspace("boundline-m1c-run-conflicts")?;
    for args in [
        vec!["run", "--one-step", "--goal", "changed"],
        vec!["run", "--resume", "--risk", "high"],
        vec!["run", "--until", "--mode", "architecture"],
    ] {
        let before = fs::read_dir(&workspace)?.count();
        let output = binary_output(&workspace, args)?;
        let after = fs::read_dir(&workspace)?.count();
        ensure(!output.status.success(), "incompatible run route succeeded")?;
        ensure(before == after, "incompatible run route changed workspace state")?;
    }
    fs::remove_dir_all(workspace)?;
    Ok(())
}

#[test]
fn stable_administration_nested_help_is_preserved_without_preview_duplicates() -> TestResult {
    let expected = [
        (
            "config",
            &[
                "show",
                "set",
                "set-capability",
                "set-canon",
                "set-semantic-acceleration",
                "unset",
                "unset-capability",
                "set-effort",
                "unset-effort",
                "set-domain",
                "unset-domain",
                "bind-context",
                "unbind-context",
            ][..],
        ),
        ("models", &["auth"][..]),
        ("provider", &["add", "show", "remove", "health"][..]),
        ("adapter", &["add", "show", "remove"][..]),
        ("index", &["status", "refresh", "rebuild", "clean", "doctor"][..]),
        ("session", &["list", "resume"][..]),
        ("assistant", &["install"][..]),
    ];
    let command = Cli::command();
    for (root, expected_nested) in expected {
        let nested: Vec<&str> = command
            .find_subcommand(root)
            .ok_or_else(|| io::Error::other(format!("{root} parser missing")))?
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect();
        ensure(nested == expected_nested, format!("{root} nested surface changed: {nested:?}"))?;
    }
    Ok(())
}

#[test]
fn stable_completion_metadata_is_repeatable_and_excludes_the_preview_gateway() -> TestResult {
    let first = stable_completion_command_names();
    let second = stable_completion_command_names();
    ensure(first == second, "stable completion metadata changed between reads")?;
    ensure(!first.contains(&"preview"), "preview gateway leaked into stable completions")
}

#[test]
fn both_cli_crates_render_the_same_stable_help() -> TestResult {
    let root_help = Cli::command().render_long_help().to_string();
    let crate_help = boundline_cli::cli::Cli::command().render_long_help().to_string();
    ensure(root_help == crate_help, "root and boundline-cli help surfaces diverged")
}

#[test]
fn legacy_preview_root_spelling_is_an_unknown_command_without_migration_guidance() -> TestResult {
    let workspace = temp_workspace("boundline-m1c-preview-root")?;
    let output = binary_output(&workspace, ["workflow"])?;
    let stderr = text(&output.stderr);
    ensure(!output.status.success(), "legacy preview root spelling succeeded")?;
    ensure(stderr.contains("unrecognized subcommand"), format!("unexpected diagnostic: {stderr}"))?;
    ensure(
        !stderr.contains("was removed in Boundline 0.90"),
        "preview root spelling received removed-command guidance",
    )?;
    fs::remove_dir_all(workspace)?;
    Ok(())
}

#[test]
fn checkpoint_internal_spelling_is_side_effect_free_and_not_advertised() -> TestResult {
    let workspace = temp_workspace("boundline-m1c-checkpoint-internal")?;
    let before = fs::read_dir(&workspace)?.count();
    let output = binary_output(&workspace, ["checkpoint"])?;
    let after = fs::read_dir(&workspace)?.count();
    let stderr = text(&output.stderr);
    ensure(!output.status.success(), "checkpoint remained executable")?;
    ensure(before == after, "checkpoint invocation changed workspace state")?;
    ensure(!stderr.contains("Use `boundline"), "checkpoint received migration guidance")?;
    fs::remove_dir_all(workspace)?;
    Ok(())
}

#[test]
fn exec_internal_spelling_is_side_effect_free_and_not_advertised() -> TestResult {
    let workspace = temp_workspace("boundline-m1c-exec-internal")?;
    let before = fs::read_dir(&workspace)?.count();
    let output = binary_output(&workspace, ["exec"])?;
    let after = fs::read_dir(&workspace)?.count();
    let stderr = text(&output.stderr);
    ensure(!output.status.success(), "exec remained executable")?;
    ensure(before == after, "exec invocation changed workspace state")?;
    ensure(!stderr.contains("Use `boundline"), "exec received migration guidance")?;
    fs::remove_dir_all(workspace)?;
    Ok(())
}
