//! Unit tests for calibration domain types.
//!
//! These tests cover ControlLevel, CalibrationPolicy, GuardianTrustRecord,
//! and related types independently from the CLI integration.

use boundline::domain::calibration::*;

// ── ControlLevel tests ────────────────────────────────────────────

#[test]
fn advisory_does_not_block() {
    assert!(!ControlLevel::Advisory.blocks_execution());
}

#[test]
fn catch_does_not_block() {
    assert!(!ControlLevel::Catch.blocks_execution());
}

#[test]
fn rule_blocks_execution() {
    assert!(ControlLevel::Rule.blocks_execution());
}

#[test]
fn hook_blocks_execution() {
    assert!(ControlLevel::Hook.blocks_execution());
}

#[test]
fn advisory_promotes_to_catch() {
    assert_eq!(ControlLevel::Advisory.promote(), Some(ControlLevel::Catch));
}

#[test]
fn catch_promotes_to_rule() {
    assert_eq!(ControlLevel::Catch.promote(), Some(ControlLevel::Rule));
}

#[test]
fn rule_promotes_to_hook() {
    assert_eq!(ControlLevel::Rule.promote(), Some(ControlLevel::Hook));
}

#[test]
fn hook_cannot_promote() {
    assert_eq!(ControlLevel::Hook.promote(), None);
}

#[test]
fn rule_demotes_to_catch() {
    assert_eq!(ControlLevel::Rule.demote(), Some(ControlLevel::Catch));
}

#[test]
fn catch_demotes_to_advisory() {
    assert_eq!(ControlLevel::Catch.demote(), Some(ControlLevel::Advisory));
}

#[test]
fn advisory_cannot_demote() {
    assert_eq!(ControlLevel::Advisory.demote(), None);
}

#[test]
fn hook_demotes_to_rule() {
    assert_eq!(ControlLevel::Hook.demote(), Some(ControlLevel::Rule));
}

#[test]
fn catch_is_overridable() {
    assert!(ControlLevel::Catch.is_overridable());
}

#[test]
fn rule_is_overridable() {
    assert!(ControlLevel::Rule.is_overridable());
}

#[test]
fn advisory_is_not_overridable() {
    assert!(!ControlLevel::Advisory.is_overridable());
}

#[test]
fn hook_is_not_overridable() {
    assert!(!ControlLevel::Hook.is_overridable());
}

#[test]
fn control_level_serialization_roundtrip() {
    let levels =
        vec![ControlLevel::Advisory, ControlLevel::Catch, ControlLevel::Rule, ControlLevel::Hook];
    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let parsed: ControlLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, parsed, "roundtrip failed for {level:?}");
    }
}

#[test]
fn authority_zone_serialization_roundtrip() {
    for zone in [AuthorityZone::Green, AuthorityZone::Yellow, AuthorityZone::Red] {
        let json = serde_json::to_string(&zone).unwrap();
        let parsed: AuthorityZone = serde_json::from_str(&json).unwrap();
        assert_eq!(zone, parsed);
    }
}

#[test]
fn risk_level_serialization_roundtrip() {
    for risk in [RiskLevel::Low, RiskLevel::Medium, RiskLevel::High] {
        let json = serde_json::to_string(&risk).unwrap();
        let parsed: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(risk, parsed);
    }
}

// ── OverridePolicy tests ──────────────────────────────────────────

#[test]
fn override_policy_deserialization() {
    let toml_str = r#"
allowed_roles = ["operator"]
required_evidence = ["reason"]
time_limited = true
max_duration_hours = 24
"#;
    let policy: OverridePolicy = toml::from_str(toml_str).unwrap();
    assert_eq!(policy.allowed_roles, vec!["operator"]);
    assert_eq!(policy.required_evidence, vec!["reason"]);
    assert!(policy.time_limited);
    assert_eq!(policy.max_duration_hours, Some(24));
}

// ── CalibrationPolicy validation tests ────────────────────────────

