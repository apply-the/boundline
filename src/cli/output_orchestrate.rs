pub use crate::cli::orchestrate::OrchestrateCommandReport;

pub fn render_human_orchestrate_report(report: &OrchestrateCommandReport) -> String {
    let mut out = String::new();

    // Headline
    if let Some(session_status) = &report.session_status {
        out.push_str(&format!("Session: {}\n", session_status.session_id));
        out.push_str(&format!(
            "Goal: {}\n",
            session_status.goal.as_deref().unwrap_or("No goal set")
        ));
    } else {
        out.push_str("Session: Unknown\n");
    }

    out.push_str(&format!("Status: {:?}\n", report.exit_status));

    // Phase Request Details
    for event in report.events.iter().rev() {
        if event.event_kind == "phase_request" {
            if let Some(phase) = &event.phase_kind {
                out.push_str(&format!("\nPhase Requested: {}\n", phase));
            } else {
                out.push_str("\nPhase Requested\n");
            }
            if let Some(phase_request) = &event.phase_request {
                out.push_str(&format!("Reason: {}\n", phase_request.reason));
                out.push_str(&format!("Question: {}\n", phase_request.question));
                if phase_request.kind != "clarification"
                    && let Some(instruction) = &event.instruction
                {
                    out.push_str(&format!("Guidance: {}\n", instruction));
                }
            } else {
                if !event.message.is_empty() {
                    out.push_str(&format!("Reason: {}\n", event.message));
                }
                if let Some(instruction) = &event.instruction {
                    out.push_str(&format!("Guidance: {}\n", instruction));
                }
            }
            break; // Stop at the latest phase request
        }
    }

    // Identify recent artifacts
    let mut recent_artifacts = Vec::new();
    for event in &report.events {
        if let Some(artifact) = &event.artifact {
            recent_artifacts.push((artifact.artifact_kind, artifact.artifact_ref.clone()));
        }
    }

    if !recent_artifacts.is_empty() {
        out.push_str("\nGenerated Artifacts:\n");
        let mut seen = std::collections::HashSet::new();
        for (kind, aref) in recent_artifacts {
            if seen.insert(aref.clone()) {
                let label = format!("{:?}", kind);
                out.push_str(&format!("  → {}: {}\n", label, aref));
            }
        }
    }

    // Blocking reason
    let mut blocked_reason = None;
    if let Some(status) = &report.session_status
        && let Some(reason) = &status.latest_governance_blocked_reason
    {
        blocked_reason = Some(reason.clone());
    }

    // Check terminal events for blocked reasons too
    if blocked_reason.is_none() {
        for event in report.events.iter().rev() {
            if event.event_kind == "terminal" && event.message.contains("blocked") {
                blocked_reason = Some(event.message.clone());
                break;
            }
        }
    }

    if let Some(blocking) = blocked_reason {
        out.push_str(&format!("\n⚠ Blocked: {}\n", blocking));
    }

    if let Some(status) = &report.session_status {
        append_completion_verification_lines(&mut out, status);
    }

    // Next action
    if let Some(last_event) = report.events.last() {
        if let Some(resume) = &last_event.resume_command {
            out.push_str(&format!("\nResume Action: {}\n", resume));
        } else if let Some(next) = &last_event.next_command {
            out.push_str(&format!("\nNext Action: {}\n", next));
        }
    }

    out
}

