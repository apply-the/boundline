use super::{
    AdvancedContextProjection, EXPLANATION_ASSUMPTION_CATEGORY_ARCHITECTURE,
    EXPLANATION_ASSUMPTION_CATEGORY_DOMAIN, EXPLANATION_ASSUMPTION_CATEGORY_GOVERNANCE,
    EXPLANATION_ASSUMPTION_CATEGORY_IMPLEMENTATION, EXPLANATION_ASSUMPTION_CATEGORY_VALIDATION,
    EXPLANATION_ASSUMPTION_RISK_HIGH, EXPLANATION_ASSUMPTION_RISK_LOW,
    EXPLANATION_ASSUMPTION_RISK_MEDIUM, EXPLANATION_ASSUMPTION_SOURCE_CANON,
    EXPLANATION_ASSUMPTION_SOURCE_TRACE, EXPLANATION_ASSUMPTION_SOURCE_WORKSPACE,
    EXPLANATION_ASSUMPTION_STATUS_EXPLICIT, EXPLANATION_ASSUMPTION_STATUS_INFERRED,
    EXPLANATION_ASSUMPTION_STATUS_MISSING, EXPLANATION_CANON_SOURCE_APPROVAL_PROVENANCE,
    EXPLANATION_CANON_SOURCE_GOVERNANCE_ACTION, EXPLANATION_CANON_SOURCE_GOVERNANCE_DECISION,
    EXPLANATION_CANON_SOURCE_GOVERNANCE_PACKET, EXPLANATION_CANON_SOURCE_GOVERNANCE_TIMELINE,
    EXPLANATION_CONFIDENCE_HIGH, EXPLANATION_CONFIDENCE_LOW, EXPLANATION_CONFIDENCE_MEDIUM,
    EXPLANATION_COUNCIL_REQUIRED_NO, EXPLANATION_COUNCIL_REQUIRED_YES,
    EXPLANATION_FALLBACK_CANON_MISSING, EXPLANATION_FALLBACK_CLARIFICATION_PREFIX,
    EXPLANATION_FALLBACK_CONTEXT_STALE_PREFIX, EXPLANATION_FALLBACK_READY,
    EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS,
    EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS,
    EXPLANATION_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES,
    EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE,
    EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS,
    EXPLANATION_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS,
    EXPLANATION_HIDDEN_IMPACT_LABEL_AFFECTED_DOMAINS,
    EXPLANATION_HIDDEN_IMPACT_LABEL_AFFECTED_SYSTEMS,
    EXPLANATION_HIDDEN_IMPACT_LABEL_CONTRACT_EXPOSURES,
    EXPLANATION_HIDDEN_IMPACT_LABEL_MISSING_EVIDENCE,
    EXPLANATION_HIDDEN_IMPACT_LABEL_MISSING_TESTS,
    EXPLANATION_HIDDEN_IMPACT_LABEL_REQUIRED_REVIEWERS, EXPLANATION_LABEL_ASSUMPTION_GROUP,
    EXPLANATION_LABEL_ASSUMPTIONS_SUMMARY, EXPLANATION_LABEL_CHALLENGE_COUNCIL_REQUIRED,
    EXPLANATION_LABEL_CHALLENGE_FAILURE_MODE, EXPLANATION_LABEL_CHALLENGE_MISSING_EVIDENCE,
    EXPLANATION_LABEL_CHALLENGE_REQUIRED_REVIEW, EXPLANATION_LABEL_CHALLENGE_STRONGEST_OBJECTION,
    EXPLANATION_LABEL_CHALLENGE_WEAKEST_ASSUMPTION, EXPLANATION_LABEL_CONFIDENCE_LEVEL,
    EXPLANATION_LABEL_EVIDENCE_SUMMARY, EXPLANATION_LABEL_EXPLAIN_PLAN_GOVERNANCE,
    EXPLANATION_LABEL_EXPLAIN_PLAN_RECOVERY, EXPLANATION_LABEL_EXPLAIN_PLAN_SUMMARY,
    EXPLANATION_LABEL_EXPLAIN_PLAN_VALIDATION, EXPLANATION_LABEL_FALLBACK_DISCLOSURE,
    EXPLANATION_LABEL_HIDDEN_IMPACT_FALLBACK_DISCLOSURE, EXPLANATION_LABEL_HIDDEN_IMPACT_SUMMARY,
    EXPLANATION_LABEL_NEXT_BEST_ACTION, EXPLANATION_LABEL_RISK_SUMMARY,
    EXPLANATION_LABEL_SOURCE_ATTRIBUTION, EXPLANATION_LABEL_WHY_SUMMARY,
    EXPLANATION_MISSING_CANON_SOURCE, EXPLANATION_MISSING_CLARIFICATION_SOURCE,
    EXPLANATION_MISSING_CONTEXT_SOURCE, EXPLANATION_NONE, EXPLANATION_REVIEW_RUNTIME_ONLY,
    EXPLANATION_RISK_CANON_GAP, EXPLANATION_RISK_NO_EXPLICIT_FAILURE,
    EXPLANATION_RUNTIME_SOURCE_AUTHORED_INPUT, EXPLANATION_RUNTIME_SOURCE_CONTEXT,
    EXPLANATION_RUNTIME_SOURCE_DECISION_TIMELINE, EXPLANATION_RUNTIME_SOURCE_REASONING_PROFILE,
    EXPLANATION_RUNTIME_SOURCE_REVIEW_TIMELINE, EXPLANATION_RUNTIME_SOURCE_SESSION_STATE,
    EXPLANATION_RUNTIME_SOURCE_TRACE_EVIDENCE, EXPLANATION_RUNTIME_SOURCE_TRACE_STEPS,
    EXPLANATION_WEAK_ASSUMPTION_NONE, EXPLANATION_WHY_FALLBACK, ExplanationAssumptionEntry,
    ExplanationCognitiveProjection, ExplanationHiddenImpactEntry, ExplanationProjection,
    ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus, ProfileActivationRecord,
    RelationshipCredibilityState, RelationshipKind, RetrievalSourceKind, SemanticCapabilityState,
    SemanticPolicyState, SessionStatusView, TraceSummaryView,
};

