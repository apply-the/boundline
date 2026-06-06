use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use boundline::SUPPORTED_CANON_VERSION;
use uuid::Uuid;

use crate::workspace_fixture::{target_test_cwd, target_test_dir, terminal_text};

#[test]
fn doctor_install_output_includes_version_pairing_and_channel_fields() {
    let canon_dir = fake_canon_directory(SUPPORTED_CANON_VERSION);
    let output = Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(["doctor", "--install"])
        .env("PATH", &canon_dir)
        .current_dir(target_test_cwd("boundline-distribution-contract-cwd"))
        .output()
        .unwrap();
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("boundline_version:"), "{text}");
    assert!(
        text.contains(&format!("supported_canon_version: {SUPPORTED_CANON_VERSION}")),
        "{text}"
    );
    assert!(text.contains("companion_state: already_satisfied"), "{text}");
    assert!(text.contains("channel_candidates:"), "{text}");
}

fn fake_canon_directory(version: &str) -> PathBuf {
    let directory = target_test_dir(&format!("boundline-distribution-contract-{}", Uuid::new_v4()));
    let canon = directory.join("canon");
    let capabilities = format!(
        r#"{{"canon_version":"{SUPPORTED_CANON_VERSION}","supported_schema_versions":["2026-02-01"],"operations":["start","refresh","capabilities"],"supported_modes":["requirements","discovery","system-shaping","architecture","backlog","change","implementation","refactor","review","verification","pr-review","incident","security-assessment","system-assessment","migration","supply-chain-analysis","brainstorming","debugging","policy-shaping"],"status_values":["governed_ready"],"approval_state_values":["not_needed"],"packet_readiness_values":["reusable"],"compatibility_notes":["stable-json"]}}"#
    );
    fs::write(
        &canon,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf 'canon version {version}\\n'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{capabilities}'\n  exit 0\nfi\nexit 1\n",
            version = version,
            capabilities = capabilities
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&canon).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&canon, permissions).unwrap();
    directory
}
