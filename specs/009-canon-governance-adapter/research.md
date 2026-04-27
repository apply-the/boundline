# Research: Canon Governance Adapter

## Decision 1: Extend the existing workspace execution manifest with governance policy

- Decision: Add an optional `governance` section to `<workspace>/.synod/execution.json` instead of creating a second workspace manifest or storing governance policy only inside transient session state.
- Rationale: Specs 006, 007, and 008 already centralize delivery, adaptive, and review behavior inside one workspace execution profile. Governance policy belongs alongside those controls because it changes how built-in flow stages execute. Keeping one manifest preserves a single bounded runtime contract per workspace.
- Alternatives considered:
  - Create `.synod/governance.json`: rejected because it would split one delivery loop across multiple workspace contracts.
  - Persist governance configuration only inside `.synod/session.json`: rejected because policy must exist before a session starts and must be reusable across sessions.

## Decision 2: Use a local-first `GovernanceRuntime` abstraction with an optional Canon CLI adapter

- Decision: Introduce one `GovernanceRuntime` abstraction with two concrete runtimes in the first slice: `LocalGovernanceRuntime` as the default path for independent testing and `CanonCliRuntime` for optional Canon-backed stage governance.
- Rationale: Principle XVI requires Synod to remain independently testable and executable without Canon. A local-first runtime preserves that property while still enabling real Canon integration when configured.
- Alternatives considered:
  - Call Canon directly from `SessionRuntime`: rejected because it would make Canon the implicit control path and weaken local testability.
  - Add a direct Rust dependency on Canon internals: rejected because the feature explicitly scopes Canon as an external CLI adapter.

## Decision 3: Govern only meaningful built-in flow stage boundaries

- Decision: Apply governance at the current stage boundary in the built-in `bug-fix`, `change`, and `delivery` flows, using existing flow metadata to bind one Synod stage to one local or Canon governance path.
- Rationale: Flow stages already define the user-visible delivery phases Synod understands and that the roadmap mapped to Canon modes. Stage boundaries are the smallest meaningful place to govern without flooding the runtime with micro-step governance.
- Alternatives considered:
  - Govern every Synod step: rejected because it would create noisy, high-overhead control flow and break the delivery-first minimal slice.
  - Govern the entire run once at startup: rejected because it would not preserve stage-specific packet reuse, approval state, or bounded stage mappings.

## Decision 4: Persist governed stage evidence in task context and project it into session and trace surfaces

- Decision: Store governed stage state inside the existing task context state map, derive session-visible governance fields from that state, and add governance-specific trace events for lifecycle transitions.
- Rationale: Adaptive execution and review already use task context plus projections instead of introducing sidecar state files. Reusing that pattern keeps the feature consistent with the current architecture and minimizes persistence churn.
- Alternatives considered:
  - Add a second session-sidecar file for governance state: rejected because it would fragment the persisted truth for one active task.
  - Extend only `ActiveSessionRecord` top-level fields: rejected because stage packet lineage and per-stage details fit better inside the persisted task context than in one flat session record.

## Decision 5: Model autopilot as a bounded governance decision chooser, not as a freeform executor

- Decision: Autopilot may choose only from this explicit bounded action vocabulary: `select_mode`, `retry_stage_with_narrowed_context`, `escalate_verification`, `escalate_pr_review`, `await_approval`, and `block_stage`.
- Decision details: Candidate generation follows one deterministic priority for the first slice. Generate `select_mode` candidates first when a Canon-governed stage has no bound mode, preserving whitelist order and preferring `discovery` before `change` for `bug-fix:investigate`. Generate `await_approval` when the selected path requests approval. Generate at most one `retry_stage_with_narrowed_context` candidate after a failed or incomplete governed attempt when the narrowed context removes at least one read target or reused packet reference while leaving goal, risk, zone, and owner unchanged. Generate `escalate_verification` only from `implement` stages and `escalate_pr_review` only from `verify` stages that permit `pr-review`. Always retain `block_stage` as the compliant terminal fallback when governance remains required.
- Rationale: The user asked for autopilot when governance is required, but Synod's constitution forbids hidden intelligence and unbounded autonomy. Limiting autopilot to explicit policy-constrained choices preserves inspectability and prevents governance bypass.
- Alternatives considered:
  - Let autopilot make arbitrary freeform agent decisions: rejected because it would hide the real control path and could bypass governance.
  - Omit autopilot from the first slice: rejected because it would not satisfy the requested feature scope.

