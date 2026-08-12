//! Shared capability admission rejects every undeclared authority escape.

use std::path::PathBuf;

use boundline_core::execution::capability::{
    CapabilityAdmission, CapabilityPolicy, CapabilityRequest, GitAuthority,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

#[test]
fn provider_tool_and_adapter_share_one_fail_closed_boundary() -> TestResult {
    let root = tempfile::tempdir()?;
    let worktree = root.path().join("worktree");
    let state = root.path().join("state");
    std::fs::create_dir_all(&worktree)?;
    std::fs::create_dir_all(worktree.join("src"))?;
    std::fs::create_dir_all(&state)?;
    let worktree = std::fs::canonicalize(worktree)?;
    let policy = CapabilityPolicy::builder(&worktree, &state)
        .allow_read_path(PathBuf::from("src"))
        .allow_command("cargo", ["test"])
        .allow_environment("RUST_LOG")
        .allow_network_host("crates.io")
        .limits(2, 1_024, 5_000)
        .build()?;
    std::os::unix::fs::symlink(root.path().join("outside"), worktree.join("src/escape"))?;
    let denied = [
        CapabilityRequest::ReadPath(root.path().join("outside")),
        CapabilityRequest::ReadPath(worktree.join("src/escape")),
        CapabilityRequest::Command { executable: "sh".into(), arguments: vec!["-c".into()] },
        CapabilityRequest::Environment("HOME".into()),
        CapabilityRequest::Secret("CARGO_REGISTRY_TOKEN".into()),
        CapabilityRequest::Network("example.com".into()),
        CapabilityRequest::BackgroundProcess,
        CapabilityRequest::ProcessCount(3),
        CapabilityRequest::OutputBytes(1_025),
        CapabilityRequest::Git(GitAuthority::UpdateRef),
        CapabilityRequest::StateRoot(state.join("leases")),
    ];
    for request in denied {
        let message = format!("escape was admitted: {request:?}");
        require(matches!(policy.admit(&request), CapabilityAdmission::Denied { .. }), &message)?;
    }
    Ok(())
}

#[test]
fn exact_declared_capabilities_are_admitted_without_publish_authority() -> TestResult {
    let root = tempfile::tempdir()?;
    let worktree = root.path().join("worktree");
    let state = root.path().join("state");
    std::fs::create_dir_all(&worktree)?;
    std::fs::create_dir_all(worktree.join("src"))?;
    std::fs::create_dir_all(&state)?;
    let worktree = std::fs::canonicalize(worktree)?;
    let policy = CapabilityPolicy::builder(&worktree, &state)
        .allow_read_path(PathBuf::from("src"))
        .allow_write_path(PathBuf::from("src"))
        .allow_command("cargo", ["test"])
        .allow_environment("RUST_LOG")
        .limits(1, 2_048, 10_000)
        .build()?;
    let admitted = [
        CapabilityRequest::ReadPath(worktree.join("src/lib.rs")),
        CapabilityRequest::WritePath(worktree.join("src/lib.rs")),
        CapabilityRequest::Command { executable: "cargo".into(), arguments: vec!["test".into()] },
        CapabilityRequest::Environment("RUST_LOG".into()),
        CapabilityRequest::ProcessCount(1),
        CapabilityRequest::OutputBytes(2_048),
        CapabilityRequest::TimeoutMillis(10_000),
        CapabilityRequest::Git(GitAuthority::Read),
    ];
    for request in admitted {
        require(
            matches!(policy.admit(&request), CapabilityAdmission::Admitted),
            "declared capability denied",
        )?;
    }
    require(
        matches!(
            policy.admit(&CapabilityRequest::Git(GitAuthority::Commit)),
            CapabilityAdmission::Denied { .. }
        ),
        "executor received commit authority",
    )
}
