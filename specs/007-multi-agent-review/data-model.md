# Data Model: Multi-Agent Review & Voting

## ReviewProfile

- Purpose: Defines the bounded review behavior attached to one execution profile.
- Fields:
  - `triggers`: Ordered list of explicit review triggers that may start a review phase.
  - `reviewers`: Ordered list of `ReviewerDefinition` entries participating in the council.
  - `vote_rule`: One `VoteRuleDefinition` describing how findings are resolved.
  - `adjudication`: Optional `AdjudicationDefinition` used when the initial vote does not produce a credible terminal decision.
  - `scenarios`: Trigger-specific `ReviewScenario` records used by the current bounded runtime to produce deterministic review findings.
- Validation rules:
  - At least one trigger must be defined.
  - At least two reviewers must be defined for a council.
  - Reviewer identifiers must be unique within the profile.
  - The configured vote rule must be compatible with the reviewer set.
  - Adjudication is optional per profile, but the runtime must support it and, when enabled, it must define one explicit adjudicator.

## ReviewTrigger

- Purpose: Names the explicit reason a bounded review phase starts.
- Supported values for the initial slice:
  - `validation_failed`
  - `high_risk_change`
  - `pr_ready`
- Validation rules:
  - Triggers must come from the supported vocabulary.
  - Only one review phase may start per task stage in the initial slice.
  - Later duplicate triggers for the same task stage are recorded but ignored.

## ReviewPhase

- Purpose: Tracks one bounded review lifecycle attached to one task stage.
- Fields:
  - `stage_key`: Stable identifier for the reviewed task stage.
  - `trigger`: The `ReviewTrigger` that started review.
  - `status`: One of `pending`, `in_review`, `voted`, `adjudicating`, or a terminal `ReviewDecision`.
  - `duplicate_trigger_count`: Number of later duplicate triggers recorded for the same stage.
- Validation rules:
  - Only one active `ReviewPhase` may exist for a given `stage_key`.
  - `stage_key` identifies the reviewable terminal delivery stage for the current run.
  - A duplicate trigger increments `duplicate_trigger_count` and emits trace evidence without creating a second phase, even when the later trigger name differs.

## ReviewerDefinition

- Purpose: Declares one review participant in the bounded council.
- Fields:
  - `reviewer_id`: Stable reviewer identifier for traces and task state.
  - `role`: Human-readable review responsibility such as safety, maintainability, or release-readiness.
  - `source`: Optional provider or source label used only for traceability and audit output.
  - `weight`: Positive integer used by weighted voting.
- Validation rules:
  - `reviewer_id` must be non-empty and unique.
  - `role` must be non-empty.
  - `weight` must be greater than zero.

## ReviewScenario

- Purpose: Provides the deterministic review output for one trigger in the current manifest-driven slice.
- Fields:
  - `trigger`: The `ReviewTrigger` that activates the scenario.
  - `findings`: Ordered list of `ReviewerFinding` values for the configured reviewers.
  - `adjudication_finding`: Optional `ReviewerFinding` returned by the adjudicator when adjudication is enabled.
- Validation rules:
  - Every finding must reference a configured reviewer.
  - A scenario must contain at least one finding.
  - At most one scenario may exist per trigger.
  - A scenario may define at most one adjudication finding.
  - In the initial slice, each reviewer step reads its finding from the active scenario rather than calling an external review service.

## ReviewerFinding

- Purpose: Captures one reviewer’s assessment of the delivery result.
- Fields:
  - `reviewer_id`: Reviewer that produced the finding.
  - `disposition`: One of `approve`, `concern`, or `block`.
  - `summary`: Short human-readable conclusion.
  - `details`: Optional longer rationale used for inspection output.
- Validation rules:
  - `reviewer_id` must reference a council participant.
  - `summary` must be non-empty.
  - `disposition` must map cleanly into vote resolution.
  - Malformed reviewer output is any finding missing a referenced reviewer, disposition, or non-empty summary, and must transition the review phase to a visible `failed` or `escalated` outcome.