fn build_missing_sources(
    canon_empty: bool,
    context_stale: bool,
    clarification_missing: bool,
) -> Vec<&'static str> {
    let mut sources = Vec::new();
    if canon_empty {
        sources.push(EXPLANATION_MISSING_CANON_SOURCE);
    }
    if context_stale {
        sources.push(EXPLANATION_MISSING_CONTEXT_SOURCE);
    }
    if clarification_missing {
        sources.push(EXPLANATION_MISSING_CLARIFICATION_SOURCE);
    }
    sources
}

fn build_evidence_and_attribution(
    runtime_sources: &[&str],
    canon_sources: &[&str],
    missing_sources: &[&str],
) -> (String, String) {
    let evidence_summary = format!(
        "runtime({}): {}; canon({}): {}; missing({}): {}",
        runtime_sources.len(),
        explanation_source_bucket_text(runtime_sources),
        canon_sources.len(),
        explanation_source_bucket_text(canon_sources),
        missing_sources.len(),
        explanation_source_bucket_text(missing_sources)
    );
    let source_attribution = format!(
        "runtime={}; canon={}; missing={}",
        explanation_source_bucket_text(runtime_sources),
        explanation_source_bucket_text(canon_sources),
        explanation_source_bucket_text(missing_sources)
    );
    (evidence_summary, source_attribution)
}

fn explanation_source_bucket_text(labels: &[&str]) -> String {
    if labels.is_empty() { EXPLANATION_NONE.to_string() } else { labels.join(", ") }
}

fn explanation_confidence_level(
    has_failure_signal: bool,
    canon_missing: bool,
    context_stale: bool,
    clarification_required: bool,
) -> &'static str {
    if has_failure_signal || clarification_required {
        EXPLANATION_CONFIDENCE_LOW
    } else if canon_missing || context_stale {
        EXPLANATION_CONFIDENCE_MEDIUM
    } else {
        EXPLANATION_CONFIDENCE_HIGH
    }
}

fn explanation_fallback_disclosure(
    canon_missing: bool,
    context_staleness_reason: Option<&str>,
    clarification_missing_fields: &[String],
) -> String {
    if canon_missing {
        EXPLANATION_FALLBACK_CANON_MISSING.to_string()
    } else if let Some(reason) = context_staleness_reason {
        format!("{EXPLANATION_FALLBACK_CONTEXT_STALE_PREFIX}{reason}")
    } else if !clarification_missing_fields.is_empty() {
        format!(
            "{EXPLANATION_FALLBACK_CLARIFICATION_PREFIX}{}",
            clarification_missing_fields.join(", ")
        )
    } else {
        EXPLANATION_FALLBACK_READY.to_string()
    }
}

pub(crate) fn explanation_projection_lines(projection: &ExplanationProjection) -> Vec<String> {
    vec![
        format!("{EXPLANATION_LABEL_WHY_SUMMARY}: {}", projection.why_summary),
        format!("{EXPLANATION_LABEL_RISK_SUMMARY}: {}", projection.risk_summary),
        format!("{EXPLANATION_LABEL_EVIDENCE_SUMMARY}: {}", projection.evidence_summary),
        format!("{EXPLANATION_LABEL_SOURCE_ATTRIBUTION}: {}", projection.source_attribution),
        format!("{EXPLANATION_LABEL_FALLBACK_DISCLOSURE}: {}", projection.fallback_disclosure),
        format!("{EXPLANATION_LABEL_CONFIDENCE_LEVEL}: {}", projection.confidence_level),
        format!("{EXPLANATION_LABEL_NEXT_BEST_ACTION}: {}", projection.next_best_action),
    ]
}

