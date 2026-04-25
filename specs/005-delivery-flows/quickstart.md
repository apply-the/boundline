# Quickstart: Delivery Flows (SDLC Backbone)

## Prerequisites

- Work from the repository root.
- Use a clean workspace with no existing `.synod/session.json`, or start a fresh session before each scenario.
- Provide a workspace fixture manifest at `.synod/fixture.json` so `plan`, `step`, and `run` have a deterministic red-to-green target.
- Run commands through `cargo run --bin synod -- ...` when validating locally.

## Scenario 1: Bug-fix flow

1. Start a new session:

   ```bash
   cargo run --bin synod -- start
   ```

2. Capture a bounded repair goal:

   ```bash
   cargo run --bin synod -- capture --goal "Fix failing checkout tests"
   ```

3. Select the bug-fix flow:

   ```bash
   cargo run --bin synod -- flow bug-fix
   ```

4. Create the flow-aware plan:

   ```bash
   cargo run --bin synod -- plan
   ```

5. Execute until completion:

   ```bash
   cargo run --bin synod -- run
   ```

6. Verify the final state:

   ```bash
   cargo run --bin synod -- status
   cargo run --bin synod -- inspect --workspace .
   ```

Expected outcome:

- `status` shows `bug-fix` as the active flow.
- Stage progress advances from `investigate` to `implement` to `verify`.
- `inspect` reveals flow selection and stage transition events in the latest trace.

Example status excerpt:

```text
active_flow: bug-fix
current_stage: verify
stage_progress: 3/3
next_command: synod inspect
```

## Scenario 2: Change flow with stage-aware next guidance

1. Start and capture a change goal:

   ```bash
   cargo run --bin synod -- start
   cargo run --bin synod -- capture --goal "Add a confirmation email after checkout"
   ```

2. Select and plan the change flow:

   ```bash
   cargo run --bin synod -- flow change
   cargo run --bin synod -- plan
   ```

3. Execute one step:

   ```bash
   cargo run --bin synod -- step
   ```

4. Ask for the next action:

   ```bash
   cargo run --bin synod -- next
   ```

Expected outcome:

- `status` and `next` both show the selected flow and active stage.
- The recommended next command remains valid for the current stage.

Example next excerpt:

```text
active_flow: change
current_stage: implement
stage_progress: 2/3
next_command: synod step
```

## Scenario 3: Failure stays inside the current stage

1. Start a bug-fix or delivery flow and plan it with a fixture that allows at least one retry.
2. Trigger a retryable or replannable failure during execution.
3. Re-run `status`, `next`, and `inspect`.

Expected outcome:

- The current stage does not advance on failure.
- Retry or replan guidance remains scoped to the same stage.
- Trace output records the failure and any stage-preserving recovery decision.

Example trace summary excerpt:

```text
stage_retry: implement
stage: implement -> verify
```

## Regression Scenario: Existing non-flow usage still works

1. Start a new session.
2. Capture a goal without selecting a flow.
3. Run `plan`, `step`, `status`, and `run` as before.

Expected outcome:

- Commands succeed without requiring flow selection.
- Status output omits flow fields cleanly when no flow is active.