## ReviewerParticipation

- Purpose: Tracks whether each configured reviewer actually contributed to the review result.
- Fields:
  - `reviewer_id`: Configured reviewer identifier.
  - `status`: One of `completed`, `failed`, or `omitted`.
  - `reason`: Optional failure or omission explanation.
- Validation rules:
  - Every configured reviewer must have one participation record in the final review output.
  - `failed` and `omitted` states should carry a reason when available.

## VoteRuleDefinition

- Purpose: Describes how Synod resolves reviewer findings.
- Fields:
  - `strategy`: One of `majority` or `weighted`.
  - `reject_on_blocking`: Boolean guard that allows blocking findings to force immediate rejection when configured.
- Validation rules:
  - `strategy` must be supported by the runtime.
  - Weighted voting requires at least one reviewer weight greater than zero, and the sum of weights must be computable.
  - Majority voting accepts when approval count is greater than half of completed reviewers and rejects when block count is greater than half; otherwise the result is `needs_adjudication` unless `reject_on_blocking` forces rejection first.
  - Weighted voting accepts when approval weight is greater than half of completed reviewer weight and rejects when block weight is greater than half; otherwise the result is `needs_adjudication` unless `reject_on_blocking` forces rejection first.

## VoteResolution

- Purpose: Inspectable result of applying the configured vote rule to the collected findings.
- Fields:
  - `strategy`: The applied vote strategy.
  - `participants`: Ordered list of `ReviewerParticipation` records.
  - `approvals`: Count or weight of approving findings.
  - `concerns`: Count or weight of concern findings.
  - `blocks`: Count or weight of blocking findings.
  - `decision`: Preliminary result such as `accepted`, `rejected`, or `needs_adjudication`.
- Validation rules:
  - Resolution must account for every completed reviewer finding.
  - Resolution must expose participation status for every configured reviewer.
  - `decision` must be one of the runtime-supported review decisions.

## AdjudicationDefinition

- Purpose: Configures the one bounded follow-up step used when disagreement persists.
- Fields:
  - `enabled`: Whether adjudication is available.
  - `reviewer_id`: The adjudicator identifier.
- Validation rules:
  - When enabled, `reviewer_id` must be present and must not duplicate an existing council reviewer in the initial slice.
  - Adjudication runs only when vote resolution returns `needs_adjudication`.

## ReviewDecision

- Purpose: Terminal result of the bounded review phase.
- States:
  - `accepted`
  - `rejected`
  - `escalated`
  - `failed`
- Transition rules:
  - A review phase starts from a trigger and enters `in_review`.
  - Vote resolution may terminate immediately in `accepted` or `rejected`.
  - If the vote requires adjudication and adjudication is enabled, the phase enters `adjudicating` and then terminates.
  - If reviewers fail, are unavailable, or produce malformed output, the phase terminates as `failed`.
  - If the review remains credible but no terminal accept or reject result is available because adjudication is disabled, omitted, or exhausted, the phase terminates as `escalated`.

## Session Review Projection

- Purpose: Defines the subset of review evidence surfaced through session status and next guidance.
- Fields:
  - `latest_review_trigger`: The most recent trigger that started review.
  - `latest_review_outcome`: The most recent terminal review decision.
  - `latest_review_vote`: Short rendered summary of the vote result.
  - `latest_reviewers`: Ordered list of participating reviewer identifiers or roles.
- Validation rules:
  - Review projections are optional for sessions without review activity.
  - When projected, they must remain consistent with the active task context and latest trace.
  - Serialization uses string fields for `latest_review_trigger`, `latest_review_outcome`, and `latest_review_vote`, plus a string list for `latest_reviewers`.
  - The projection is derived from the latest review phase and refreshed whenever reviewer participation, vote resolution, or terminal review state changes.
