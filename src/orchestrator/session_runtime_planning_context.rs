use super::*;

impl SessionRuntime {
    pub(super) fn planning_context_sources(
        &self,
        session: &ActiveSessionRecord,
        goal: &str,
    ) -> PlanningContextSources {
        let negotiation_packet = self.session_negotiation_packet(session, goal);
        let compacted_project_memory =
            Self::compacted_project_memory_for_workspace(&self.workspace_ref);

        PlanningContextSources {
            authored_input_summary: session
                .authored_brief
                .as_ref()
                .map(|bundle| bundle.summary_text()),
            authored_input_sources: session
                .authored_brief
                .as_ref()
                .map(|bundle| bundle.ordered_source_labels())
                .unwrap_or_default(),
            authored_input_documents: session
                .authored_brief
                .as_ref()
                .map(|bundle| {
                    bundle
                        .sources
                        .iter()
                        .map(|source| AuthoredInputDocument {
                            label: source.display_label(),
                            content: source.content.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            execution_profile_read_targets: load_workspace_execution_profile(&self.workspace_ref)
                .map(|profile| profile.read_targets)
                .unwrap_or_default(),
            negotiation_goal_summary: negotiation_packet
                .as_ref()
                .map(|packet| packet.goal_summary.clone()),
            negotiation_resolution: negotiation_packet
                .as_ref()
                .map(|packet| packet.resolution_state.as_str().to_string()),
            negotiation_acceptance_boundary: negotiation_packet
                .as_ref()
                .map(|packet| packet.acceptance_boundary.success_headline.clone()),
            latest_trace_ref: session.latest_trace_ref.clone(),
            workflow_progress: session.workflow_progress.clone(),
            canon_capability_snapshot: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.latest_canon_capability_snapshot().ok().flatten()),
            compacted_canon_memory: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.latest_compacted_canon_memory().ok().flatten())
                .or(compacted_project_memory),
            latest_changed_files: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.state.get("latest_changed_files"))
                .and_then(|value| value.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            latest_validation_status: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.state.get("latest_validation_status"))
                .and_then(|value| value.as_str().map(str::to_string)),
        }
    }

    fn compacted_project_memory_for_workspace(
        workspace_ref: &Path,
    ) -> Option<CompactedCanonMemory> {
        let context = read_project_memory(workspace_ref);
        Self::compacted_canon_memory_from_project_memory_context(workspace_ref, &context)
    }

    pub(super) fn compacted_canon_memory_from_project_memory_context(
        workspace_ref: &Path,
        context: &ProjectMemoryContext,
    ) -> Option<CompactedCanonMemory> {
        if context.status == ProjectMemoryStatus::Absent {
            return None;
        }

        let condition = context.condition_for_workspace(workspace_ref)?;
        let artifact_refs = if context.status == ProjectMemoryStatus::Available {
            Self::project_memory_artifact_refs(workspace_ref, context)
        } else {
            Vec::new()
        };
        let contribution_summaries = if context.status == ProjectMemoryStatus::Available {
            Self::project_memory_contribution_summaries(workspace_ref, context)
        } else {
            Vec::new()
        };
        let credibility = match condition.decision() {
            crate::domain::project_memory::ProjectMemoryDecision::Proceed => {
                MemoryCredibilityState::Credible
            }
            crate::domain::project_memory::ProjectMemoryDecision::Warning => {
                MemoryCredibilityState::Stale
            }
            crate::domain::project_memory::ProjectMemoryDecision::HardStop => {
                MemoryCredibilityState::Insufficient
            }
        };
        let (possible_actions, recommended_next_action) = match condition {
            ProjectMemoryCondition::Stable => (Vec::new(), None),
            ProjectMemoryCondition::Pending => (
                vec![Self::project_memory_action(
                    "refresh",
                    "refresh project memory after Canon promotes a stable docs/project surface",
                )],
                Some(Self::project_memory_recommended_action(
                    "refresh",
                    "refresh project memory after Canon promotes a stable docs/project surface",
                )),
            ),
            ProjectMemoryCondition::EvidenceOnly => (
                vec![Self::project_memory_action(
                    "promote",
                    "publish a stable docs/project surface from Canon before reusing project memory as planning context",
                )],
                Some(Self::project_memory_recommended_action(
                    "promote",
                    "publish a stable docs/project surface from Canon before reusing project memory as planning context",
                )),
            ),
            ProjectMemoryCondition::ManualPromotion => (
                vec![Self::project_memory_action(
                    "promote",
                    "complete the manual Canon promotion step and refresh project memory",
                )],
                Some(Self::project_memory_recommended_action(
                    "promote",
                    "complete the manual Canon promotion step and refresh project memory",
                )),
            ),
            ProjectMemoryCondition::IncompleteMetadata => (
                vec![Self::project_memory_action(
                    "inspect",
                    "inspect the Canon packet metadata sidecars and refresh project memory",
                )],
                Some(Self::project_memory_recommended_action(
                    "inspect",
                    "inspect the Canon packet metadata sidecars and refresh project memory",
                )),
            ),
            ProjectMemoryCondition::BlockedGovernance => (
                vec![Self::project_memory_action(
                    "unblock",
                    "resolve the blocked Canon governance outcome before planning continues",
                )],
                Some(Self::project_memory_recommended_action(
                    "unblock",
                    "resolve the blocked Canon governance outcome before planning",
                )),
            ),
            ProjectMemoryCondition::MissingRequiredApproval => (
                vec![Self::project_memory_action(
                    "approve",
                    "complete the required Canon approval flow and refresh project memory",
                )],
                Some(Self::project_memory_recommended_action(
                    "approve",
                    "complete the required Canon approval flow before planning",
                )),
            ),
            ProjectMemoryCondition::MissingRequiredSourceArtifacts => (
                vec![Self::project_memory_action(
                    "restore",
                    "restore or republish the required Canon source artifacts before planning",
                )],
                Some(Self::project_memory_recommended_action(
                    "restore",
                    "restore or republish the required Canon source artifacts before planning",
                )),
            ),
            ProjectMemoryCondition::UnsupportedContract => (
                vec![Self::project_memory_action(
                    "update",
                    "update Canon or Boundline so both support the same project-memory contract",
                )],
                Some(Self::project_memory_recommended_action(
                    "update",
                    "update Canon or Boundline so both support the same project-memory contract before planning",
                )),
            ),
        };

        Some(CompactedCanonMemory {
            headline: condition.headline().to_string(),
            credibility,
            stage_key: None,
            run_ref: context.surfaces.iter().find_map(|surface| {
                surface.lineage.as_ref().map(|lineage| lineage.source_ref_leaf().to_string())
            }),
            packet_ref: None,
            reason_code: condition.reason_code().map(str::to_string),
            artifact_refs: artifact_refs.clone(),
            mode_summary: None,
            possible_actions,
            recommended_next_action,
            evidence_summary: (!artifact_refs.is_empty() || !contribution_summaries.is_empty())
                .then_some(CanonEvidenceInspectSummary {
                    execution_posture: None,
                    carried_forward_items: contribution_summaries,
                    artifact_provenance_links: artifact_refs,
                    closure_status: None,
                    closure_findings: Vec::new(),
                }),
            authority_provenance_lines: Vec::new(),
            adaptive_provenance_lines: Vec::new(),
            semantic_provenance_lines: Vec::new(),
        })
    }

    fn project_memory_action(action: &str, text: &str) -> CanonPossibleActionSummary {
        CanonPossibleActionSummary {
            action: action.to_string(),
            text: text.to_string(),
            target: None,
        }
    }

    fn project_memory_recommended_action(
        action: &str,
        rationale: &str,
    ) -> CanonRecommendedActionSummary {
        CanonRecommendedActionSummary {
            action: action.to_string(),
            rationale: rationale.to_string(),
            target: None,
        }
    }

    pub(super) fn project_memory_artifact_refs(
        workspace_ref: &Path,
        context: &ProjectMemoryContext,
    ) -> Vec<String> {
        let mut refs = context
            .surfaces
            .iter()
            .map(|surface| surface.path.display().to_string())
            .collect::<Vec<_>>();

        for lineage in &context.evidence_refs {
            let evidence_root = evidence_root_for_lineage(workspace_ref, lineage);
            if evidence_root.exists() {
                let display = evidence_root
                    .strip_prefix(workspace_ref)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| evidence_root.display().to_string());
                if !refs.contains(&display) {
                    refs.push(display);
                }
            }
        }

        refs
    }