pub(crate) fn explanation_cognitive_projection_lines(
    projection: &ExplanationCognitiveProjection,
) -> Vec<String> {
    let mut lines = vec![format!(
        "{EXPLANATION_LABEL_ASSUMPTIONS_SUMMARY}: {}",
        projection.assumptions_summary
    )];
    lines.extend(projection.assumption_groups.iter().cloned());
    lines.push(format!(
        "{EXPLANATION_LABEL_HIDDEN_IMPACT_SUMMARY}: {}",
        projection.hidden_impact_summary
    ));
    lines.extend(projection.hidden_impact_lines.iter().cloned());
    if let Some(fallback_disclosure) = projection.hidden_impact_fallback_disclosure.as_deref() {
        lines.push(format!(
            "{EXPLANATION_LABEL_HIDDEN_IMPACT_FALLBACK_DISCLOSURE}: {fallback_disclosure}"
        ));
    }
    lines.push(format!(
        "{EXPLANATION_LABEL_CHALLENGE_STRONGEST_OBJECTION}: {}",
        projection.challenge_strongest_objection
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_CHALLENGE_WEAKEST_ASSUMPTION}: {}",
        projection.challenge_weakest_assumption
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_CHALLENGE_MISSING_EVIDENCE}: {}",
        projection.challenge_missing_evidence
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_CHALLENGE_FAILURE_MODE}: {}",
        projection.challenge_failure_mode
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_CHALLENGE_REQUIRED_REVIEW}: {}",
        projection.challenge_required_review
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_CHALLENGE_COUNCIL_REQUIRED}: {}",
        projection.challenge_council_required
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_EXPLAIN_PLAN_SUMMARY}: {}",
        projection.explain_plan_summary
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_EXPLAIN_PLAN_VALIDATION}: {}",
        projection.explain_plan_validation
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_EXPLAIN_PLAN_GOVERNANCE}: {}",
        projection.explain_plan_governance
    ));
    lines.push(format!(
        "{EXPLANATION_LABEL_EXPLAIN_PLAN_RECOVERY}: {}",
        projection.explain_plan_recovery
    ));
    lines
}

fn reasoning_projection_why_summary(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.and_then(|profile| {
        profile
            .outcome
            .as_ref()
            .map(|outcome| outcome.headline.clone())
            .or_else(|| Some(profile.activation_reason.clone()))
    })
}

fn reasoning_projection_risk_summary(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.and_then(|profile| {
        profile
            .confidence
            .as_ref()
            .map(|confidence| confidence.summary.clone())
            .or_else(|| {
                profile.outcome.as_ref().and_then(|outcome| outcome.disagreement_summary.clone())
            })
            .or_else(|| profile.outcome.as_ref().map(|outcome| outcome.headline.clone()))
    })
}

fn reasoning_projection_confidence_level(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<&'static str> {
    reasoning_profile.and_then(|profile| {
        profile.confidence.as_ref().map(|confidence| confidence.confidence_level.as_str())
    })
}

fn reasoning_projection_next_action(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.and_then(|profile| {
        profile.outcome.as_ref().and_then(|outcome| outcome.next_action.clone())
    })
}

fn reasoning_projection_governance_summary(
    reasoning_profile: Option<&ProfileActivationRecord>,
) -> Option<String> {
    reasoning_profile.map(|profile| {
        let mut parts = vec![
            format!("reasoning_profile={}", profile.profile_id),
            format!("status={}", profile.status.as_str()),
        ];
        if let Some(confidence) = &profile.confidence {
            parts.push(format!("confidence={}", confidence.confidence_level.as_str()));
            parts.push(format!("admission_effect={}", confidence.admission_effect.as_str()));
        }
        if let Some(posture) = &profile.posture {
            parts.push(format!("posture_contract={}", posture.contract_line));
        }
        parts.join("; ")
    })
}

pub(crate) fn explanation_projection_for_trace_summary(
    summary: &TraceSummaryView,
    next_command: &str,
) -> ExplanationProjection {
    let mut runtime_sources = Vec::new();
    if summary.authored_input_summary.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_AUTHORED_INPUT);
    }
    if summary.context_summary.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_CONTEXT);
    }
    if !summary.executed_steps.is_empty() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_TRACE_STEPS);
    }
    if !summary.decision_timeline.is_empty() || !summary.failure_evidence.is_empty() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_TRACE_EVIDENCE);
    }
    if !summary.review_timeline.is_empty() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_REVIEW_TIMELINE);
    }
    if summary.reasoning_profile.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_REASONING_PROFILE);
    }

    let mut canon_sources = Vec::new();
    if !summary.governance_timeline.is_empty() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_GOVERNANCE_TIMELINE);
    }
    if summary.governance_approval_provenance.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_APPROVAL_PROVENANCE);
    }
    if summary.governance_reason.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_GOVERNANCE_DECISION);
    }
    if summary.governance_next_action.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_GOVERNANCE_ACTION);
    }

    let missing_sources = build_missing_sources(
        canon_sources.is_empty(),
        summary.context_staleness_reason.is_some(),
        !summary.clarification_missing_fields.is_empty(),
    );

    let why_summary = reasoning_projection_why_summary(summary.reasoning_profile.as_ref())
        .or_else(|| summary.goal_plan_summary.clone())
        .or_else(|| summary.negotiation_goal_summary.clone())
        .or_else(|| {
            summary.executed_steps.last().map(|step| {
                format!("latest bounded step {} reports {}", step.step_id, step.headline)
            })
        })
        .unwrap_or_else(|| {
            if summary.terminal_reason.message.trim().is_empty() {
                EXPLANATION_WHY_FALLBACK.to_string()
            } else {
                summary.terminal_reason.message.clone()
            }
        });

    let risk_summary = if let Some(reasoning_risk) =
        reasoning_projection_risk_summary(summary.reasoning_profile.as_ref())
    {
        reasoning_risk
    } else if !summary.failure_evidence.is_empty() {
        summary.failure_evidence[0].clone()
    } else if let Some(reason) = summary.context_staleness_reason.as_ref() {
        format!("stale context reduces confidence: {reason}")
    } else if canon_sources.is_empty() {
        EXPLANATION_RISK_CANON_GAP.to_string()
    } else if summary.terminal_reason.message.trim().is_empty() {
        EXPLANATION_RISK_NO_EXPLICIT_FAILURE.to_string()
    } else {
        summary.terminal_reason.message.clone()
    };

    let (evidence_summary, source_attribution) =
        build_evidence_and_attribution(&runtime_sources, &canon_sources, &missing_sources);

    let fallback_disclosure = explanation_fallback_disclosure(
        canon_sources.is_empty(),
        summary.context_staleness_reason.as_deref(),
        &summary.clarification_missing_fields,
    );

    let confidence_level = reasoning_projection_confidence_level(
        summary.reasoning_profile.as_ref(),
    )
    .unwrap_or_else(|| {
        explanation_confidence_level(
            !summary.failure_evidence.is_empty(),
            canon_sources.is_empty(),
            summary.context_staleness_reason.is_some(),
            !summary.clarification_missing_fields.is_empty(),
        )
    });

    let next_best_action = reasoning_projection_next_action(summary.reasoning_profile.as_ref())
        .or_else(|| summary.governance_next_action.clone())
        .unwrap_or_else(|| next_command.to_string());

    ExplanationProjection {
        why_summary,
        risk_summary,
        evidence_summary,
        source_attribution,
        fallback_disclosure,
        confidence_level,
        next_best_action,
    }
}

