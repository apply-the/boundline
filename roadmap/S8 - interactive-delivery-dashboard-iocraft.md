# S8 - Interactive Delivery Dashboard (iocraft)

This specification outlines the creation of a dedicated interactive dashboard for Boundline, built using the `iocraft` framework. This dashboard provides a real-time, high-density visualization of the Boundline Pilot Loop, session state, and governance posture.

## Motivation

As Boundline evolves into a project-scale delivery orchestrator, the current linear terminal output (via `println!` and `dialoguer`) becomes insufficient for monitoring complex, multi-step agent workflows.

An interactive dashboard (TUI) allows the operator to:
1. **Monitor the Pilot Loop** (`observe -> decide -> act -> verify`) in real-time without scrolling through log history.
2. **Visualize "Stop Rules"** and blocking conditions (e.g., "Insufficient Context", "Approval Required") in a persistent status area.
3. **Inspect the "Context Pack"** and "Negotiated Delivery" artifacts during the planning phase.
4. **Perform Side-by-Side Reviews** of proposed code mutations before confirmation.
5. **Navigate Canon Artifacts** and governance packets through a structured, interactive explorer.

## Architectural Decision: Standalone Product (Separate Crate)

The dashboard will be implemented as a separate crate: `boundline-dashboard`, producing a dedicated binary.

- **Standalone Completeness:** The dashboard is not just a viewer; it is a complete product. It will embed all core functionalities, including `init`, `doctor`, `config`, and the full delivery workflow.
- **Dual-Binary Strategy:**
    - `boundline` (or `boundline-cli`): The slim, high-performance CLI optimized for CI/CD, scripting, and quick terminal interactions.
    - `boundline-dashboard`: The immersive, interactive environment for heavy engineering sessions.
- **Decoupling:** Isolates TUI-specific dependencies (like `iocraft`, `taffy`, and terminal backends) from the automation-focused CLI.

## Key Features (Full Lifecycle)

- **Interactive Init Wizard:** A rich, component-based `init` experience that replaces the current linear prompts with a structured, visual configuration dashboard.
- **Pilot Monitor:** Displays the active step, agent role, and real-time execution logs.
- **State Sidebar:** Shows the current `session.json` state, active `Checkpoint`, and `Authority Zone`.
- **Decision Panel:** Visualizes the `GoalPlan` rationale and the next recommended action.
- **Review Canvas:** A rich interface for diff review and plan confirmation.
- **Notification Toast/Alerts:** For critical "Stop Rule" triggers or required human intervention.

### Example Component Structure (Conceptual)

```rust
element! {
    Box(flex_direction: FlexDirection::Column, width: Percent(100), height: Percent(100)) [
        Header(title: "BOUNDLINE DASHBOARD", version: "0.60.0"),
        Box(flex_direction: FlexDirection::Row, flex_grow: 1.0) [
            MainView(active_workflow: session.workflow),
            Sidebar(
                context_credibility: session.context_credibility,
                stop_rules: session.active_stop_rules,
            )
        ],
        Footer(shortcuts: ["(c) confirm", "(r) replan", "(q) quit"])
    ]
}
```

## Integration with boundline-cli

The main `boundline` executable will provide a new command:

```bash
boundline dashboard
```

This command will:
1. Verify terminal capabilities.
2. Launch the `boundline-dashboard` runtime.
3. Connect to the local `.boundline/session.json` and watch for filesystem changes to update the UI reactively.

## Success Criteria

- The dashboard provides a real-time view of the Pilot Loop without screen flickering.
- "Stop Rules" are visually prominent and explain the blocking reason clearly.
- Plan confirmation through the dashboard is more intuitive than the current text-based prompts.
- The `boundline-dashboard` crate adds zero overhead to the standard `boundline` binary size when not in use (via feature flags or separate binary).