## Decision 6: Treat governed packet quality as a first-class acceptance gate

- Decision: A governed stage is reusable only if the resulting packet has every expected document, non-empty authored body content for each required document, and no runtime-declared missing sections. Packet scaffolding, empty documents, or explicit missing-authored-body markers are treated as incomplete and cannot satisfy stage completion or downstream reuse.
- Rationale: Canon can produce governed structure, but the feature only adds delivery value if later Synod stages can rely on packet quality. Quality gating prevents Synod from treating an empty governed shell as credible reasoning input.
- Alternatives considered:
  - Trust any successful Canon exit status: rejected because it would let structurally valid but substantively empty packets pass.
  - Require only manual operator inspection: rejected because packet reuse must be enforceable inside the runtime.

## Decision 7: Add explicit governance trace events and session fields instead of hiding governance inside generic logs

- Decision: Extend trace and session projections with governance-specific event and field names such as runtime selection, mode binding, awaiting approval, blocked outcome, packet rejection, and autopilot decision rationale.
- Rationale: Review and adaptive execution already have inspectable surfaces. Governance must meet the same bar so a developer can see why a stage proceeded, blocked, or waited without reading raw CLI logs.
- Alternatives considered:
  - Reuse only generic `StepCompleted` payloads: rejected because governance-specific reasoning would be hard to discover and validate.
  - Emit governance information only to raw stderr or tracing logs: rejected because it would bypass the user-facing `status`, `run`, and `inspect` contracts.

## Decision 8: Keep approval resolution outside Synod and refresh it through existing commands

- Decision: Synod does not author or grant approvals in the first slice. When a governance path enters `awaiting_approval`, every later `status`, `step`, or `run` invocation for that workspace refreshes the current runtime state and only allows stage continuation after approval becomes `granted`; rejected or expired approvals become explicit blocked outcomes.
- Rationale: The current CLI already exposes sequential session commands and inspectable state surfaces. Reusing those commands avoids inventing a second approval UX while still making the approval boundary explicit and testable.
- Alternatives considered:
  - Add a new Synod-native approval command in the first slice: rejected because approval authority belongs to the governance runtime, not to Synod.
  - Auto-resume immediately after external approval: rejected because Synod must preserve explicit user-visible stage transitions.

## Decision 9: Reuse governed packets through explicit upstream-stage bindings

- Decision: Downstream stages resolve the newest reusable packet from the same session whose source stage is either the same stage on rerun, the immediately previous stage in the same built-in flow, or the explicit escalation source stage for a newly opened downstream governed attempt. Synod passes only bounded packet references, headlines, and missing-section metadata into downstream stage input.
- Rationale: This creates a deterministic reuse rule without exposing the full `.canon/` tree or rebuilding stage context from scratch. It also matches the sequential flow model already used by Synod.
- Alternatives considered:
  - Expose the full governed artifact directory to every later stage: rejected because it would widen reasoning scope and reduce inspectability.
  - Require operators to manually re-attach packets: rejected because the feature's value is explicit reusable governed context.

## Decision 10: Validate Canon stage mappings at manifest load time

- Decision: The workspace execution profile validates Canon-governed stage policies against the first-slice whitelist during manifest loading. If a stage omits `canon_mode`, Synod may derive it only when exactly one whitelist mode exists for that stage; otherwise the stage remains pending explicit selection.
- Rationale: Failing fast at manifest load time keeps invalid governance policies out of the runtime loop and gives autopilot a deterministic set of compliant choices.
- Alternatives considered:
  - Defer invalid mode detection until a stage begins: rejected because it would produce late failures after session startup.
  - Allow arbitrary Canon mode strings and rely on Canon CLI errors: rejected because Synod must keep the supported governance envelope explicit.