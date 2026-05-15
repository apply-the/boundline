use std::fs;
use std::path::{Path, PathBuf};

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{execute_capture, execute_plan, execute_start};
use uuid::Uuid;

use crate::workspace_fixture::{run_boundline_in_with_env, temp_fixture_workspace, terminal_text};

const ASSISTANT_ROOT_OVERRIDE_ENV: &str = "BOUNDLINE_ASSISTANT_ROOT";

fn temp_assistant_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(root.join("assistant/packs")).unwrap();
    root
}

fn write_catalog_pack(
    assistant_root: &Path,
    pack_name: &str,
    guidance_index: &str,
    guardian_index: &str,
    guidance_files: &[(&str, &str)],
) {
    let pack_dir = assistant_root.join("assistant/packs").join(pack_name);
    let catalog_dir = pack_dir.join("catalog");

    fs::create_dir_all(&catalog_dir).unwrap();
    fs::write(
        pack_dir.join("pack.toml"),
        format!(
            "[pack]\nid = \"{pack_name}\"\nversion = \"0.1.0\"\nkind = \"guidance-pack\"\ndescription = \"{pack_name}\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n"
        ),
    )
    .unwrap();
    fs::write(
        catalog_dir.join("catalog-manifest.toml"),
        format!(
            "[catalog]\nid = \"{pack_name}\"\nversion = \"0.1.0\"\nkind = \"guidance-catalog\"\nstatus = \"draft\"\ndescription = \"{pack_name}\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n\n[layout]\nguidance_dir = \"guidance\"\nguardians_dir = \"guardians\"\nschemas_dir = \"schemas\"\nexamples_dir = \"examples\"\n\n[pillars]\nincluded = [\"clean-code\", \"resilience\"]\n"
        ),
    )
    .unwrap();
    fs::write(catalog_dir.join("guidance-index.toml"), guidance_index).unwrap();
    fs::write(catalog_dir.join("guardian-index.toml"), guardian_index).unwrap();

    for (relative_path, contents) in guidance_files {
        let file_path = pack_dir.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, contents).unwrap();
    }
}

fn write_pack_without_catalog_manifest(assistant_root: &Path, pack_name: &str) {
    let pack_dir = assistant_root.join("assistant/packs").join(pack_name);
    fs::create_dir_all(pack_dir.join("catalog")).unwrap();
    fs::write(
        pack_dir.join("pack.toml"),
        format!(
            "[pack]\nid = \"{pack_name}\"\nversion = \"0.1.0\"\nkind = \"guidance-pack\"\ndescription = \"{pack_name}\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n"
        ),
    )
    .unwrap();
}

#[test]
fn plan_surfaces_catalog_pack_loading_and_keeps_validation_findings_empty() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-catalog");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing rust tests with explicit clean code and resilience guidance"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let plan_report = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should be persisted");

    assert!(
        plan.guidance_guardian
            .loaded_packs
            .iter()
            .any(|pack| { pack.contains("assistant/packs/guidance-catalog") })
    );
    assert!(plan.guidance_guardian.catalog_validation_findings.is_empty());
    assert!(
        plan.guidance_guardian
            .loaded_guidance_sources
            .iter()
            .any(|source| { source == "assistant/packs/guidance-catalog" })
    );
    assert!(
        plan_report.terminal_output.contains("loaded_packs: assistant/packs/guidance-catalog"),
        "{}",
        plan_report.terminal_output
    );
}

#[test]
fn plan_discovers_valid_catalog_packs_and_skips_missing_catalog_manifests() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-catalog-missing-manifest");
    let assistant_root = temp_assistant_root("boundline-assistant-root-missing-manifest");
    write_catalog_pack(
        &assistant_root,
        "custom-guidance-pack",
        "[guidance.clean_code]\npath = \"guidance/clean-code.md\"\npillar = \"clean-code\"\nstrength = \"recommended\"\napplies_to = [\"planning\"]\nroles = [\"planner\"]\n",
        "",
        &[("guidance/clean-code.md", "# Custom Clean Code\nPrefer the custom pack.\n")],
    );
    write_pack_without_catalog_manifest(&assistant_root, "missing-manifest-pack");

    let assistant_root_value = assistant_root.to_string_lossy().into_owned();
    let env = [(ASSISTANT_ROOT_OVERRIDE_ENV, assistant_root_value.as_str())];

    let start = run_boundline_in_with_env(&workspace, &["start"], &env);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));
    let capture = run_boundline_in_with_env(
        &workspace,
        &["capture", "--goal", "apply the custom clean code planning guidance"],
        &env,
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));
    let plan_report = run_boundline_in_with_env(&workspace, &["plan"], &env);
    let plan_text = terminal_text(&plan_report);
    assert_eq!(plan_report.status.code(), Some(0), "{plan_text}");

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should be persisted");

    assert!(plan.guidance_guardian.loaded_packs.iter().any(|pack| {
        pack.contains("assistant/packs/custom-guidance-pack")
            && pack.contains("pack=custom-guidance-pack")
    }));
    assert!(plan.guidance_guardian.skipped_packs.iter().any(|pack| {
        pack.contains("assistant/packs/missing-manifest-pack")
            && pack.contains("failed to read catalog manifest")
    }));
    assert!(
        plan.guidance_guardian
            .loaded_guidance_sources
            .iter()
            .any(|source| { source == "assistant/packs/custom-guidance-pack" })
    );
    assert!(
        plan_text.contains("loaded_packs: assistant/packs/custom-guidance-pack"),
        "{plan_text}"
    );
    assert!(
        plan_text.contains("skipped_packs: assistant/packs/missing-manifest-pack"),
        "{plan_text}"
    );
}

