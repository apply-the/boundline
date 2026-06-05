use super::{AdvancedContextProjection, RetrievedEvidenceCandidate, SemanticTraceRecord};

pub(crate) fn push_context_projection_lines(
    lines: &mut Vec<String>,
    context_summary: Option<&str>,
    context_credibility: Option<&str>,
    context_primary_inputs: &[String],
    context_provenance: &[String],
    context_staleness_reason: Option<&str>,
) {
    if let Some(context_summary) = context_summary {
        lines.push(format!("context_summary: {context_summary}"));
    }

    if let Some(context_credibility) = context_credibility {
        lines.push(format!("context_credibility: {context_credibility}"));
    }

    if !context_primary_inputs.is_empty() {
        lines.push(format!("context_primary_inputs: {}", context_primary_inputs.join(", ")));
    }

    if !context_provenance.is_empty() {
        lines.push(format!("context_provenance: {}", context_provenance.join(" | ")));
    }

    if let Some(context_staleness_reason) = context_staleness_reason {
        lines.push(format!("context_staleness_reason: {context_staleness_reason}"));
    }
}

// Render the compact advanced-context projection so `status` and `inspect`
// can explain retrieval state without hiding the runtime reasoning.
pub(crate) fn push_advanced_context_lines(
    lines: &mut Vec<String>,
    advanced_context: Option<&AdvancedContextProjection>,
) {
    let Some(advanced_context) = advanced_context else {
        return;
    };

    lines.push(format!("retrieval_mode: {}", advanced_context.retrieval_mode.as_str()));
    lines.push(format!("retrieval_state: {}", advanced_context.retrieval_state.as_str()));
    lines.push(format!("retrieval_authority_order: {}", advanced_context.authority_order_text()));
    lines.push(format!(
        "retrieval_index_state: {}",
        advanced_context.retrieval_index_state.as_str()
    ));
    lines.push(format!(
        "semantic_policy_state: {}",
        advanced_context.semantic_policy_state.as_str()
    ));
    lines.push(format!(
        "semantic_capability_state: {}",
        advanced_context.semantic_capability_contract_label()
    ));
    if let Some(recovery_guidance) = retrieval_recovery_guidance(advanced_context) {
        lines.push(format!("retrieval_recovery_guidance: {recovery_guidance}"));
    }
    lines.push(format!("semantic_engine: {}", advanced_context.semantic_engine().as_str()));
    lines.push(format!("hybrid_outcome: {}", advanced_context.hybrid_outcome.as_str()));
    lines.push(format!("vector_query_count: {}", advanced_context.vector_query_count()));
    lines.push(format!(
        "vector_candidates_returned: {}",
        advanced_context.vector_candidates_returned()
    ));
    if let Some(fallback_reason) = advanced_context.semantic_fallback_reason() {
        lines.push(format!("semantic_fallback_reason: {fallback_reason}"));
    }
    if let Some(terminal_reason) = advanced_context.terminal_reason.as_deref() {
        lines.push(format!("retrieval_terminal_reason: {terminal_reason}"));
    }
    lines.push(format!("selected_evidence_count: {}", advanced_context.selected_evidence_count()));
    lines.push(format!("semantic_selected_count: {}", advanced_context.semantic_selected_count()));
    lines.push(format!("semantic_rejected_count: {}", advanced_context.semantic_rejected_count()));
    lines.push(format!("impact_finding_count: {}", advanced_context.impact_finding_count()));
    if let Some(repository_map_state) = advanced_context.repository_map_state {
        lines.push(format!("repository_map_state: {}", repository_map_state.as_str()));
    }
    if let Some(snapshot_cache_state) = advanced_context.snapshot_cache_state {
        lines.push(format!("snapshot_cache_state: {}", snapshot_cache_state.as_str()));
    }
    if !advanced_context.context_pack_entries.is_empty() {
        lines.push(format!(
            "context_pack_entry_count: {}",
            advanced_context.context_pack_entries.len()
        ));
    }
    if !advanced_context.omission_findings.is_empty() {
        lines.push(format!(
            "context_omission_finding_count: {}",
            advanced_context.omission_findings.len()
        ));
    }
    if !advanced_context.patch_safe_edit_attempts.is_empty() {
        lines.push(format!(
            "patch_safe_edit_attempt_count: {}",
            advanced_context.patch_safe_edit_attempts.len()
        ));
    }

    for candidate in &advanced_context.selected_evidence {
        lines.push(format_candidate_line("selected_evidence", candidate));
    }

    for candidate in &advanced_context.rejected_candidates {
        lines.push(format_candidate_line("rejected_candidate", candidate));
    }

    for record in &advanced_context.semantic_trace_records {
        lines.push(format_semantic_trace_line(record));
    }

    for relationship in &advanced_context.relationships {
        lines.push(format!(
            "relationship: {} [{}] {}",
            relationship.subject_ref,
            relationship.relationship_kind.as_str(),
            relationship.explanation
        ));
    }

    for finding in &advanced_context.impact_findings {
        lines.push(format!(
            "impact_finding: {} [{}] {}",
            finding.subject_ref,
            finding.finding_kind.as_str(),
            finding.recommended_follow_up
        ));
    }

    for entry in &advanced_context.context_pack_entries {
        let mut line = format!(
            "context_entry: {} [{}] tier={} mode={} required={} {}",
            entry.source_ref,
            entry.source_kind.as_str(),
            entry.fidelity_tier.as_str(),
            entry.inclusion_mode.as_str(),
            entry.required_for_admission,
            entry.reason
        );
        if let Some(ranking_rationale) = entry.ranking_rationale.as_deref() {
            line.push_str(&format!(" ranking={ranking_rationale}"));
        }
        if let Some(digest_ref) = entry.digest_ref.as_ref() {
            line.push_str(&format!(
                " digest={} resolve_path={}",
                digest_ref.digest, digest_ref.resolve_path
            ));
        }
        lines.push(line);
    }

    for finding in &advanced_context.omission_findings {
        let mut line = format!(
            "context_omission: {} [{}] {}",
            finding.candidate_ref,
            finding.severity.as_str(),
            finding.message
        );
        line.push_str(&format!(" code={}", finding.reason_code));
        if let Some(required_fidelity) = finding.required_fidelity {
            line.push_str(&format!(" required_fidelity={}", required_fidelity.as_str()));
        }
        if let Some(observed_mode) = finding.observed_mode {
            line.push_str(&format!(" observed_mode={}", observed_mode.as_str()));
        }
        lines.push(line);
    }

    for attempt in &advanced_context.patch_safe_edit_attempts {
        lines.push(format!(
            "patch_safe_edit: {} [{}] anchors={} verification={}",
            attempt.target_ref,
            attempt.result_state.as_str(),
            attempt.anchor_refs.join(", "),
            attempt.post_apply_verification.join(" | ")
        ));
    }
}