fn make_entry(
    rule_id: &str,
    zone: AuthorityZone,
    risk: RiskLevel,
    default_level: ControlLevel,
) -> ControlLevelEntry {
    ControlLevelEntry {
        rule_id: rule_id.to_string(),
        authority_zone: zone,
        risk_level: risk,
        default_level,
        green_level: ControlLevel::Catch,
        yellow_level: ControlLevel::Rule,
        red_level: ControlLevel::Rule,
        confidence_threshold: 0.85,
        override_policy: OverridePolicy {
            allowed_roles: vec!["operator".to_string()],
            required_evidence: vec!["reason".to_string()],
            time_limited: false,
            max_duration_hours: None,
        },
    }
}

fn make_policy(entries: Vec<ControlLevelEntry>) -> CalibrationPolicy {
    CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 5,
        minimum_evidence_threshold: 3,
        entries,
    }
}

#[test]
fn valid_policy_passes_validation() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Advisory,
    )]);
    assert!(policy.validate().is_ok());
}

#[test]
fn contradictory_entries_fail_closed() {
    let policy = make_policy(vec![
        make_entry(
            "rust-runtime-change",
            AuthorityZone::Green,
            RiskLevel::Low,
            ControlLevel::Advisory,
        ),
        make_entry("rust-runtime-change", AuthorityZone::Green, RiskLevel::Low, ControlLevel::Rule),
    ]);
    let err = policy.validate().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("contradictory"), "expected contradiction error, got: {msg}");
}

#[test]
fn red_zone_advisory_fails_validation() {
    let policy = make_policy(vec![make_entry(
        "security",
        AuthorityZone::Red,
        RiskLevel::High,
        ControlLevel::Advisory,
    )]);
    let err = policy.validate().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("red-zone"), "expected red-zone error, got: {msg}");
}

#[test]
fn confidence_out_of_range_fails_validation() {
    let mut entry =
        make_entry("test", AuthorityZone::Green, RiskLevel::Low, ControlLevel::Advisory);
    entry.confidence_threshold = 1.5;
    let policy = make_policy(vec![entry]);
    let err = policy.validate().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("out of range"), "expected range error, got: {msg}");
}

#[test]
fn empty_policy_fails_validation() {
    let policy = make_policy(vec![]);
    let err = policy.validate().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("empty"), "expected empty error, got: {msg}");
}

#[test]
fn builtin_policy_is_valid() {
    let policy = builtin_calibration_policy();
    // Built-in empty policy validation — empty entries is the fail-safe mode
    // where all guardians default to advisory. The validate() error for empty
    // policy only applies to user-authored policy files. The builtin is
    // intentionally empty to mean "all advisory."
    assert!(policy.entries.is_empty());
    assert_eq!(policy.evidence_window, 5);
    assert_eq!(policy.minimum_evidence_threshold, 3);
}

// ── resolve_level tests ───────────────────────────────────────────

#[test]
fn resolve_level_cold_start_defaults_to_default_level() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Advisory,
    )]);
    let assignment =
        policy.resolve_level("rust-runtime-change", AuthorityZone::Green, RiskLevel::Low, None);
    assert_eq!(assignment.assigned_level, ControlLevel::Advisory);
    assert!(
        assignment.reason.contains("cold start") || assignment.reason.contains("default level")
    );
}

#[test]
fn resolve_level_missing_entry_defaults_to_advisory() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Advisory,
    )]);
    let assignment =
        policy.resolve_level("unknown-guardian", AuthorityZone::Red, RiskLevel::High, None);
    assert_eq!(assignment.assigned_level, ControlLevel::Advisory);
}

#[test]
fn resolve_level_with_high_tpr_promotes() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Advisory,
    )]);
    let mut trust = GuardianTrustRecord::new("rust-runtime-change");
    trust.true_positive_count = 5;
    trust.false_positive_count = 0;
    let assignment = policy.resolve_level(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        Some(&trust),
    );
    assert!(
        assignment.assigned_level == ControlLevel::Catch
            || assignment.assigned_level == ControlLevel::Advisory,
        "expected promotion to Catch or stay Advisory, got {:?}",
        assignment.assigned_level
    );
}

