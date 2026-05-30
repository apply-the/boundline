# Implementation Plan: S7 Assistant Delight Layer

**Branch**: `060-assistant-delight-layer` | **Date**: 2026-05-17 | **Spec**: [./spec.md](./spec.md)
**Input**: Feature specification from `/specs/060-assistant-delight-layer/spec.md`

## Summary

S7 (Assistant Delight and Cognitive Affordance Layer) is the Boundline-side
runtime and assistant-surface implementation that makes the session-native
runtime feel immediately useful across CLI, chat hosts, and inspect views.
This feature implements the commands, inspect lenses, assistant package assets,
and fallback behavior promised by the active roadmap while consuming governed
Canon inputs only through Canon's `057-s7-delight-provider` contract.

**Primary requirement**: Deliver a compact, assistant-native S7 layer that:
- Adds cognitive affordance surfaces for `why`, `risk`, `assumptions`,
  `hidden-impact`, `challenge`, `evidence`, `next-best`, `explain-plan`, and
  `doctor-context`
- Uses existing session state, traces, inspect/status output, and workspace
  evidence as the authoritative runtime substrate
- Consumes Canon provider signals only from the active 057 delight-provider
  contract and surfaces degradation explicitly when those signals are missing or
  incompatible
- Keeps the default package palette compact and contextual rather than noisy
- Preserves governance boundaries, stop semantics, and review requirements
  instead of bypassing them

**Technical approach**: Extend the existing session-native and assistant asset
surfaces in four coordinated layers:
1. **Assistant command assets**: add S7 command definitions and host prompts
   under `assistant/`
2. **CLI and inspect projections**: extend `inspect`, `output`, and diagnostic
   rendering so S7 has authoritative runtime-backed explanations
3. **Capability disclosure and fallback**: make missing Canon, missing
   advanced-context, or setup gaps visible with recommended next commands