pub(crate) fn explanation_projection_for_session_status(
    view: &SessionStatusView,
) -> ExplanationProjection {
    let mut runtime_sources = vec![EXPLANATION_RUNTIME_SOURCE_SESSION_STATE];
    if view.authored_input_summary.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_AUTHORED_INPUT);
    }
    if view.context_summary.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_CONTEXT);
    }
    if view.latest_selection_reason.is_some() || view.latest_validation_status.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_DECISION_TIMELINE);
    }
    if view.latest_review_headline.is_some() || view.latest_review_outcome.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_REVIEW_TIMELINE);
    }
    if view.latest_reasoning_profile.is_some() {
        runtime_sources.push(EXPLANATION_RUNTIME_SOURCE_REASONING_PROFILE);
    }

    let mut canon_sources = Vec::new();
    if view.latest_governance_packet_ref.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_GOVERNANCE_PACKET);
    }
    if view.latest_governance_approval_provenance.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_APPROVAL_PROVENANCE);
    }
    if view.latest_governance_decision.is_some() || view.latest_governance_reason.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_GOVERNANCE_DECISION);
    }
    if view.governance_next_action.is_some() {
        canon_sources.push(EXPLANATION_CANON_SOURCE_GOVERNANCE_ACTION);
    }

    let clarification_missing_fields =
        view.clarification_missing_fields.clone().unwrap_or_default();

    let missing_sources = build_missing_sources(
        canon_sources.is_empty(),
        view.context_staleness_reason.is_some(),
        !clarification_missing_fields.is_empty(),
    );

    let why_summary = reasoning_projection_why_summary(view.latest_reasoning_profile.as_ref())
        .or_else(|| view.planning_rationale.clone())
        .or_else(|| view.latest_selection_reason.clone())
        .or_else(|| view.goal.clone())
        .unwrap_or_else(|| view.explanation.clone());

    let risk_summary = if let Some(reasoning_risk) =
        reasoning_projection_risk_summary(view.latest_reasoning_profile.as_ref())
    {
        reasoning_risk
    } else if let Some(reason) = view.latest_exhaustion_reason.as_ref() {
        reason.clone()
    } else if let Some(reason) = view.latest_governance_blocked_reason.as_ref() {
        reason.clone()
    } else if let Some(reason) = view.context_staleness_reason.as_ref() {
        format!("stale context reduces confidence: {reason}")
    } else if canon_sources.is_empty() {
        EXPLANATION_RISK_CANON_GAP.to_string()
    } else if let Some(status) = view.latest_validation_status.as_ref() {
        status.clone()
    } else {
        EXPLANATION_RISK_NO_EXPLICIT_FAILURE.to_string()
    };

    let (evidence_summary, source_attribution) =
        build_evidence_and_attribution(&runtime_sources, &canon_sources, &missing_sources);

    let fallback_disclosure = explanation_fallback_disclosure(
        canon_sources.is_empty(),
        view.context_staleness_reason.as_deref(),
        &clarification_missing_fields,
    );

    let confidence_level =
        reasoning_projection_confidence_level(view.latest_reasoning_profile.as_ref())
            .unwrap_or_else(|| {
                explanation_confidence_level(
                    view.latest_exhaustion_reason.is_some()
                        || view.latest_governance_blocked_reason.is_some(),
                    canon_sources.is_empty(),
                    view.context_staleness_reason.is_some(),
                    !clarification_missing_fields.is_empty(),
                )
            });

    let next_best_action = reasoning_projection_next_action(view.latest_reasoning_profile.as_ref())
        .or_else(|| view.governance_next_action.clone())
        .or_else(|| view.next_command.clone())
        .or_else(|| view.workflow_next_action.clone())
        .unwrap_or_else(|| view.explanation.clone());

    ExplanationProjection {
        why_summary,
        risk_summary,
        evidence_summary,
        source_attribution,
        fallback_disclosure,
        confidence_level,
        next_best_action,
    }
}