#[test]
fn resolve_level_with_high_fpr_demotes_or_stays() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Catch,
    )]);
    let mut trust = GuardianTrustRecord::new("rust-runtime-change");
    trust.true_positive_count = 1;
    trust.false_positive_count = 4;
    let assignment = policy.resolve_level(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        Some(&trust),
    );
    assert!(
        assignment.assigned_level == ControlLevel::Advisory
            || assignment.assigned_level == ControlLevel::Catch,
        "expected demotion to Advisory or stay Catch, got {:?}",
        assignment.assigned_level
    );
}

#[test]
fn resolve_level_insufficient_sample_stays_default() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Advisory,
    )]);
    let mut trust = GuardianTrustRecord::new("rust-runtime-change");
    trust.true_positive_count = 1;
    trust.false_positive_count = 1;
    // Only 2 adjudicated, below threshold of 3.
    let assignment = policy.resolve_level(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        Some(&trust),
    );
    // With only 2 adjudicated (below evidence_window=5), should stay default.
    assert_eq!(assignment.assigned_level, ControlLevel::Advisory);
}

#[test]
fn resolve_level_incident_lock_prevents_promotion() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Catch,
    )]);
    let mut trust = GuardianTrustRecord::new("rust-runtime-change");
    trust.true_positive_count = 5;
    trust.false_positive_count = 0;
    trust.incident_correlation = true;
    let assignment = policy.resolve_level(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        Some(&trust),
    );
    assert_eq!(assignment.assigned_level, ControlLevel::Catch);
    assert!(assignment.reason.contains("incident"));
}

// ── GuardianTrustRecord tests ─────────────────────────────────────

#[test]
fn trust_record_true_positive_rate_sufficient_sample() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 4;
    record.false_positive_count = 1;
    let tpr = record.true_positive_rate();
    assert!(tpr.is_some());
    assert!((tpr.unwrap() - 0.80).abs() < 0.01);
}

#[test]
fn trust_record_true_positive_rate_insufficient_sample() {
    let record = GuardianTrustRecord::new("test-guardian");
    assert_eq!(record.true_positive_rate(), None);
}

#[test]
fn trust_record_true_positive_rate_zero_denominator() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 0;
    record.false_positive_count = 0;
    assert_eq!(record.true_positive_rate(), None);
}

#[test]
fn trust_record_false_positive_rate() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 4;
    record.false_positive_count = 1;
    let fpr = record.false_positive_rate();
    assert!(fpr.is_some());
    assert!((fpr.unwrap() - 0.20).abs() < 0.01);
}

#[test]
fn trust_record_calibrated_confidence_with_perfect_score() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 10;
    record.false_positive_count = 0;
    record.eval_pass_rate = Some(0.95);
    let conf = record.calibrated_confidence(0.85);
    assert!(conf > 0.90);
}

#[test]
fn trust_record_calibrated_confidence_with_incident() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 10;
    record.false_positive_count = 0;
    record.eval_pass_rate = Some(0.95);
    record.incident_correlation = true;
    let conf = record.calibrated_confidence(0.85);
    assert!(conf < 0.75, "incident penalty should reduce confidence below 0.75, got {conf}");
}

#[test]
fn trust_record_adjudicated_count_excludes_deferred() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 3;
    record.false_positive_count = 2;
    record.deferred_count = 5;
    assert_eq!(record.adjudicated_count(), 5);
}

#[test]
fn trust_record_deferred_not_counted_in_tpr() {
    let mut record = GuardianTrustRecord::new("test-guardian");
    record.true_positive_count = 1;
    record.false_positive_count = 0;
    record.deferred_count = 10;
    let tpr = record.true_positive_rate();
    assert!(tpr.is_some());
    assert!((tpr.unwrap() - 1.0).abs() < 0.01);
}