fn append_completion_verification_lines(
    out: &mut String,
    status: &crate::domain::session::SessionStatusView,
) {
    let Some(state) = status.completion_verification_state else {
        return;
    };

    out.push_str(&format!("\nCompletion Verification State: {}\n", state.as_str()));
    if let Some(claim) = &status.completion_claim {
        out.push_str(&format!("Claim: {} ({})\n", claim.kind.as_str(), claim.source.as_str()));
        out.push_str(&format!("Claim Summary: {}\n", claim.summary));
    }
    if let Some(blocked_claims) = &status.completion_blocked_claims
        && !blocked_claims.is_empty()
    {
        out.push_str(&format!(
            "Blocked Claims: {}\n",
            blocked_claims.iter().map(|claim| claim.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }
    if let Some(evidence_refs) = &status.completion_evidence_refs
        && !evidence_refs.is_empty()
    {
        out.push_str(&format!("Evidence Refs: {}\n", evidence_refs.join(", ")));
    }
    if let Some(findings) = &status.completion_verification_findings {
        for finding in findings {
            out.push_str(&format!(
                "Finding: {} | {} | {}\n",
                finding.kind.as_str(),
                finding.severity.as_str(),
                finding.message
            ));
            if !finding.changed_paths.is_empty() {
                out.push_str(&format!("Changed Paths: {}\n", finding.changed_paths.join(", ")));
            }
            out.push_str(&format!("Required Action: {}\n", finding.required_action.as_str()));
        }
    }
    out.push_str(
        "Boundary: Boundline owns proof execution and task-completion gating; Canon may consume emitted evidence refs later.\n",
    );
}

#[cfg(test)]
mod tests {
    use super::render_human_orchestrate_report;
    use crate::cli::CommandExitStatus;
    use crate::cli::orchestrate::{
        OrchestrateCommandReport, OrchestrateEventEnvelope, OrchestratePhaseRequest,
        OrchestratePhaseRequestExpectedAnswer,
    };
    use crate::domain::session::SessionStatusView;

    #[test]
    fn render_human_orchestrate_report_prefers_structured_phase_request_question() {
        let report = OrchestrateCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: String::new(),
            trace_location: None,
            session_status: None,
            trace_summary: None,
            events: vec![OrchestrateEventEnvelope {
                event_id: "orchestrate-event-1".to_string(),
                timestamp_ms: 1,
                event_kind: "phase_request".to_string(),
                audit: None,
                actor_kind: None,
                actor_name: None,
                runtime_kind: None,
                provider: None,
                route_slot: None,
                model_name: None,
                decision_family: None,
                review_step: None,
                vote_summary: None,
                adjudication_summary: None,
                governance_mode: None,
                session_ref: Some("session-1".to_string()),
                phase_kind: Some("planning".to_string()),
                stage_key: Some("plan".to_string()),
                message: "clarification is required before planning can continue".to_string(),
                artifact: None,
                phase_request: Some(OrchestratePhaseRequest {
                    request_id: "req-session-1-planning-plan-1".to_string(),
                    kind: "clarification".to_string(),
                    phase: "planning".to_string(),
                    reason: "clarification is required before planning can continue".to_string(),
                    question:
                        "Which persistence store is authoritative for the first slice?"
                            .to_string(),
                    expected_answer: Some(OrchestratePhaseRequestExpectedAnswer {
                        answer_type: "free_text".to_string(),
                        options: Vec::new(),
                    }),
                }),
                instruction: Some(
                    "answer this question before planning continues: Which persistence store is authoritative for the first slice?"
                        .to_string(),
                ),
                resume_command: Some("boundline orchestrate --json-stream".to_string()),
                assistant_resume_command: None,
                next_command: None,
                assistant_next_command: None,
                session_status: None,
                trace_summary: None,
            }],
        };

        let rendered = render_human_orchestrate_report(&report);

        assert!(rendered.contains("Phase Requested: planning"), "{rendered}");
        assert!(
            rendered.contains(
                "Question: Which persistence store is authoritative for the first slice?"
            ),
            "{rendered}"
        );
        assert!(!rendered.contains("Guidance:"), "{rendered}");
    }

    fn empty_envelope(event_kind: &str) -> OrchestrateEventEnvelope {
        OrchestrateEventEnvelope {
            event_id: "test-event".to_string(),
            timestamp_ms: 1,
            event_kind: event_kind.to_string(),
            audit: None,
            actor_kind: None,
            actor_name: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            decision_family: None,
            review_step: None,
            vote_summary: None,
            adjudication_summary: None,
            governance_mode: None,
            session_ref: None,
            phase_kind: None,
            stage_key: None,
            message: String::new(),
            artifact: None,
            phase_request: None,
            instruction: None,
            resume_command: None,
            assistant_resume_command: None,
            next_command: None,
            assistant_next_command: None,
            session_status: None,
            trace_summary: None,
        }
    }

    #[test]
    fn render_orchestrate_report_covers_missing_phase_kind_and_no_phase_request() {
        let mut event = empty_envelope("phase_request");
        event.phase_kind = None;
        event.phase_request = None;
        event.message = "phase event without structured request".to_string();
        event.instruction = Some("follow the guidance doc".to_string());

        let report = OrchestrateCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: String::new(),
            trace_location: None,
            session_status: None,
            trace_summary: None,
            events: vec![event],
        };

        let rendered = render_human_orchestrate_report(&report);
        assert!(rendered.contains("Phase Requested\n"), "{rendered}");
        assert!(rendered.contains("Reason: phase event without structured request"), "{rendered}");
        assert!(rendered.contains("Guidance: follow the guidance doc"), "{rendered}");
    }

    #[test]
    fn render_orchestrate_report_covers_governance_blocked_reason_and_non_clarification_guidance() {
        let mut event = empty_envelope("phase_request");
        event.phase_kind = Some("implementation".to_string());
        event.phase_request = Some(OrchestratePhaseRequest {
            request_id: "req-impl-1".to_string(),
            kind: "feedback".to_string(),
            phase: "implementation".to_string(),
            reason: "implementation guidance needed".to_string(),
            question: "Which module to update?".to_string(),
            expected_answer: Some(OrchestratePhaseRequestExpectedAnswer {
                answer_type: "free_text".to_string(),
                options: Vec::new(),
            }),
        });
        event.instruction = Some("update the bounded context first".to_string());

        let status = SessionStatusView {
            session_id: "session-blocked".to_string(),
            latest_governance_blocked_reason: Some("awaiting canon approval".to_string()),
            ..Default::default()
        };

        let report = OrchestrateCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: String::new(),
            trace_location: None,
            session_status: Some(status),
            trace_summary: None,
            events: vec![event],
        };

        let rendered = render_human_orchestrate_report(&report);
        assert!(rendered.contains("Guidance: update the bounded context first"), "{rendered}");
        assert!(rendered.contains("Blocked: awaiting canon approval"), "{rendered}");
    }
}