fn retrieval_recovery_guidance(
    advanced_context: &AdvancedContextProjection,
) -> Option<&'static str> {
    if matches!(
        advanced_context.semantic_capability_state,
        crate::domain::context_intelligence::SemanticCapabilityState::Degraded
            | crate::domain::context_intelligence::SemanticCapabilityState::Corrupt
    ) {
        return Some(
            "run boundline index doctor to inspect vector capability, hooks, and tracked-file hygiene",
        );
    }

    match advanced_context.retrieval_index_state {
        crate::domain::context_intelligence::RetrievalIndexState::Ready => None,
        crate::domain::context_intelligence::RetrievalIndexState::Missing
        | crate::domain::context_intelligence::RetrievalIndexState::Stale => Some(
            "run boundline index refresh in the target workspace before relying on semantic retrieval",
        ),
        crate::domain::context_intelligence::RetrievalIndexState::Incompatible
        | crate::domain::context_intelligence::RetrievalIndexState::Corrupt => {
            Some("run boundline index rebuild or boundline index doctor in the target workspace")
        }
        crate::domain::context_intelligence::RetrievalIndexState::Degraded
        | crate::domain::context_intelligence::RetrievalIndexState::SemanticUnavailable => Some(
            "run boundline index doctor to inspect vector capability, hooks, and tracked-file hygiene",
        ),
        crate::domain::context_intelligence::RetrievalIndexState::Building
        | crate::domain::context_intelligence::RetrievalIndexState::Insufficient => {
            Some("rerun boundline index status or refresh after the derived index is available")
        }
    }
}

fn format_candidate_line(prefix: &str, candidate: &RetrievedEvidenceCandidate) -> String {
    let mut line = format!(
        "{prefix}: {} [{}] origin={}{} {}",
        candidate.source_ref,
        candidate.source_kind.as_str(),
        candidate.match_origin.as_str(),
        candidate_score_suffix(candidate),
        candidate.selection_reason
    );
    if let (Some(contract_line), Some(provenance_ref)) = (
        candidate.canon_semantic_contract_line.as_deref(),
        candidate.canon_semantic_provenance_ref.as_deref(),
    ) {
        line.push_str(&format!(
            " canon_contract={} canon_provenance={}",
            contract_line, provenance_ref
        ));
    }
    if candidate.compatibility_state.as_str() != "compatible" {
        line.push_str(&format!(" compatibility={}", candidate.compatibility_state.as_str()));
    }
    line
}

fn format_semantic_trace_line(record: &SemanticTraceRecord) -> String {
    let mut line = format!("semantic_trace: {}", record.event_kind.as_str());
    if let Some(candidate_ref) = record.candidate_ref.as_deref() {
        line.push_str(&format!(" ref={candidate_ref}"));
    }
    if let Some(match_origin) = record.match_origin {
        line.push_str(&format!(" origin={}", match_origin.as_str()));
    }
    if let Some(compatibility_state) = record.compatibility_state {
        line.push_str(&format!(" compatibility={}", compatibility_state.as_str()));
    }
    if let Some(semantic_score) = record.semantic_score {
        line.push_str(&format!(" semantic_score={:.3}", semantic_score.as_raw()));
    }
    if let Some(artifact_class) = record.canon_artifact_class.as_deref() {
        line.push_str(&format!(" canon_artifact_class={artifact_class}"));
    }
    if let Some(contract_line) = record.canon_semantic_contract_line.as_deref() {
        line.push_str(&format!(" canon_contract={contract_line}"));
    }
    if let Some(boundary) = record.canon_semantic_provenance_boundary {
        line.push_str(&format!(" canon_boundary={}", boundary.as_str()));
    }
    if let Some(provenance_ref) = record.canon_semantic_provenance_ref.as_deref() {
        line.push_str(&format!(" canon_provenance={provenance_ref}"));
    }
    line.push(' ');
    line.push_str(&record.reason);
    line
}

fn candidate_score_suffix(candidate: &RetrievedEvidenceCandidate) -> String {
    let mut suffix = String::new();
    if let Some(lexical_score) = candidate.lexical_score {
        suffix.push_str(&format!(" lexical_score={:.3}", lexical_score.as_raw()));
    }
    if let Some(semantic_score) = candidate.semantic_score {
        suffix.push_str(&format!(" semantic_score={:.3}", semantic_score.as_raw()));
    }
    suffix
}