fn explanation_assumption_entries(
    advanced_context: Option<&AdvancedContextProjection>,
) -> Vec<ExplanationAssumptionEntry> {
    let Some(advanced_context) = advanced_context else {
        return Vec::new();
    };

    advanced_context
        .relationships
        .iter()
        .map(|relationship| ExplanationAssumptionEntry {
            category: explanation_assumption_category(relationship.relationship_kind),
            subject_ref: relationship.subject_ref.clone(),
            status: explanation_assumption_status(relationship.relationship_kind),
            source: explanation_assumption_source(
                advanced_context,
                relationship.supporting_candidate_ids.as_slice(),
            ),
            risk: explanation_assumption_risk(
                relationship.relationship_kind,
                relationship.credibility_state,
            ),
            explanation: relationship.explanation.clone(),
        })
        .collect()
}

fn explanation_hidden_impact_entries(
    advanced_context: Option<&AdvancedContextProjection>,
) -> Vec<ExplanationHiddenImpactEntry> {
    let Some(advanced_context) = advanced_context else {
        return Vec::new();
    };

    advanced_context
        .impact_findings
        .iter()
        .map(|finding| ExplanationHiddenImpactEntry {
            group: explanation_hidden_impact_group(finding.finding_kind),
            label: explanation_hidden_impact_label(finding.finding_kind),
            subject_ref: finding.subject_ref.clone(),
            status: explanation_hidden_impact_status(finding.status),
            severity: explanation_hidden_impact_severity(finding.severity),
            follow_up: finding.recommended_follow_up.clone(),
        })
        .collect()
}

fn explanation_assumption_category(kind: RelationshipKind) -> &'static str {
    match kind {
        RelationshipKind::AffectsSystem | RelationshipKind::ExposesContract => {
            EXPLANATION_ASSUMPTION_CATEGORY_ARCHITECTURE
        }
        RelationshipKind::AffectsDomain => EXPLANATION_ASSUMPTION_CATEGORY_DOMAIN,
        RelationshipKind::SuggestsReviewer => EXPLANATION_ASSUMPTION_CATEGORY_GOVERNANCE,
        RelationshipKind::SupportsRisk => EXPLANATION_ASSUMPTION_CATEGORY_IMPLEMENTATION,
        RelationshipKind::ExercisesTest | RelationshipKind::RequiresEvidence => {
            EXPLANATION_ASSUMPTION_CATEGORY_VALIDATION
        }
    }
}

fn explanation_assumption_status(kind: RelationshipKind) -> &'static str {
    match kind {
        RelationshipKind::ExercisesTest | RelationshipKind::ExposesContract => {
            EXPLANATION_ASSUMPTION_STATUS_EXPLICIT
        }
        RelationshipKind::RequiresEvidence => EXPLANATION_ASSUMPTION_STATUS_MISSING,
        RelationshipKind::AffectsSystem
        | RelationshipKind::AffectsDomain
        | RelationshipKind::SuggestsReviewer
        | RelationshipKind::SupportsRisk => EXPLANATION_ASSUMPTION_STATUS_INFERRED,
    }
}

fn explanation_assumption_source(
    advanced_context: &AdvancedContextProjection,
    supporting_candidate_ids: &[String],
) -> &'static str {
    for candidate_id in supporting_candidate_ids {
        if let Some(source_kind) = advanced_context
            .selected_evidence
            .iter()
            .chain(advanced_context.rejected_candidates.iter())
            .find(|candidate| candidate.candidate_id == *candidate_id)
            .map(|candidate| candidate.source_kind)
        {
            return match source_kind {
                RetrievalSourceKind::CanonArtifact => EXPLANATION_ASSUMPTION_SOURCE_CANON,
                RetrievalSourceKind::Trace
                | RetrievalSourceKind::ReviewFinding
                | RetrievalSourceKind::VerificationEvidence => EXPLANATION_ASSUMPTION_SOURCE_TRACE,
                RetrievalSourceKind::WorkspaceFile | RetrievalSourceKind::ProjectMemory => {
                    EXPLANATION_ASSUMPTION_SOURCE_WORKSPACE
                }
            };
        }
    }

    EXPLANATION_ASSUMPTION_SOURCE_TRACE
}

fn explanation_assumption_risk(
    kind: RelationshipKind,
    credibility_state: RelationshipCredibilityState,
) -> &'static str {
    if matches!(kind, RelationshipKind::RequiresEvidence) {
        return EXPLANATION_ASSUMPTION_RISK_HIGH;
    }

    match credibility_state {
        RelationshipCredibilityState::Credible => EXPLANATION_ASSUMPTION_RISK_LOW,
        RelationshipCredibilityState::Tentative => EXPLANATION_ASSUMPTION_RISK_MEDIUM,
        RelationshipCredibilityState::Insufficient => EXPLANATION_ASSUMPTION_RISK_HIGH,
    }
}

fn explanation_assumption_summary(entries: &[ExplanationAssumptionEntry]) -> String {
    explanation_group_summary(
        entries.iter().map(|entry| entry.category),
        &[
            EXPLANATION_ASSUMPTION_CATEGORY_DOMAIN,
            EXPLANATION_ASSUMPTION_CATEGORY_ARCHITECTURE,
            EXPLANATION_ASSUMPTION_CATEGORY_IMPLEMENTATION,
            EXPLANATION_ASSUMPTION_CATEGORY_VALIDATION,
            EXPLANATION_ASSUMPTION_CATEGORY_GOVERNANCE,
        ],
    )
}