4. **Cross-repo alignment**: validate the Boundline consumer behavior against
   Canon 057 provider semantics and degradation rules

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024 for runtime and CLI changes;
JSON and Markdown for assistant asset metadata and prompts  
**Primary Dependencies**: Existing Boundline stack (`clap`, `serde`,
`serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) only; no new runtime
crates planned for the first S7 slice  
**Storage**: Existing workspace-local `.boundline/session.json`, trace files
under `.boundline/traces/`, optional advanced-context artifacts already shipped,
and repository-managed assistant assets under `assistant/`  
**Testing**: Unit tests in `tests/unit/`, contract tests in `tests/contract/`,
integration flows in `tests/integration/`, plus cross-repo Canon 057 alignment
review  
**Target Platform**: macOS/Linux developer workstations and supported assistant
host packages (Claude, Codex, Copilot prompt packs, Cursor, global bootstrap
assets)  
**Project Type**: Rust workspace CLI/runtime with repository-managed assistant
package assets  
**Execution Model**: Runtime implementation feature that changes authoritative
CLI/inspect output and assistant command assets  
**Observability Surface**: `status`, `inspect`, trace summaries, package command
metadata, and explicit fallback or degradation text rendered to operators  
**Performance Goals**: First useful `why` or `risk` answer within five minutes
of bootstrap on a partial setup; no new remote dependency requirement for the
initial answer path  
**Constraints**:
- Must consume Canon only through the active `057-s7-delight-provider` contract
- Must remain useful without Canon project memory or advanced-context indexes
- Must preserve the session-native runtime as the source of truth
- Must keep the default assistant palette compact and avoid global command bloat
- Must not introduce new governance semantics, new councils, or hidden fallback
  logic
**Scale/Scope**: One Boundline runtime slice covering CLI-backed S7 lenses,
assistant package assets, and contextual diagnosis for the S7 roadmap story

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Delivery identity (PASS)**: S7 improves delivery by making active runtime
  state explainable, inspectable, and actionable for bounded engineering work.
- **Delivery-first scope (PASS)**: The initial slice focuses on useful runtime
  answers and setup diagnosis, not on UI spectacle or speculative intelligence.
- **Primary workflow (PASS)**: The session-native path remains primary; S7 reads
  from that loop and does not invent a separate runtime.
- **Bounded execution (PASS)**: Cognitive commands operate on the active
  session, current traces, current workspace evidence, and bounded Canon inputs.
- **Stateful execution (PASS)**: S7 reads current session and trace state and
  renders explicit updates from that state.
- **Mutable planning (PASS)**: `challenge`, `why`, `risk`, and `next-best` make
  mutable planning visible without hiding replans or stop conditions.
- **Sequential-first design (PASS)**: The feature adds explanation surfaces over
  the existing sequential runtime rather than concurrent orchestration.
- **Tool-agent symmetry (PASS)**: Assistant commands map to the real Boundline
  runtime and `inspect`/`status` surfaces, not to hidden reasoning-only paths.
- **Observability and explicit intelligence (PASS)**: S7 answers must cite
  evidence, confidence, degradation, and next actions explicitly.
- **Catalog currency (PASS)**: Existing assistant host catalog remains current;
  no bundled model changes are required for the initial S7 slice.
- **Non-goals and external separation (PASS)**: Canon stays the provider of
  governed input, not the owner of Boundline assistant UX or runtime logic.
- **Minimal slice (PASS)**: The first slice delivers concrete commands and
  inspect lenses before deeper S6/S8 follow-on work.

**Gate Result**: ✅ **PASS** — The feature fits Boundline's delivery-first and
session-native constitutive constraints.

## Project Structure

### Documentation (this feature)

```text
specs/060-assistant-delight-layer/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── assistant-delight-contract.md
│   ├── assistant-delight-explanation-vocabulary.md
│   ├── assistant-delight-explanation-vocabulary.toml
│   ├── assistant-delight-degradation-modes.md
│   ├── assistant-delight-extension-procedures.md
│   └── assistant-delight-input-classes.schema.json
└── tasks.md
```

### Runtime And Assistant Surfaces

```text
assistant/
├── commands/session-workflow.json
├── plugin-metadata.json
├── global/manifest.json
├── README.md
├── claude/commands/
├── codex/commands/
└── copilot/prompts/

src/cli/
├── assistant_assets.rs
├── diagnostics.rs
├── inspect.rs
├── output.rs
├── run.rs
└── session.rs

tests/
├── unit/
│   ├── assistant_assets.rs
│   ├── cli_output.rs
│   ├── context_intelligence_projection.rs
│   └── session_cli_runtime.rs
├── contract/
│   ├── assistant_command_definition_contract.rs
│   ├── assistant_command_pack_contract.rs
│   ├── global_assistant_install_contract.rs
│   ├── host_command_output_contract.rs
│   └── trace_summary_contract.rs
└── integration/
    ├── cli_trace_inspection.rs
    ├── distribution_doctor_flow.rs
    ├── global_assistant_bootstrap.rs
    └── host_trace_runtime_flow.rs
```

**Structure Decision**: This feature ships real runtime and assistant-surface
changes. It extends the existing `assistant/` asset pipeline and `src/cli/`
projection modules instead of creating a parallel dashboard or new runtime.
Canon alignment remains external and is consumed through existing runtime and
contract tests.

## Complexity Tracking

- `src/cli/output.rs` and `src/cli/inspect.rs` are the likely complexity
  hotspots because they already own flattened operator-facing rendering.
- `assistant/plugin-metadata.json` and `assistant/commands/session-workflow.json`
  are the likely package-noise hotspots because S7 adds many commands.
- Mitigation: keep the initial slice focused on explicit command definitions,
  bounded inspect lenses, and contextual visibility instead of a large freeform
  assistant framework.

---

## Phase 0 Design Inputs

Existing design inputs from the previous contract-definition work remain useful
and become implementation inputs rather than the feature outcome:

- **Canon input classes** and degradation semantics remain in `contracts/`
- **Explanation vocabulary** remains the operator-facing terminology source
- **Data model** remains the projection model for source-attributed answers
- **Canon 057 provider contract** stays the external dependency that defines
  which governed inputs Boundline may consume

The plan for 060 is therefore not to redefine those boundaries again, but to
use them to implement the actual S7 runtime and assistant surfaces.

## Phase 1 Design Focus

Implementation work should resolve these design slices before touching many
files at once:

1. **Command-surface mapping**: which S7 commands are always visible, which are
   contextual, and how they map to real Boundline CLI/inspect behavior
2. **Projection mapping**: how `inspect`, `status`, trace summaries, and
   diagnostics feed `why`, `risk`, `evidence`, `next-best`, `assumptions`, and
   `challenge`
3. **Capability disclosure**: how missing Canon, missing project memory, or
   missing advanced-context capability becomes explicit operator text
4. **Cross-host packaging**: how Claude, Codex, Copilot, Cursor, and global
   bootstrap assets stay aligned without duplicating semantics manually
5. **Cross-repo validation**: how Boundline runtime tests and manual review stay
   aligned with Canon 057 provider semantics while Canon remains in progress

## Next Steps: Phase 2 Task Execution

The next action is to execute the runtime implementation tasks in
`/specs/060-assistant-delight-layer/tasks.md` in this order:

1. Add failing assistant-command and inspect-surface tests
2. Implement the MVP commands (`why`, `risk`, `evidence`, `next-best`) across
   assistant assets and CLI-backed output
3. Implement deeper cognitive affordances (`assumptions`, `hidden-impact`,
   `challenge`, `explain-plan`) with explicit fallback disclosure
4. Implement `doctor-context`, package contextualization, and compact palette
   rules
5. Finish with Canon 057 alignment review, docs updates, complexity review,
   clippy, coverage, and formatting
