# Data Model: Delivery Flows (SDLC Backbone)

## FlowDefinition

- Purpose: Defines one built-in delivery flow that can be selected for an active session.
- Fields:
  - `name`: Stable flow identifier such as `bug-fix`, `change`, or `delivery`.
  - `display_name`: Human-readable label for CLI output.
  - `stages`: Ordered list of `FlowStageDefinition` records.
- Validation rules:
  - `name` must be unique across built-in flows.
  - `stages` must contain at least one stage.
  - Stage identifiers must be unique within a flow.
- Relationships:
  - One `FlowDefinition` owns many `FlowStageDefinition` records.
  - One `FlowDefinition` may be referenced by many `SessionFlowState` snapshots over time.
- Built-in stage sequences:
  - `bug-fix`: `investigate` -> `implement` -> `verify`
  - `change`: `understand-change` -> `implement` -> `verify`
  - `delivery`: `requirements` -> `architecture` -> `backlog` -> `implementation`

## FlowStageDefinition

- Purpose: Defines one deterministic stage in a selected delivery flow.
- Fields:
  - `id`: Stable stage identifier such as `investigate`, `implement`, `verify`, or `requirements`.
  - `display_name`: Human-readable label for CLI output.
  - `description`: Delivery intent for the stage.
  - `sequence_index`: Zero-based order within the parent flow.
- Validation rules:
  - `sequence_index` values must be contiguous and start at zero.
  - The first stage is the entry point for a selected flow.
  - The last stage defines successful flow completion when its steps complete.

## SessionFlowState

- Purpose: Persists the currently selected flow for an active session together with current stage position.
- Fields:
  - `flow_name`: Identifier of the selected `FlowDefinition`.
  - `current_stage_id`: Identifier of the active `FlowStageDefinition`.
  - `current_stage_index`: Zero-based stage index.
  - `total_stages`: Total number of stages in the selected flow.
- Validation rules:
  - This entity is optional so existing non-flow sessions remain valid.
  - If present, `flow_name` must resolve to a built-in `FlowDefinition`.
  - `current_stage_index` must be less than `total_stages`.
  - `current_stage_id` must match the stage at `current_stage_index` in the resolved flow definition.
- State transitions:
  - `None` -> initialized flow state when a user selects a flow.
  - Current stage `n` -> current stage `n + 1` when the active stage completes successfully.
  - Current stage `n` -> current stage `n` when retry or replan occurs inside the stage.
  - Flow state remains present in terminal success or failure so the finished stage path stays inspectable.

## Execution Bounds

- Purpose: Defines how flow-aware execution stays bounded without introducing a second limit system.
- Rules:
  - A flow-aware task inherits the existing task-level limits already enforced by Boundline for total step attempts, retries, and replans.
  - Stage recovery may retry or replan only while the next recovery action would stay within those existing task-level limits.
  - Stage exhaustion occurs when the next retry or replan would exceed the inherited task-level limits, or when the planner reports no credible next step for the current stage.
  - Stage exhaustion terminates the flow in a failed terminal state; it does not skip to a later stage.

## StepStageBinding

- Purpose: Associates a flat plan step with the stage it belongs to.
- Fields:
  - `step_id`: Identifier of the existing plan step.
  - `flow_name`: Selected flow identifier.
  - `stage_id`: Stage identifier for the step.
  - `stage_index`: Stage order for the step.
- Validation rules:
  - Every step in a flow-aware plan must map to exactly one stage.
  - Step order in the plan must be non-decreasing by `stage_index`.
  - Replanned steps must keep the same `stage_id` and `stage_index` as the stage being recovered.
- Persistence model:
  - `StepStageBinding` is computed when a flow-aware plan is generated.
  - The binding is stored with the step metadata inside the existing flat plan representation rather than as a separate persisted collection.
  - The binding can be reconstructed from `SessionFlowState`, the selected `FlowDefinition`, and the stage-tagged plan steps loaded from session state.

## StageProgressRecord

- Purpose: Represents inspectable evidence about the active or completed stage within a session or trace.
- Fields:
  - `flow_name`: Selected flow identifier.
  - `stage_id`: Stage identifier.
  - `stage_index`: Stage order.
  - `entered_at`: Timestamp when the stage became active.
  - `terminal_result`: Optional result summary with one of `success`, `failed`, or `incomplete`.
  - `attempt_count`: Count of step attempts executed while the stage is active.
- Validation rules:
  - `entered_at` must be recorded when a stage becomes current.
  - `terminal_result` remains empty while the stage is active and is populated only on stage completion or flow termination.
  - `attempt_count` must be non-decreasing within the stage lifetime.
  - Only `success` may advance the flow to the next stage.
  - `failed` is used when the stage exhausts its inherited execution bounds or no credible next step remains within the stage.
  - `incomplete` is used only for interrupted or partially executed stages that remain recoverable within the current flow state.

## Trace Events

- Purpose: Defines the inspectable event payloads required to understand flow lifecycle changes through persisted traces.
- Event types:
  - `flow_selected`: records `flow_name`, `current_stage_id`, `current_stage_index`, and `total_stages` when a flow is bound to a session.
  - `stage_transitioned`: records `flow_name`, `from_stage_id`, `to_stage_id`, `from_stage_index`, `to_stage_index`, and the triggering `step_id` when a stage advances.
  - `stage_retry_scheduled`: records `flow_name`, `stage_id`, `stage_index`, `step_id`, and the retry reason when recovery stays within the same stage.
  - `stage_replanned`: records `flow_name`, `stage_id`, `stage_index`, the replaced step ids, and added step ids when stage-scoped replanning occurs.
  - `stage_failed`: records `flow_name`, `stage_id`, `stage_index`, and the failure reason when the flow stops inside the stage.
- Validation rules:
  - Every `stage_transitioned`, `stage_retry_scheduled`, `stage_replanned`, and `stage_failed` event must reference the currently selected flow.
  - `stage_transitioned` may occur only after a stage reaches terminal result `success`.
  - `stage_failed` must be emitted before the flow reaches its terminal failed state.