#[test]
fn plan_discloses_when_canon_guidance_supersedes_catalog_pack_guidance() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-canon");
    fs::create_dir_all(workspace.join(".canon/boundline/guidance")).unwrap();
    fs::write(
        workspace.join(".canon/boundline/guidance/clean-code.md"),
        "# Canon Clean Code\nPrefer the governed standard.\n",
    )
    .unwrap();

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the rust bug with the governed clean code guidance"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    let plan_report = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should be persisted");

    assert!(
        plan.guidance_guardian
            .loaded_guidance_sources
            .iter()
            .any(|source| { source == ".canon/boundline/guidance/clean-code.md" })
    );
    assert!(plan.guidance_guardian.skipped_guidance_sources.iter().any(|source| {
        source.contains("assistant/packs/guidance-catalog") && source.contains("shadowed")
    }));
    assert!(
        plan_report
            .terminal_output
            .contains("loaded_guidance_sources: .canon/boundline/guidance/clean-code.md"),
        "{}",
        plan_report.terminal_output
    );
}

#[test]
fn plan_surfaces_catalog_warning_and_error_findings_from_custom_assistant_root() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-catalog-findings");
    let assistant_root = temp_assistant_root("boundline-assistant-root-findings");
    write_catalog_pack(
        &assistant_root,
        "warning-pack",
        concat!(
            "[guidance.clean_code]\n",
            "path = \"guidance/missing-clean-code.md\"\n",
            "pillar = \"clean-code\"\n",
            "strength = \"recommended\"\n",
            "applies_to = [\"planning\"]\n",
            "roles = [\"planner\"]\n\n",
            "[guidance.resilience]\n",
            "path = \"guidance/resilience.md\"\n",
            "pillar = \"resilience\"\n",
            "strength = \"mandatory\"\n",
            "applies_to = [\"planning\"]\n",
            "roles = [\"planner\"]\n",
        ),
        "",
        &[("guidance/resilience.md", "# Resilience\nPrefer bounded recovery steps.\n")],
    );
    write_pack_without_catalog_manifest(&assistant_root, "error-pack");

    let assistant_root_value = assistant_root.to_string_lossy().into_owned();
    let env = [(ASSISTANT_ROOT_OVERRIDE_ENV, assistant_root_value.as_str())];

    let start = run_boundline_in_with_env(&workspace, &["start"], &env);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));
    let capture = run_boundline_in_with_env(
        &workspace,
        &["capture", "--goal", "plan the resilience fix with explicit catalog findings"],
        &env,
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));
    let plan_report = run_boundline_in_with_env(&workspace, &["plan"], &env);
    let plan_text = terminal_text(&plan_report);
    assert_eq!(plan_report.status.code(), Some(0), "{plan_text}");

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should be persisted");

    assert!(plan.guidance_guardian.catalog_validation_findings.iter().any(|finding| {
        finding.starts_with("warning: assistant/packs/warning-pack/guidance/missing-clean-code.md")
            && finding.contains("missing guidance markdown for catalog entry clean_code")
    }));
    assert!(plan.guidance_guardian.catalog_validation_findings.iter().any(|finding| {
        finding.starts_with("error: assistant/packs/error-pack/catalog/catalog-manifest.toml")
            && finding.contains("failed to read catalog manifest")
    }));
    assert!(plan.guidance_guardian.skipped_guidance_sources.iter().any(|source| {
        source.contains("assistant/packs/warning-pack/guidance/missing-clean-code.md")
            && source.contains("missing guidance markdown for catalog entry clean_code")
    }));
    assert!(plan_text.contains("catalog_validation_findings:"), "{plan_text}");
    assert!(
        plan_text.contains("warning: assistant/packs/warning-pack/guidance/missing-clean-code.md"),
        "{plan_text}"
    );
    assert!(
        plan_text.contains("error: assistant/packs/error-pack/catalog/catalog-manifest.toml"),
        "{plan_text}"
    );
}