fn explanation_assumption_group_lines(entries: &[ExplanationAssumptionEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{EXPLANATION_LABEL_ASSUMPTION_GROUP}: {} -> {} [{}] source={} risk={} {}",
                entry.category,
                entry.subject_ref,
                entry.status,
                entry.source,
                entry.risk,
                entry.explanation
            )
        })
        .collect()
}

fn explanation_hidden_impact_group(kind: ImpactFindingKind) -> &'static str {
    match kind {
        ImpactFindingKind::AffectedSystem => EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS,
        ImpactFindingKind::AffectedDomain => EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS,
        ImpactFindingKind::MissingTest => EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS,
        ImpactFindingKind::ContractExposure => EXPLANATION_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES,
        ImpactFindingKind::ReviewerGap => EXPLANATION_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS,
        ImpactFindingKind::EvidenceGap => EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE,
    }
}

fn explanation_hidden_impact_label(kind: ImpactFindingKind) -> &'static str {
    match kind {
        ImpactFindingKind::AffectedSystem => EXPLANATION_HIDDEN_IMPACT_LABEL_AFFECTED_SYSTEMS,
        ImpactFindingKind::AffectedDomain => EXPLANATION_HIDDEN_IMPACT_LABEL_AFFECTED_DOMAINS,
        ImpactFindingKind::MissingTest => EXPLANATION_HIDDEN_IMPACT_LABEL_MISSING_TESTS,
        ImpactFindingKind::ContractExposure => EXPLANATION_HIDDEN_IMPACT_LABEL_CONTRACT_EXPOSURES,
        ImpactFindingKind::ReviewerGap => EXPLANATION_HIDDEN_IMPACT_LABEL_REQUIRED_REVIEWERS,
        ImpactFindingKind::EvidenceGap => EXPLANATION_HIDDEN_IMPACT_LABEL_MISSING_EVIDENCE,
    }
}

fn explanation_hidden_impact_status(status: ImpactFindingStatus) -> &'static str {
    match status {
        ImpactFindingStatus::Open => "open",
        ImpactFindingStatus::Acknowledged => "acknowledged",
        ImpactFindingStatus::Resolved => "resolved",
        ImpactFindingStatus::Invalidated => "invalidated",
    }
}

fn explanation_hidden_impact_severity(severity: ImpactFindingSeverity) -> &'static str {
    match severity {
        ImpactFindingSeverity::Low => EXPLANATION_ASSUMPTION_RISK_LOW,
        ImpactFindingSeverity::Medium => EXPLANATION_ASSUMPTION_RISK_MEDIUM,
        ImpactFindingSeverity::High => EXPLANATION_ASSUMPTION_RISK_HIGH,
    }
}

fn explanation_hidden_impact_summary(entries: &[ExplanationHiddenImpactEntry]) -> String {
    explanation_group_summary(
        entries.iter().map(|entry| entry.group),
        &[
            EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS,
            EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS,
            EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS,
            EXPLANATION_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES,
            EXPLANATION_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS,
            EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE,
        ],
    )
}

fn explanation_hidden_impact_lines(entries: &[ExplanationHiddenImpactEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{}: {} [{}/{}] {}",
                entry.label, entry.subject_ref, entry.status, entry.severity, entry.follow_up
            )
        })
        .collect()
}

fn explanation_group_summary<'a>(
    entries: impl Iterator<Item = &'a str>,
    ordered_groups: &[&str],
) -> String {
    let collected = entries.collect::<Vec<_>>();
    let mut parts = Vec::new();
    for group in ordered_groups {
        let count = collected.iter().filter(|entry| **entry == *group).count();
        if count > 0 {
            parts.push(format!("{group}({count})"));
        }
    }

    if parts.is_empty() { format!("{EXPLANATION_NONE}(0)") } else { parts.join(", ") }
}

fn explanation_hidden_impact_fallback_disclosure(
    advanced_context: Option<&AdvancedContextProjection>,
) -> Option<String> {
    let advanced_context = advanced_context?;
    if advanced_context.semantic_policy_state == SemanticPolicyState::Local
        && advanced_context.semantic_capability_state != SemanticCapabilityState::Ready
    {
        let reason = advanced_context
            .terminal_reason
            .as_deref()
            .unwrap_or("semantic acceleration is unavailable; using baseline structured retrieval");
        return Some(format!("higher-order impact inference is unavailable because {reason}"));
    }

    None
}

fn explanation_highest_priority_impact(
    entries: &[ExplanationHiddenImpactEntry],
) -> Option<&ExplanationHiddenImpactEntry> {
    entries.iter().max_by_key(|entry| {
        (
            usize::from(entry.status == "open"),
            match entry.severity {
                EXPLANATION_ASSUMPTION_RISK_HIGH => 3,
                EXPLANATION_ASSUMPTION_RISK_MEDIUM => 2,
                _ => 1,
            },
        )
    })
}

fn explanation_weakest_assumption(entries: &[ExplanationAssumptionEntry]) -> String {
    entries
        .iter()
        .max_by_key(|entry| {
            (
                usize::from(entry.status == EXPLANATION_ASSUMPTION_STATUS_MISSING),
                match entry.risk {
                    EXPLANATION_ASSUMPTION_RISK_HIGH => 3,
                    EXPLANATION_ASSUMPTION_RISK_MEDIUM => 2,
                    _ => 1,
                },
            )
        })
        .map(|entry| {
            format!("{} -> {} [{}/{}]", entry.category, entry.subject_ref, entry.status, entry.risk)
        })
        .unwrap_or_else(|| EXPLANATION_WEAK_ASSUMPTION_NONE.to_string())
}