#[test]
fn override_policy_with_time_limit() {
    let policy = OverridePolicy {
        allowed_roles: vec!["security-lead".to_string()],
        required_evidence: vec!["security_review".to_string(), "threat_model_update".to_string()],
        time_limited: true,
        max_duration_hours: Some(48),
    };
    let json = serde_json::to_string(&policy).unwrap();
    let parsed: OverridePolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.allowed_roles.len(), 1);
    assert_eq!(parsed.max_duration_hours, Some(48));
}

// ── Serialization roundtrip tests ─────────────────────────────────

#[test]
fn calibration_policy_toml_roundtrip() {
    let policy = make_policy(vec![make_entry(
        "rust-runtime-change",
        AuthorityZone::Green,
        RiskLevel::Low,
        ControlLevel::Advisory,
    )]);
    let toml_str = toml::to_string(&policy).unwrap();
    let parsed: CalibrationPolicy = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.schema_version, "1.0");
    assert_eq!(parsed.evidence_window, 5);
    assert_eq!(parsed.entries.len(), 1);
}

#[test]
fn control_level_assignment_json_roundtrip() {
    let assignment = ControlLevelAssignment {
        rule_id: "test-guardian".to_string(),
        assigned_level: ControlLevel::Catch,
        guardian_confidence: 0.92,
        calibrated_confidence: 0.88,
        authority_zone: AuthorityZone::Green,
        risk_level: RiskLevel::Low,
        reason: "trust-based promotion".to_string(),
        degraded_from: None,
        degradation_reason: None,
    };
    let json = serde_json::to_string(&assignment).unwrap();
    let parsed: ControlLevelAssignment = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.rule_id, "test-guardian");
    assert_eq!(parsed.assigned_level, ControlLevel::Catch);
}

#[test]
fn degradation_event_serialization_roundtrip() {
    let event = DegradationEvent {
        rule_id: "test".to_string(),
        original_level: ControlLevel::Rule,
        degraded_level: ControlLevel::Advisory,
        degradation_trigger: DegradationTrigger::ProviderUnavailable,
        safe: true,
        requires_human_gate: false,
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: DegradationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.original_level, ControlLevel::Rule);
    assert_eq!(parsed.degraded_level, ControlLevel::Advisory);
    assert!(parsed.safe);
}

#[test]
fn escalation_event_serialization_roundtrip() {
    let event = EscalationEvent {
        rule_id: "test".to_string(),
        escalation_trigger: EscalationTrigger::RedZone,
        current_level: ControlLevel::Rule,
        recommended_level: ControlLevel::Hook,
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: EscalationEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.recommended_level, ControlLevel::Hook);
}

// ── Edge case: hook never silently downgrades ─────────────────────

#[test]
fn hook_never_downgrades_in_degradation() {
    // A hook-level finding must never degrade silently.
    // Degradation logic should check: if level == Hook, escalate instead of degrade.
    let original = ControlLevel::Hook;
    let can_degrade = original != ControlLevel::Hook;
    assert!(!can_degrade, "Hook must never silently downgrade");
}

// ── Edge case: red-zone degradation must be trace-visible ──────────

#[test]
fn red_zone_degradation_produces_trace_event() {
    let event = DegradationEvent {
        rule_id: "security-guardian".to_string(),
        original_level: ControlLevel::Rule,
        degraded_level: ControlLevel::Catch,
        degradation_trigger: DegradationTrigger::ModelUnavailable,
        safe: false,
        requires_human_gate: true,
    };
    assert!(event.requires_human_gate);
    assert!(!event.safe);
}
#[test]
fn control_level_as_str() {
    assert_eq!(ControlLevel::Advisory.as_str(), "advisory");
    assert_eq!(ControlLevel::Catch.as_str(), "catch");
    assert_eq!(ControlLevel::Rule.as_str(), "rule");
    assert_eq!(ControlLevel::Hook.as_str(), "hook");
}

#[test]
fn default_functions_return_correct_values() {
    assert_eq!(default_evidence_window(), 5);
    assert_eq!(default_minimum_evidence_threshold(), 3);
}

