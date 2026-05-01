# Data Model: Adaptive Repair Depth

**Feature**: 021-adaptive-repair-depth  
**Date**: 2026-05-01

## Core Entities

### Validation Guidance

The bounded subset of the latest validation failure evidence that can credibly influence the next adaptive attempt.

```text
ValidationGuidance
├── source: validation_record | failure_message
├── matched_paths: Vec<String>
├── matched_terms: Vec<String>
├── headline: String
└── confidence: hinted | strong
```

**Behavioral rules**:
- Guidance must derive only from the active task's latest validation record and current failure message.
- Guidance may influence ranking and selection, but it must not override execution limits or candidate-signature guards.
- Missing or ambiguous guidance must remain visible as an explicit non-selection or low-confidence reason.

### Adaptive Candidate Ranking

The ordered bounded set of repair candidates considered for the next adaptive attempt.

```text
AdaptiveCandidateRanking
├── selected_targets: Vec<String>
├── scored_targets: Vec<PathScore>
├── candidate_signatures: Vec<String>
├── guidance: ValidationGuidance
└── selected_signature: String
```

**Behavioral rules**:
- Ranking starts from existing bounded `read_targets` and local candidate synthesis.
- Validation guidance may re-rank targets or candidates but must not introduce files outside bounded scope.
- Previously used candidate signatures remain excluded unless new bounded evidence makes them credible and explicitly visible.

### Adaptive Selection Evidence

The inspectable explanation of why a bounded repair candidate was selected after failed validation.

```text
AdaptiveSelectionEvidence
├── goal_terms: Vec<String>
├── validation_terms: Vec<String>
├── validation_guidance: ValidationGuidance
├── path_scores: Vec<PathScore>
└── reason: String
```

**Behavioral rules**:
- Selection evidence must explain both the bounded slice and the validation-guided reason for the new attempt.
- Evidence must be persistable in task context and inspectable through CLI and trace surfaces.
- Evidence must remain understandable without exposing hidden heuristics or external analyzers.

### Attempt Lineage

The explicit relationship between the current adaptive attempt and the prior bounded attempt.

```text
AttemptLineage
├── previous_attempt_id: Option<String>
├── current_attempt_id: String
├── transition_kind: initial | narrowed | broadened | replaced | terminated
└── reason: String
```

**Behavioral rules**:
- Lineage reasons must include whether validation guidance caused the new attempt.
- A transition to `terminated` must reflect that no credible new bounded candidate remained.
- Changing the selected workspace slice must remain visible through lineage plus latest workspace-slice summaries.

### Adaptive Route Guidance Example

The authored example used in shipped docs to explain how adaptive compatibility execution coexists with session-native workflows, review, and governance.

```text
AdaptiveRouteGuidanceExample
├── execution_path: fixture_compatibility
├── workflow_present: Boolean
├── review_present: Boolean
├── governance_present: Boolean
└── non_goals: Vec<String>
```

**Behavioral rules**:
- The example must keep adaptive execution explicit as a compatibility path.
- It must explain that workflow, review, or governance visibility does not transfer control of adaptive execution.
- It must reject Canon-owned or workflow-owned adaptive orchestration expectations explicitly.

## Relationships

- `ValidationGuidance` is derived from the latest persisted `ValidationRecord` and current step failure information.
- `AdaptiveCandidateRanking` uses `ValidationGuidance` plus bounded workspace scoring and candidate-signature history.
- `AdaptiveSelectionEvidence` persists the reasoning behind one chosen ranked candidate.
- `AttemptLineage` explains how one adaptive attempt replaced, narrowed, or terminated the previous path.
- `AdaptiveRouteGuidanceExample` documents how the slice relates to existing session, workflow, review, and governance surfaces.

## State Transitions

### Validation-Guided Adaptive Replanning

```text
validation_failed -> extract_guidance
extract_guidance -> rerank_candidates
rerank_candidates -> select_new_attempt
rerank_candidates -> terminal_when_no_credible_candidate
select_new_attempt -> execute_code_and_verify
```

### Attempt Lineage

```text
initial_attempt -> replaced_attempt
replaced_attempt -> replaced_attempt
replaced_attempt -> terminated
```

### Route Story

```text
explicit_compatibility_run -> adaptive_replan
adaptive_replan -> status_next_inspect_projection
workflow_or_governance_present -> explicit_projection_only
```

The model stays intentionally narrow: it deepens bounded adaptive repair quality and inspectability without moving adaptive control to a new runtime surface.