use super::*;

impl SessionRuntime {
    // Builds the guardian request from a fixture-style step result, preferring
    // normalized changed-file state and then backfilling explicit evidence refs.
    pub(super) fn guardian_request_for_step(
        &self,
        session: &ActiveSessionRecord,
        task: &Task,
        step: &Step,
        phase: CapabilityPhase,
        result: &StepExecutionResult,
    ) -> GuardianExecutionRequest {
        let goal_text = session.goal.clone().unwrap_or_else(|| task.goal.clone());
        let target_ref = step
            .target_name
            .clone()
            .or_else(|| {
                session
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.tasks.get(task.plan.current_step_index))
                    .map(|planned| planned.target.clone())
            })
            .unwrap_or_else(|| "workspace".to_string());
        let changed_files = Self::changed_files_for_guardian(task, result, step, &target_ref);
        let mut evidence_refs = changed_files.clone();
        if let Some(target_name) = step.target_name.as_ref()
            && !evidence_refs.iter().any(|reference| reference == target_name)
        {
            evidence_refs.push(target_name.clone());
        }
        if let Some(evidence) = result.evidence.as_ref() {
            evidence_refs.push(evidence.to_string());
        }

        GuardianExecutionRequest {
            goal_text,
            target_ref,
            phase,
            evidence_refs,
            changed_files,
            workspace_signals: collect_workspace_signals(&self.workspace_ref),
        }
    }

    // Reconstructs the same guardian request shape for native runs, where the
    // authoritative evidence lives in persisted decisions instead of step payloads.
    pub(super) fn native_guardian_request(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
        decisions: &[Decision],
    ) -> Option<GuardianExecutionRequest> {
        // Native runs do not emit fixture step payloads, so reuse the guardian
        // executor by deriving the same request shape from persisted decisions.
        let phase = Self::guardian_phase_for_decisions(decisions)?;
        let mut changed_files = decisions
            .iter()
            .filter(|decision| {
                matches!(
                    decision.decision_type,
                    DecisionType::Code | DecisionType::Fix | DecisionType::Test
                )
            })
            .map(|decision| decision.target.trim().to_string())
            .filter(|target| !target.is_empty())
            .collect::<Vec<_>>();
        if changed_files.is_empty() {
            changed_files = goal_plan
                .tasks
                .iter()
                .map(|task| task.target.trim().to_string())
                .filter(|target| !target.is_empty())
                .collect();
        }
        if changed_files.is_empty() {
            return None;
        }
        let mut unique_files = BTreeSet::new();
        changed_files.retain(|target| unique_files.insert(target.clone()));

        let target_ref = changed_files.first().cloned()?;
        let mut seen_refs = BTreeSet::new();
        let mut evidence_refs = Vec::new();
        for changed_file in &changed_files {
            if seen_refs.insert(changed_file.clone()) {
                evidence_refs.push(changed_file.clone());
            }
        }
        for decision in decisions {
            if seen_refs.insert(decision.target.clone()) {
                evidence_refs.push(decision.target.clone());
            }
            for evidence in &decision.evidence_inputs {
                if seen_refs.insert(evidence.reference.clone()) {
                    evidence_refs.push(evidence.reference.clone());
                }
            }
            if let Some(tool_result) = decision.tool_result.as_ref()
                && seen_refs.insert(tool_result.invocation.clone())
            {
                evidence_refs.push(tool_result.invocation.clone());
            }
        }

        Some(GuardianExecutionRequest {
            goal_text: session.goal.clone().unwrap_or_else(|| goal_plan.goal_text.clone()),
            target_ref,
            phase,
            evidence_refs,
            changed_files,
            workspace_signals: collect_workspace_signals(&self.workspace_ref),
        })
    }

    // Maps the planned step hint to the lifecycle phase used to resolve and run
    // guidance or guardians after that step finishes.
    pub(super) fn guardian_phase_for_step(
        session: &ActiveSessionRecord,
        step_index: usize,
    ) -> CapabilityPhase {
        match session
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.tasks.get(step_index))
            .and_then(|planned| planned.decision_type_hint)
        {
            Some(crate::domain::decision::DecisionType::Analyze) => CapabilityPhase::Planning,
            Some(crate::domain::decision::DecisionType::Code)
            | Some(crate::domain::decision::DecisionType::Fix) => CapabilityPhase::Implementation,
            Some(crate::domain::decision::DecisionType::Test) => CapabilityPhase::Verification,
            Some(crate::domain::decision::DecisionType::Replan) => CapabilityPhase::Review,
            None => CapabilityPhase::Implementation,
        }
    }

    // Native flows do not have an explicit step cursor, so infer the guardian
    // phase from the latest persisted decision that materially changed the run.
    pub(super) fn guardian_phase_for_decisions(decisions: &[Decision]) -> Option<CapabilityPhase> {
        decisions
            .iter()
            .rev()
            .map(|decision| match decision.decision_type {
                DecisionType::Analyze => Some(CapabilityPhase::Planning),
                DecisionType::Code | DecisionType::Fix => Some(CapabilityPhase::Implementation),
                DecisionType::Test => Some(CapabilityPhase::Verification),
                DecisionType::Replan => Some(CapabilityPhase::Review),
            })
            .next()
            .flatten()
    }

    pub(super) fn changed_files_for_guardian(
        task: &Task,
        result: &StepExecutionResult,
        step: &Step,
        fallback_target: &str,
    ) -> Vec<String> {
        // Successful bounded work normalizes changed files under
        // latest_changed_files; fall back only when that normalized view is absent.
        for state_key in ["latest_changed_files", "changed_files"] {
            if let Some(changed_files) = task.context.state.get(state_key).and_then(Value::as_array)
            {
                let files = changed_files
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                if !files.is_empty() {
                    return files;
                }
            }
        }

        if let Some(changed_files) = result
            .evidence
            .as_ref()
            .and_then(|value| value.get("changed_files"))
            .and_then(Value::as_array)
        {
            let files = changed_files
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            if !files.is_empty() {
                return files;
            }
        }

        step.target_name
            .clone()
            .map(|target| vec![target])
            .unwrap_or_else(|| vec![fallback_target.to_string()])
    }

    // Merge execution output into the flattened read-side projection while
    // keeping planning-time guidance selection stable across later phases.
    pub(super) fn merge_guardian_projection(
        projection: &mut GuidanceGuardianProjection,
        update: &GuidanceGuardianProjection,
    ) {
        // Planning guidance stays stable once selected, while execution-phase
        // guardian output should reflect the latest authoritative verification pass.
        if projection.capability_resolution_summary.is_none() {
            projection.capability_resolution_summary = update.capability_resolution_summary.clone();
        }
        if projection.loaded_guidance_sources.is_empty() {
            projection.loaded_guidance_sources = update.loaded_guidance_sources.clone();
        }
        if projection.skipped_guidance_sources.is_empty() {
            projection.skipped_guidance_sources = update.skipped_guidance_sources.clone();
        }
        projection.loaded_guardian_sources = update.loaded_guardian_sources.clone();
        projection.skipped_guardian_sources = update.skipped_guardian_sources.clone();
        projection.guardian_timeline = update.guardian_timeline.clone();
        projection.guardian_findings_summary = update.guardian_findings_summary.clone();
        projection.guardian_findings = update.guardian_findings.clone();
        projection.guardian_degradations = update.guardian_degradations.clone();
        projection.guardian_blocking_outcome = update.guardian_blocking_outcome.clone();
    }

    // Mirror the flattened projection into trace payloads so `inspect` can
    // hydrate the same operator story without recomputing runtime resolution.
    pub(super) fn append_guardian_projection_payload(
        payload: &mut Value,
        projection: &GuidanceGuardianProjection,
    ) {
        let Some(object) = payload.as_object_mut() else {
            return;
        };
        if let Some(summary) = projection.capability_resolution_summary.as_ref() {
            object.insert(
                "capability_resolution_summary".to_string(),
                Value::String(summary.clone()),
            );
        }
        if !projection.loaded_guidance_sources.is_empty() {
            object.insert(
                "loaded_guidance_sources".to_string(),
                serde_json::to_value(&projection.loaded_guidance_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.skipped_guidance_sources.is_empty() {
            object.insert(
                "skipped_guidance_sources".to_string(),
                serde_json::to_value(&projection.skipped_guidance_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.loaded_guardian_sources.is_empty() {
            object.insert(
                "loaded_guardian_sources".to_string(),
                serde_json::to_value(&projection.loaded_guardian_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.skipped_guardian_sources.is_empty() {
            object.insert(
                "skipped_guardian_sources".to_string(),
                serde_json::to_value(&projection.skipped_guardian_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.guardian_timeline.is_empty() {
            object.insert(
                "guardian_timeline".to_string(),
                serde_json::to_value(&projection.guardian_timeline).unwrap_or(Value::Null),
            );
        }
        if let Some(summary) = projection.guardian_findings_summary.as_ref() {
            object.insert("guardian_findings_summary".to_string(), Value::String(summary.clone()));
        }
        if !projection.guardian_findings.is_empty() {
            object.insert(
                "guardian_findings".to_string(),
                serde_json::to_value(&projection.guardian_findings).unwrap_or(Value::Null),
            );
        }
        if !projection.guardian_degradations.is_empty() {
            object.insert(
                "guardian_degradations".to_string(),
                serde_json::to_value(&projection.guardian_degradations).unwrap_or(Value::Null),
            );
        }
        if let Some(outcome) = projection.guardian_blocking_outcome.as_ref() {
            object.insert("guardian_blocking_outcome".to_string(), Value::String(outcome.clone()));
        }
    }
}
