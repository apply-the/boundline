use std::fs;
use std::path::Path;

use crate::dashboard_fixture::{DashboardTestResult, require};

const MAX_NON_TEST_LINES: usize = 500;

#[test]
fn dashboard_implementation_modules_stay_below_file_size_guardrail() -> DashboardTestResult {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [
        "src/domain/dashboard.rs",
        "src/adapters/dashboard_state.rs",
        "src/cli/dashboard.rs",
        "crates/boundline-dashboard/src/app.rs",
        "crates/boundline-dashboard/src/state.rs",
        "crates/boundline-dashboard/src/render.rs",
        "crates/boundline-dashboard/src/input.rs",
        "crates/boundline-dashboard/src/branding.rs",
    ];

    for file in files {
        let path = repo_root.join(file);
        if path.exists() {
            let line_count = fs::read_to_string(&path)?.lines().count();
            require(line_count <= MAX_NON_TEST_LINES, &format!("{file} has {line_count} lines"))?;
        }
    }

    Ok(())
}