    fn project_memory_contribution_summaries(
        workspace_ref: &Path,
        context: &ProjectMemoryContext,
    ) -> Vec<String> {
        let mut summaries = BTreeSet::new();
        for lineage in context
            .evidence_refs
            .iter()
            .chain(context.surfaces.iter().filter_map(|surface| surface.lineage.as_ref()))
        {
            for summary in evidence_contribution_summaries(workspace_ref, lineage) {
                summaries.insert(summary);
            }
        }

        summaries.into_iter().collect()
    }

    pub(super) fn session_negotiation_packet(
        &self,
        session: &ActiveSessionRecord,
        goal: &str,
    ) -> Option<NegotiatedDeliveryPacket> {
        session.negotiation_packet.clone().or_else(|| {
            session
                .authored_brief
                .as_ref()
                .map(|bundle| {
                    NegotiatedDeliveryPacket::from_authored_brief(
                        &session.session_id,
                        &session.workspace_ref,
                        goal,
                        bundle,
                    )
                })
                .or_else(|| {
                    (!goal.trim().is_empty()).then(|| {
                        NegotiatedDeliveryPacket::from_goal(
                            &session.session_id,
                            &session.workspace_ref,
                            goal,
                        )
                    })
                })
        })
    }

    pub(super) fn apply_negotiation_projection(
        &self,
        session: &ActiveSessionRecord,
        goal: &str,
        goal_plan: &mut GoalPlan,
    ) {
        if let Some(packet) = self.session_negotiation_packet(session, goal) {
            goal_plan.negotiation_goal_summary = Some(packet.goal_summary);
            goal_plan.negotiation_resolution = Some(packet.resolution_state.as_str().to_string());
            goal_plan.negotiation_acceptance_boundary =
                Some(packet.acceptance_boundary.success_headline);
        }
    }

    pub(super) fn unresolved_planning_governance_record<'a>(
        &self,
        session: &'a ActiveSessionRecord,
    ) -> Option<&'a GovernedStageRecord> {
        session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle.stage_records.iter().rev().find(|record| {
                planning_canon_mode_for_stage_key(&record.stage_key).is_some()
                    && matches!(
                        record.lifecycle_state,
                        GovernanceLifecycleState::AwaitingApproval
                            | GovernanceLifecycleState::Blocked
                            | GovernanceLifecycleState::Failed
                    )
            })
        })
    }
}
