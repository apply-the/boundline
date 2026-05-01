# Data Model: Broaden Bounded Adaptive Repair

**Feature**: 023-broaden-bounded-adaptive-repair  
**Date**: 2026-05-01

## Core Entities

### Adaptive Mutation Family

The bounded built-in change category that can synthesize deterministic repair candidates for one selected workspace slice.

```text
AdaptiveMutationFamily
├── kind: arithmetic_swap | comparison_flip | boolean_flip | ...bounded built-in extensions
├── selected_target: String
├── generated_candidates: Integer
├── deterministic_order: Integer
└── supported_patterns: Vec<String>
```

**Behavioral rules**:
- Mutation families must be built-in and deterministic.
- Families may only generate candidates inside manifest-declared `read_targets`.
- Families must produce stable candidate signatures for materially identical changes.

### Adaptive Candidate Credibility

The inspectable explanation of why one bounded candidate is more plausible than the remaining candidates.

```text
AdaptiveCandidateCredibility
├── candidate_signature: String
├── mutation_family: String
├── credibility_reason: String
├── credibility_inputs:
│   ├── goal_terms: Vec<String>
│   ├── validation_terms: Vec<String>
│   ├── validation_guidance_headline: Option<String>
│   └── prior_failure_penalty: Boolean
├── rejected_alternatives: Vec<String>
└── rejection_reasons: Vec<String>
```

**Behavioral rules**:
- Every selected candidate must have a non-empty credibility reason.
- Rejected alternatives should stay bounded to the candidates already considered in the current adaptive decision.
- Prior failed signatures should reduce credibility unless new bounded evidence changes the selection.

### Adaptive Exhaustion State

The explicit terminal explanation that no remaining bounded candidate is credible enough or allowed enough to continue.

```text
AdaptiveExhaustionState
├── terminal_status: failed | exhausted
├── exhaustion_reason: String
├── rejected_candidate_count: Integer
├── remaining_candidate_count: Integer
├── limit_hit: Option<String>
└── recommended_follow_up: String
```

**Behavioral rules**:
- Exhaustion must remain explicit instead of being folded into generic validation failure output.
- Exhaustion must name whether the stop came from credibility collapse, candidate depletion, or configured execution limits.
- Follow-up guidance must remain on the explicit compatibility route.

### Adaptive Selection Evidence

The persisted summary of the chosen candidate, rejected alternatives, validation hints, and attempt-lineage relationship.

```text
AdaptiveSelectionEvidence
├── workspace_slice: WorkspaceSliceSelection
├── selected_candidate: AdaptiveCandidateCredibility
├── attempt_lineage: AttemptLineage
├── validation_guidance: Option<ValidationGuidance>
└── exhaustion_state: Option<AdaptiveExhaustionState>
```

**Behavioral rules**:
- Selection evidence must be derivable from persisted task context and trace state.
- Selection evidence must stay consistent across `run`, `status`, `next`, and `inspect`.
- Exhaustion state must be omitted when the run still has a credible next bounded candidate.

## Relationships

- `AdaptiveMutationFamily` generates bounded candidates for the selected `WorkspaceSliceSelection` already defined in the adaptive engine.
- `AdaptiveCandidateCredibility` explains why one generated candidate becomes the next `ExecutionAttemptDefinition`.
- `AdaptiveExhaustionState` is the explicit terminal outcome when no remaining candidate can be selected credibly.
- `AdaptiveSelectionEvidence` is the read-side projection that ties together workspace slice, selected candidate, validation guidance, and exhaustion state.

## State Transitions

### Candidate Lifecycle

```text
generated -> ranked -> selected
generated -> rejected
selected -> validated
validated -> replanned
validated -> terminal
```

### Exhaustion Lifecycle

```text
replan_requested -> candidates_evaluated
candidates_evaluated -> selected_candidate
candidates_evaluated -> exhausted_terminal
selected_candidate -> succeeded_terminal
selected_candidate -> failed_terminal
```

The model remains intentionally narrow: it deepens bounded adaptive candidate generation and explanation without changing the underlying route ownership model.