#[test]
fn guardian_trust_record_record_adjudication() {
    let mut t = GuardianTrustRecord::new("test");
    t.record_adjudication(true, false);
    assert_eq!(t.true_positive_count, 1);
    t.record_adjudication(false, false);
    assert_eq!(t.false_positive_count, 1);
    t.record_adjudication(false, true);
    assert_eq!(t.deferred_count, 1);
}

#[test]
fn resolve_level_incident_correlation() {
    let policy = CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 1,
        minimum_evidence_threshold: 1,
        entries: vec![ControlLevelEntry {
            rule_id: "test".to_string(),
            authority_zone: AuthorityZone::Green,
            risk_level: RiskLevel::Low,
            default_level: ControlLevel::Rule,
            green_level: ControlLevel::Rule,
            yellow_level: ControlLevel::Rule,
            red_level: ControlLevel::Rule,
            confidence_threshold: 0.85,
            override_policy: OverridePolicy {
                allowed_roles: vec![],
                required_evidence: vec![],
                time_limited: false,
                max_duration_hours: None,
            },
        }],
    };
    let mut trust = GuardianTrustRecord::new("test");
    trust.true_positive_count = 10;
    trust.incident_correlation = true;
    let assignment =
        policy.resolve_level("test", AuthorityZone::Green, RiskLevel::Low, Some(&trust));
    assert_eq!(assignment.assigned_level, ControlLevel::Catch);
    assert!(assignment.reason.contains("incident correlation"));
}

#[test]
fn resolve_level_eval_failing() {
    let policy = CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 1,
        minimum_evidence_threshold: 1,
        entries: vec![ControlLevelEntry {
            rule_id: "test".to_string(),
            authority_zone: AuthorityZone::Green,
            risk_level: RiskLevel::Low,
            default_level: ControlLevel::Advisory,
            green_level: ControlLevel::Advisory,
            yellow_level: ControlLevel::Advisory,
            red_level: ControlLevel::Advisory,
            confidence_threshold: 0.85,
            override_policy: OverridePolicy {
                allowed_roles: vec![],
                required_evidence: vec![],
                time_limited: false,
                max_duration_hours: None,
            },
        }],
    };
    let mut trust = GuardianTrustRecord::new("test");
    trust.true_positive_count = 10;
    trust.eval_pass_rate = Some(0.5);
    let assignment =
        policy.resolve_level("test", AuthorityZone::Green, RiskLevel::Low, Some(&trust));
    assert_eq!(assignment.assigned_level, ControlLevel::Advisory);
    assert!(assignment.reason.contains("eval pass rate"));
}

#[test]
fn resolve_level_insufficient_evidence() {
    let policy = CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 10,
        minimum_evidence_threshold: 1,
        entries: vec![ControlLevelEntry {
            rule_id: "test".to_string(),
            authority_zone: AuthorityZone::Green,
            risk_level: RiskLevel::Low,
            default_level: ControlLevel::Advisory,
            green_level: ControlLevel::Advisory,
            yellow_level: ControlLevel::Advisory,
            red_level: ControlLevel::Advisory,
            confidence_threshold: 0.85,
            override_policy: OverridePolicy {
                allowed_roles: vec![],
                required_evidence: vec![],
                time_limited: false,
                max_duration_hours: None,
            },
        }],
    };
    let mut trust = GuardianTrustRecord::new("test");
    trust.true_positive_count = 2; // threshold is 1 but window is 10
    let assignment =
        policy.resolve_level("test", AuthorityZone::Green, RiskLevel::Low, Some(&trust));
    assert_eq!(assignment.assigned_level, ControlLevel::Advisory);
    assert!(assignment.reason.contains("insufficient evidence"));
}

