use super::host::{push_output_section, stdout_presentation};
use super::*;

fn diagnostic_follow_up_actions(report: &DiagnosticsReport) -> Vec<String> {
    if !report.ready {
        return Vec::new();
    }

    match report.subject {
        crate::cli::diagnostics::DiagnosticsSubject::Workspace => vec![format!(
            "- capture a goal: boundline goal --workspace {} --goal <goal>",
            report.workspace_ref.as_deref().unwrap_or("<workspace>")
        )],
        crate::cli::diagnostics::DiagnosticsSubject::Install => {
            vec!["- verify a workspace next: boundline doctor --workspace <workspace>".to_string()]
        }
    }
}

pub fn render_diagnostics(report: &DiagnosticsReport) -> String {
    let readiness = if report.ready { "ready" } else { "not ready" };
    let subject = match report.subject {
        crate::cli::diagnostics::DiagnosticsSubject::Workspace => format!(
            "workspace {}",
            report.workspace_ref.as_deref().unwrap_or("<unknown-workspace>")
        ),
        crate::cli::diagnostics::DiagnosticsSubject::Install => format!(
            "installation {}",
            report.installation_ref.as_deref().unwrap_or("<current-machine>")
        ),
    };
    let presentation = stdout_presentation();
    let mut lines = vec![format!("doctor: {readiness} for {subject}")];
    let mut summary_lines = vec![
        "- assistant_hint: Diagnostic output format is optimized for chat parsing.".to_string(),
    ];

    if let Some(boundline_version) = &report.boundline_version {
        summary_lines.push(format!("- boundline_version: {boundline_version}"));
    }
    if let Some(supported_canon_version) = &report.supported_canon_version {
        summary_lines.push(format!("- supported_canon_version: {supported_canon_version}"));
    }
    if let Some(companion_state) = report.companion_state {
        summary_lines.push(format!("- companion_state: {companion_state}"));
    }
    if !report.channel_candidates.is_empty() {
        summary_lines
            .push(format!("- channel_candidates: {}", report.channel_candidates.join(", ")));
    }
    push_output_section(&mut lines, presentation, "summary", summary_lines);

    let check_lines = report
        .checks
        .iter()
        .map(|check| {
            let status = match check.status {
                DiagnosticsStatus::Passed => "passed",
                DiagnosticsStatus::Advisory => "advisory",
                DiagnosticsStatus::Failed => "failed",
            };
            format!("- {}: {} - {}", check.name, status, check.message)
        })
        .collect::<Vec<_>>();
    push_output_section(&mut lines, presentation, "checks", check_lines);

    let mut action_lines = Vec::new();
    for action in &report.suggested_actions {
        let rendered = format!("- {action}");
        if !action_lines.iter().any(|existing| existing == &rendered) {
            action_lines.push(rendered);
        }
    }
    for action in diagnostic_follow_up_actions(report) {
        if !action_lines.iter().any(|existing| existing == &action) {
            action_lines.push(action);
        }
    }
    push_output_section(&mut lines, presentation, "actions", action_lines);

    lines.join("\n")
}

pub(crate) fn validation_line_from_event(payload: &Value) -> Option<String> {
    let validation =
        payload.get("output").and_then(|output| output.get("validation")).or_else(|| {
            payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
        })?;
    let command = validation.get("command").and_then(Value::as_str).unwrap_or("validation");
    let succeeded = validation.get("succeeded").and_then(Value::as_bool).unwrap_or(false);
    let exit_code =
        validation.get("exit_code").and_then(Value::as_i64).unwrap_or(UNKNOWN_VALIDATION_EXIT_CODE);
    Some(format!(
        "validation: {} ({command}, exit_code={exit_code})",
        if succeeded { "passed" } else { "failed" }
    ))
}

