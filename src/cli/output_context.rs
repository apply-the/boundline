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
        advanced_context.semantic_capability_state.as_str()
    ));
    lines.push(format!("hybrid_outcome: {}", advanced_context.hybrid_outcome.as_str()));
    if let Some(terminal_reason) = advanced_context.terminal_reason.as_deref() {
        lines.push(format!("retrieval_terminal_reason: {terminal_reason}"));
    }
    lines.push(format!("selected_evidence_count: {}", advanced_context.selected_evidence_count()));
    lines.push(format!("semantic_selected_count: {}", advanced_context.semantic_selected_count()));
    lines.push(format!("semantic_rejected_count: {}", advanced_context.semantic_rejected_count()));
    lines.push(format!("impact_finding_count: {}", advanced_context.impact_finding_count()));

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