fn explanation_challenge_strongest_objection(
    impacts: &[ExplanationHiddenImpactEntry],
    hidden_impact_fallback_disclosure: Option<&str>,
) -> String {
    if let Some(impact) = explanation_highest_priority_impact(impacts) {
        return match impact.group {
            EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS => {
                format!("missing test coverage is still open for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE => {
                format!("required evidence is still missing for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS => {
                format!("required reviewer coverage is still missing for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES => {
                format!("contract exposure still needs review for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS => {
                format!("system impact extends beyond the current slice for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS => {
                format!("domain impact extends beyond the current slice for {}", impact.subject_ref)
            }
            _ => impact.follow_up.clone(),
        };
    }

    hidden_impact_fallback_disclosure.unwrap_or(EXPLANATION_RISK_NO_EXPLICIT_FAILURE).to_string()
}

fn explanation_challenge_missing_evidence(
    impacts: &[ExplanationHiddenImpactEntry],
    hidden_impact_fallback_disclosure: Option<&str>,
) -> String {
    let evidence_refs = impacts
        .iter()
        .filter(|impact| {
            impact.group == EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS
                || impact.group == EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE
        })
        .map(|impact| impact.subject_ref.clone())
        .collect::<Vec<_>>();
    if !evidence_refs.is_empty() {
        return evidence_refs.join(", ");
    }
    hidden_impact_fallback_disclosure
        .map(str::to_string)
        .unwrap_or_else(|| EXPLANATION_NONE.to_string())
}

fn explanation_challenge_failure_mode(
    impacts: &[ExplanationHiddenImpactEntry],
    hidden_impact_fallback_disclosure: Option<&str>,
) -> String {
    if let Some(impact) = explanation_highest_priority_impact(impacts) {
        return match impact.group {
            EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS => {
                format!(
                    "bounded validation can regress without a focused test for {}",
                    impact.subject_ref
                )
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_EVIDENCE => {
                format!("the plan can proceed without required evidence for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_REQUIRED_REVIEWERS => {
                format!("review can miss critical dissent for {}", impact.subject_ref)
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_CONTRACT_EXPOSURES => {
                format!(
                    "downstream consumers can break if {} changes without review",
                    impact.subject_ref
                )
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_SYSTEMS => {
                format!(
                    "cross-system impact can escape the bounded slice for {}",
                    impact.subject_ref
                )
            }
            EXPLANATION_HIDDEN_IMPACT_GROUP_AFFECTED_DOMAINS => {
                format!("domain invariants can drift for {}", impact.subject_ref)
            }
            _ => impact.follow_up.clone(),
        };
    }

    hidden_impact_fallback_disclosure
        .map(str::to_string)
        .unwrap_or_else(|| EXPLANATION_RISK_NO_EXPLICIT_FAILURE.to_string())
}

fn explanation_council_required(governance_present: bool) -> &'static str {
    if governance_present {
        EXPLANATION_COUNCIL_REQUIRED_YES
    } else {
        EXPLANATION_COUNCIL_REQUIRED_NO
    }
}

pub(crate) fn explanation_cognitive_projection_for_trace_summary(
    summary: &TraceSummaryView,
    next_command: &str,
    fallback_disclosure: &str,
) -> ExplanationCognitiveProjection {
    let assumptions = explanation_assumption_entries(summary.advanced_context.as_ref());
    let impacts = explanation_hidden_impact_entries(summary.advanced_context.as_ref());
    let hidden_impact_fallback_disclosure =
        explanation_hidden_impact_fallback_disclosure(summary.advanced_context.as_ref());
    let governance_present = !summary.governance_timeline.is_empty()
        || summary.governance_reason.is_some()
        || summary.governance_approval_provenance.is_some()
        || summary.governance_next_action.is_some();
    let stage_text = if !summary.executed_steps.is_empty() {
        format!("{} step(s)", summary.executed_steps.len())
    } else {
        "trace_inspect".to_string()
    };
    let reasoning_why = reasoning_projection_why_summary(summary.reasoning_profile.as_ref())
        .unwrap_or_else(|| "none".to_string());

    ExplanationCognitiveProjection {
        assumptions_summary: explanation_assumption_summary(&assumptions),
        assumption_groups: explanation_assumption_group_lines(&assumptions),
        hidden_impact_summary: explanation_hidden_impact_summary(&impacts),
        hidden_impact_lines: explanation_hidden_impact_lines(&impacts),
        hidden_impact_fallback_disclosure: hidden_impact_fallback_disclosure.clone(),
        challenge_strongest_objection: explanation_challenge_strongest_objection(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_weakest_assumption: explanation_weakest_assumption(&assumptions),
        challenge_missing_evidence: explanation_challenge_missing_evidence(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_failure_mode: explanation_challenge_failure_mode(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_required_review: if governance_present {
            "governance timeline remains authoritative".to_string()
        } else {
            EXPLANATION_REVIEW_RUNTIME_ONLY.to_string()
        },
        challenge_council_required: explanation_council_required(governance_present),
        explain_plan_summary: format!(
            "goal={}; stages={stage_text}; reasoning={reasoning_why}; risks={}; assumptions={}",
            summary.goal,
            explanation_hidden_impact_summary(&impacts),
            explanation_assumption_summary(&assumptions)
        ),
        explain_plan_validation: reasoning_projection_next_action(
            summary.reasoning_profile.as_ref(),
        )
        .or_else(|| {
            impacts
                .iter()
                .find(|impact| impact.group == EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS)
                .map(|impact| impact.follow_up.clone())
        })
        .unwrap_or_else(|| next_command.to_string()),
        explain_plan_governance: if let Some(reasoning_summary) =
            reasoning_projection_governance_summary(summary.reasoning_profile.as_ref())
        {
            reasoning_summary
        } else if governance_present {
            summary
                .governance_next_action
                .clone()
                .or_else(|| summary.governance_approval_provenance.clone())
                .or_else(|| summary.governance_reason.clone())
                .unwrap_or_else(|| fallback_disclosure.to_string())
        } else {
            fallback_disclosure.to_string()
        },
        explain_plan_recovery: reasoning_projection_next_action(summary.reasoning_profile.as_ref())
            .or_else(|| summary.latest_checkpoint_restore_command.clone())
            .unwrap_or_else(|| next_command.to_string()),
    }
}

pub(crate) fn explanation_cognitive_projection_for_session_status(
    view: &SessionStatusView,
    fallback_disclosure: &str,
) -> ExplanationCognitiveProjection {
    let assumptions = explanation_assumption_entries(view.advanced_context.as_ref());
    let impacts = explanation_hidden_impact_entries(view.advanced_context.as_ref());
    let hidden_impact_fallback_disclosure =
        explanation_hidden_impact_fallback_disclosure(view.advanced_context.as_ref());
    let governance_present = view.latest_governance_packet_ref.is_some()
        || view.latest_governance_runtime.is_some()
        || view.latest_governance_decision.is_some()
        || view.latest_governance_reason.is_some()
        || view.governance_next_action.is_some();
    let stage_text = match (view.active_flow.as_deref(), view.flow_state.as_deref()) {
        (Some(flow), Some(state)) => format!("{flow}/{state}"),
        (Some(flow), None) => flow.to_string(),
        _ => view.current_stage_id.clone().unwrap_or_else(|| "session_state".to_string()),
    };
    let goal = view.goal.clone().unwrap_or_else(|| view.explanation.clone());
    let reasoning_why = reasoning_projection_why_summary(view.latest_reasoning_profile.as_ref())
        .unwrap_or_else(|| "none".to_string());

    ExplanationCognitiveProjection {
        assumptions_summary: explanation_assumption_summary(&assumptions),
        assumption_groups: explanation_assumption_group_lines(&assumptions),
        hidden_impact_summary: explanation_hidden_impact_summary(&impacts),
        hidden_impact_lines: explanation_hidden_impact_lines(&impacts),
        hidden_impact_fallback_disclosure: hidden_impact_fallback_disclosure.clone(),
        challenge_strongest_objection: explanation_challenge_strongest_objection(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_weakest_assumption: explanation_weakest_assumption(&assumptions),
        challenge_missing_evidence: explanation_challenge_missing_evidence(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_failure_mode: explanation_challenge_failure_mode(
            &impacts,
            hidden_impact_fallback_disclosure.as_deref(),
        ),
        challenge_required_review: if let Some(packet_ref) =
            view.latest_governance_packet_ref.as_deref()
        {
            format!("governance packet {packet_ref} remains authoritative")
        } else if governance_present {
            "governance runtime remains authoritative".to_string()
        } else {
            EXPLANATION_REVIEW_RUNTIME_ONLY.to_string()
        },
        challenge_council_required: explanation_council_required(governance_present),
        explain_plan_summary: format!(
            "goal={goal}; stages={stage_text}; reasoning={reasoning_why}; risks={}; assumptions={}",
            explanation_hidden_impact_summary(&impacts),
            explanation_assumption_summary(&assumptions)
        ),
        explain_plan_validation: reasoning_projection_next_action(
            view.latest_reasoning_profile.as_ref(),
        )
        .or_else(|| view.verification_strategy.clone())
        .or_else(|| {
            impacts
                .iter()
                .find(|impact| impact.group == EXPLANATION_HIDDEN_IMPACT_GROUP_MISSING_TESTS)
                .map(|impact| impact.follow_up.clone())
        })
        .or_else(|| view.next_command.clone())
        .unwrap_or_else(|| view.explanation.clone()),
        explain_plan_governance: if let Some(reasoning_summary) =
            reasoning_projection_governance_summary(view.latest_reasoning_profile.as_ref())
        {
            reasoning_summary
        } else if let Some(packet_ref) = view.latest_governance_packet_ref.as_deref() {
            format!(
                "governance_packet={packet_ref}; council_required={}",
                explanation_council_required(governance_present)
            )
        } else if governance_present {
            view.latest_governance_decision
                .clone()
                .or_else(|| view.governance_next_action.clone())
                .unwrap_or_else(|| fallback_disclosure.to_string())
        } else {
            fallback_disclosure.to_string()
        },
        explain_plan_recovery: reasoning_projection_next_action(
            view.latest_reasoning_profile.as_ref(),
        )
        .or_else(|| view.latest_checkpoint_restore_command.clone())
        .or_else(|| view.next_command.clone())
        .unwrap_or_else(|| view.explanation.clone()),
    }
}
