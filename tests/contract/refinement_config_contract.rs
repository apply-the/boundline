//! Contract tests for refinement profile config loading.

use std::fs;
use std::path::{Path, PathBuf};

use boundline::domain::refinement::{
    DEFAULT_MAX_ROUNDS, RefinementConfigError, RefinementProfile, RefinementRoles,
    load_refinement_profile, resolve_effective_profile,
};

fn write_config(workspace: &Path, content: &str) {
    let d = workspace.join(".boundline");
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join("refinement-profiles.toml"), content).unwrap();
}

fn tmp_ws() -> (PathBuf, PathBuf) {
    let d = std::env::temp_dir().join(format!("bl-test-{}", std::process::id()));
    fs::create_dir_all(&d).unwrap();
    let w = d.join("ws");
    fs::create_dir_all(&w).unwrap();
    (d, w)
}

#[test]
fn valid_config_loads() {
    let (_t, w) = tmp_ws();
    let toml = r#"[profiles.plan_refinement]
enabled = true
max_rounds = 5
max_elapsed_time_seconds = 600

[profiles.plan_refinement.roles]
planner_provider_id = "p1"
critic_provider_id = "p2"
finalizer_provider_id = "p3"
"#;
    write_config(&w, toml);
    let p = load_refinement_profile(&w, "plan_refinement").unwrap().unwrap();
    assert!(p.enabled);
    assert_eq!(p.max_rounds, 5);
}

#[test]
fn missing_file_returns_none() {
    let (_t, w) = tmp_ws();
    assert!(load_refinement_profile(&w, "plan_refinement").unwrap().is_none());
}

#[test]
fn zero_max_rounds_fails() {
    let p = RefinementProfile {
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        enabled: true,
        max_rounds: 0,
        max_elapsed_time_seconds: 300,
        roles: RefinementRoles {
            planner_provider_id: "p".into(),
            critic_provider_id: "p".into(),
            finalizer_provider_id: "p".into(),
        },
    };
    assert!(matches!(
        resolve_effective_profile(Some(p), false, false, None, None),
        Err(RefinementConfigError::ZeroMaxRounds(0))
    ));
}

#[test]
fn zero_time_fails() {
    let p = RefinementProfile {
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        enabled: true,
        max_rounds: 3,
        max_elapsed_time_seconds: 0,
        roles: RefinementRoles {
            planner_provider_id: "p".into(),
            critic_provider_id: "p".into(),
            finalizer_provider_id: "p".into(),
        },
    };
    assert!(matches!(
        resolve_effective_profile(Some(p), false, false, None, None),
        Err(RefinementConfigError::ZeroMaxElapsedTime(0))
    ));
}

#[test]
fn refine_activates_disabled() {
    let p = RefinementProfile {
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        enabled: false,
        max_rounds: 3,
        max_elapsed_time_seconds: 300,
        roles: RefinementRoles {
            planner_provider_id: "p".into(),
            critic_provider_id: "p".into(),
            finalizer_provider_id: "p".into(),
        },
    };
    assert!(resolve_effective_profile(Some(p), true, false, None, None).unwrap().enabled);
}

#[test]
fn no_refine_disables_enabled() {
    let p = RefinementProfile {
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        enabled: true,
        max_rounds: 3,
        max_elapsed_time_seconds: 300,
        roles: RefinementRoles {
            planner_provider_id: "p".into(),
            critic_provider_id: "p".into(),
            finalizer_provider_id: "p".into(),
        },
    };
    assert!(!resolve_effective_profile(Some(p), false, true, None, None).unwrap().enabled);
}

#[test]
fn cli_overrides_max_rounds() {
    let p = RefinementProfile {
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        enabled: true,
        max_rounds: 3,
        max_elapsed_time_seconds: 300,
        roles: RefinementRoles {
            planner_provider_id: "p".into(),
            critic_provider_id: "p".into(),
            finalizer_provider_id: "p".into(),
        },
    };
    assert_eq!(
        resolve_effective_profile(Some(p), false, false, Some(7), None).unwrap().max_rounds,
        7
    );
}

#[test]
fn cli_only_uses_defaults() {
    let e = resolve_effective_profile(None, true, false, None, None).unwrap();
    assert!(e.enabled);
    assert_eq!(e.max_rounds, DEFAULT_MAX_ROUNDS);
}
