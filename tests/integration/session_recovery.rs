use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use synod::adapters::trace_store::{FileTraceStore, TraceStore};
use synod::domain::session::{ActiveSessionRecord, SessionStatus};
use synod::domain::trace::TraceEventType;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-session-recovery-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.5.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    workspace
}

fn run_synod_in(workspace: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod")).args(args).current_dir(workspace).output().unwrap()
}

fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    let session_path = workspace.join(".synod").join("session.json");
    serde_json::from_slice(&fs::read(session_path).unwrap()).unwrap()
}

#[test]
fn plan_without_an_active_session_fails_with_start_guidance() {
    let workspace = temp_workspace();
    let output = run_synod_in(&workspace, &["plan"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("plan: session error"), "{text}");
    assert!(text.contains("reason: no active session found for the current workspace"), "{text}");
    assert!(text.contains("next_command: synod start"), "{text}");
}

#[test]
fn session_run_persists_failure_state_and_latest_trace_for_non_success_goals() {
    let workspace = temp_workspace();

    let start = run_synod_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_synod_in(
        &workspace,
        &["capture", "--goal", "Force a non-success failure for the default developer flow"],
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let plan = run_synod_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_reason:"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");

    let record = load_session_record(&workspace);

    assert_eq!(record.latest_status, SessionStatus::Failed);
    assert!(record.latest_trace_ref.as_ref().is_some_and(|path| PathBuf::from(path).exists()));
}

#[test]
fn session_run_records_retry_recovery_and_finishes_successfully_for_retryable_goals() {
    let workspace = temp_workspace();

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--goal", "Recover a retryable bounded developer flow"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let record = load_session_record(&workspace);
    assert_eq!(record.latest_status, SessionStatus::Succeeded);

    let trace_path = PathBuf::from(record.latest_trace_ref.unwrap());
    let trace = FileTraceStore::for_workspace(&workspace).load(&trace_path).unwrap();
    assert!(
        trace.events.iter().any(|event| event.event_type == TraceEventType::RetryScheduled),
        "{trace:?}"
    );
}

#[test]
fn session_run_persists_failed_state_for_replan_required_goals_without_replacements() {
    let workspace = temp_workspace();

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--goal", "Force a replan for the default developer flow"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");

    let record = load_session_record(&workspace);
    assert_eq!(record.latest_status, SessionStatus::Failed);
    assert!(
        record
            .latest_terminal_reason
            .as_ref()
            .is_some_and(|reason| reason.message.contains("credible replacement plan"))
    );
    assert!(record.latest_trace_ref.as_ref().is_some_and(|path| PathBuf::from(path).exists()));
}

#[test]
fn session_run_persists_exhausted_state_when_step_budget_is_already_spent() {
    let workspace = temp_workspace();

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--goal", "Summarize the current bounded developer flow"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let session_path = workspace.join(".synod").join("session.json");
    let mut record = load_session_record(&workspace);
    let task = record.active_task.as_mut().unwrap();
    task.limits.max_steps = 1;
    task.total_step_attempts = 1;
    fs::write(&session_path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");

    let record = load_session_record(&workspace);
    assert_eq!(record.latest_status, SessionStatus::Exhausted);
    assert!(record.latest_trace_ref.as_ref().is_some_and(|path| PathBuf::from(path).exists()));
}

#[test]
fn status_and_next_fail_with_start_guidance_when_the_session_record_is_corrupted() {
    let workspace = temp_workspace();

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));

    let session_path = workspace.join(".synod").join("session.json");
    fs::write(
		&session_path,
		"{\n  \"session_id\": \"broken\",\n  \"workspace_ref\": \"\",\n  \"latest_status\": \"planned\",\n  \"created_at\": 2,\n  \"updated_at\": 1\n}\n",
	)
	.unwrap();

    for command in [["status"].as_slice(), ["next"].as_slice()] {
        let output = run_synod_in(&workspace, command);
        let text = terminal_text(&output);
        assert_eq!(output.status.code(), Some(1), "{text}");
        assert!(text.contains("session error"), "{text}");
        assert!(text.contains("next_command: synod start"), "{text}");
    }
}