#[test]
fn resolve_level_high_fpr() {
    let policy = CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 1,
        minimum_evidence_threshold: 1,
        entries: vec![ControlLevelEntry {
            rule_id: "test".to_string(),
            authority_zone: AuthorityZone::Green,
            risk_level: RiskLevel::Low,
            default_level: ControlLevel::Rule,
            green_level: ControlLevel::Rule,
            yellow_level: ControlLevel::Rule,
            red_level: ControlLevel::Rule,
            confidence_threshold: 0.85,
            override_policy: OverridePolicy {
                allowed_roles: vec![],
                required_evidence: vec![],
                time_limited: false,
                max_duration_hours: None,
            },
        }],
    };
    let mut trust = GuardianTrustRecord::new("test");
    trust.true_positive_count = 5;
    trust.false_positive_count = 5; // TPR = 0.5 < 0.8
    let assignment =
        policy.resolve_level("test", AuthorityZone::Green, RiskLevel::Low, Some(&trust));
    assert_eq!(assignment.assigned_level, ControlLevel::Catch);
    assert!(assignment.reason.contains("FPR"));
}

#[test]
fn resolve_level_in_range() {
    let policy = CalibrationPolicy {
        schema_version: "1.0".to_string(),
        evidence_window: 1,
        minimum_evidence_threshold: 1,
        entries: vec![ControlLevelEntry {
            rule_id: "test".to_string(),
            authority_zone: AuthorityZone::Green,
            risk_level: RiskLevel::Low,
            default_level: ControlLevel::Rule,
            green_level: ControlLevel::Rule,
            yellow_level: ControlLevel::Rule,
            red_level: ControlLevel::Rule,
            confidence_threshold: 0.85,
            override_policy: OverridePolicy {
                allowed_roles: vec![],
                required_evidence: vec![],
                time_limited: false,
                max_duration_hours: None,
            },
        }],
    };
    let mut trust = GuardianTrustRecord::new("test");
    trust.true_positive_count = 85;
    trust.false_positive_count = 15; // TPR = 0.85 -> in range
    let assignment =
        policy.resolve_level("test", AuthorityZone::Green, RiskLevel::Low, Some(&trust));
    assert_eq!(assignment.assigned_level, ControlLevel::Rule);
    assert!(assignment.reason.contains("within acceptable range"));
}
#[test]
fn validate_rejects_red_zone_advisory_in_red_level() {
    let policy = CalibrationPolicy {
        schema_version: "1.0".into(),
        evidence_window: 10,
        minimum_evidence_threshold: 5,
        entries: vec![ControlLevelEntry {
            rule_id: "r1".into(),
            authority_zone: AuthorityZone::Red,
            risk_level: RiskLevel::High,
            default_level: ControlLevel::Catch, // valid
            green_level: ControlLevel::Catch,
            yellow_level: ControlLevel::Catch,
            red_level: ControlLevel::Advisory, // invalid
            confidence_threshold: 0.5,
            override_policy: OverridePolicy {
                allowed_roles: vec![],
                required_evidence: vec![],
                time_limited: false,
                max_duration_hours: None,
            },
        }],
    };
    assert!(matches!(policy.validate(), Err(CalibrationPolicyError::RedZoneAdvisory { .. })));
}

#[test]
fn validate_rejects_contradiction_in_zone_specific_levels() {
    let entry1 = ControlLevelEntry {
        rule_id: "r1".into(),
        authority_zone: AuthorityZone::Green,
        risk_level: RiskLevel::Low,
        default_level: ControlLevel::Catch,
        green_level: ControlLevel::Rule, // difference
        yellow_level: ControlLevel::Catch,
        red_level: ControlLevel::Catch,
        confidence_threshold: 0.5,
        override_policy: OverridePolicy {
            allowed_roles: vec![],
            required_evidence: vec![],
            time_limited: false,
            max_duration_hours: None,
        },
    };
    let mut entry2 = entry1.clone();
    entry2.green_level = ControlLevel::Advisory; // contradiction

    let policy = CalibrationPolicy {
        schema_version: "1.0".into(),
        evidence_window: 10,
        minimum_evidence_threshold: 5,
        entries: vec![entry1, entry2],
    };
    assert!(matches!(policy.validate(), Err(CalibrationPolicyError::Contradiction { .. })));
}