pub(crate) fn review_event_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::ReviewStarted => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(Value::as_str)
            .map(|trigger| format!("review_trigger: {trigger}")),
        TraceEventType::ReviewTriggerIgnored => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(Value::as_str)
            .map(|trigger| format!("review_trigger_ignored: {trigger}")),
        TraceEventType::ReviewerCompleted => reviewer_event_line(payload),
        TraceEventType::ReviewVoteResolved => payload
            .get(KEY_SUMMARY)
            .and_then(Value::as_str)
            .map(|summary| format!("review_vote: {summary}"))
            .or_else(|| {
                payload.get(KEY_VOTE_RESOLUTION).map(|resolution| {
                    format!(
                        "review_vote: {}",
                        serde_json::to_string(resolution).unwrap_or_default()
                    )
                })
            }),
        TraceEventType::ReviewAdjudicated => {
            reviewer_event_line(payload).map(|line| format!("review_adjudication: {line}"))
        }
        TraceEventType::ReviewTerminalRecorded => payload
            .get(KEY_REVIEW_OUTCOME)
            .and_then(Value::as_str)
            .map(|outcome| format!("review_outcome: {outcome}"))
            .or_else(|| {
                payload
                    .get(KEY_FAILURE_REASON)
                    .and_then(Value::as_str)
                    .map(|reason| format!("review_reason: {reason}"))
            }),
        _ => None,
    }
}

pub(crate) fn governance_event_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::GovernanceSelected => Some(format!(
            "governance_selected: {} -> {}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload.get("selected_runtime").and_then(Value::as_str).unwrap_or("unknown-runtime")
        )),
        TraceEventType::GovernanceStarted => Some(format!(
            "governance_started: {}{}{}{}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload
                .get("canon_mode")
                .and_then(Value::as_str)
                .map(|mode| format!(" ({mode})"))
                .unwrap_or_default(),
            payload
                .get("run_ref")
                .and_then(Value::as_str)
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceDecisionRecorded => payload
            .get("selected_action")
            .and_then(Value::as_str)
            .map(|action| format!("governance_decision: {action}"))
            .or_else(|| {
                payload
                    .get("blocked_reason")
                    .and_then(Value::as_str)
                    .map(|reason| format!("governance_decision_blocked: {reason}"))
            }),
        TraceEventType::GovernanceAwaitingApproval => Some(format!(
            "governance_awaiting_approval: {} ({}){}{}",
            payload.get("stage_key").and_then(Value::as_str).unwrap_or("unknown-stage"),
            payload.get("approval_state").and_then(Value::as_str).unwrap_or("unknown"),
            payload
                .get("run_ref")
                .and_then(Value::as_str)
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceCompleted => Some(format!(
            "governance_completed: {}{}{}",
            payload.get("headline").and_then(Value::as_str).unwrap_or("governed packet ready"),
            payload
                .get("packet_ref")
                .and_then(Value::as_str)
                .map(|packet_ref| format!(" [{packet_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceBlocked => Some(format!(
            "governance_blocked: {}{}",
            payload.get("reason").and_then(Value::as_str).unwrap_or("blocked"),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernancePacketRejected => Some(format!(
            "governance_packet_rejected: {}{}",
            payload.get("reason").and_then(Value::as_str).unwrap_or("packet rejected"),
            governance_packet_provenance_suffix(payload)
        )),
        _ => None,
    }
}

fn governance_packet_provenance_suffix(payload: &Value) -> String {
    governance_packet_provenance_text(
        payload.get("packet_source_stage").and_then(Value::as_str),
        payload.get("packet_binding_reason").and_then(Value::as_str),
    )
    .map(|provenance| format!(" from {provenance}"))
    .unwrap_or_default()
}

pub(crate) fn reviewer_event_line(payload: &Value) -> Option<String> {
    let reviewer_id = payload.get(KEY_REVIEWER_ID).and_then(Value::as_str)?;

    if let Some(finding) = payload.get(KEY_FINDING) {
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        let role = payload.get(KEY_REVIEWER_ROLE).and_then(Value::as_str);
        return Some(match role {
            Some(role) => format!("reviewer {reviewer_id} ({role}) {disposition}: {summary}"),
            None => format!("reviewer {reviewer_id} {disposition}: {summary}"),
        });
    }

    payload
        .get(KEY_FAILURE_REASON)
        .and_then(Value::as_str)
        .map(|reason| format!("reviewer {reviewer_id} failed: {reason}"))
}
