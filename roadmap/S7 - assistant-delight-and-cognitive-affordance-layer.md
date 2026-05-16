# Assistant Delight And Cognitive Affordance Layer

## Status

Proposed Specification

## Scope

Boundline-first, Canon-aware

## Position In The Spec Stack

This is Spec 7.

It depends on the conceptual foundation of:

- S1 Runtime Intelligence Substrate
- S2 Domain Expert Packs And Runtime Role Composition
- S3 Authority-Zoned Delivery Roles, Personas, And Review Councils
- S4 Control Graduation And Adaptive Governance
- S5 Advanced Context Intelligence
- S6 Advanced Multi-Agent Reasoning Profiles

However, this specification must deliver value even before every deep runtime capability is complete.

---

# 1. Outcome

Boundline gains an assistant-facing delight layer that makes the runtime feel immediately useful, intelligent, legible, and emotionally trustworthy from chat, CLI, and IDE assistant surfaces.

The goal is not to add more governance.

The goal is to expose Boundline's intelligence through low-friction cognitive affordances that help a developer understand:

- what is happening
- why it matters
- what is risky
- what is missing
- what to do next
- what the assistant is assuming
- what should be challenged

This layer turns deep runtime capability into visible product value.

---

# 2. Product Thesis

A powerful runtime that feels heavy will lose to a simpler assistant that feels helpful.

Boundline and Canon can become structurally superior to prompt-pack systems, but they still need an immediate experience layer that feels:

- fast
- useful
- explainable
- safe
- concrete
- low-friction
- assistant-native

The user should feel value before they understand the architecture.

This specification exists to prevent:

```text
too much architecture before delight
```

The target experience is:

```text
install
→ ask
→ understand
→ act safely
→ inspect why
```

---

# 3. Design Principles

## 3.1 Runtime Depth Must Surface As Simple Questions

Users should not have to understand:

- authority zones
- context packs
- councils
- Canon contracts
- project memory
- adaptive governance
- expert packs
- retrieval layers

to get value.

They should be able to ask:

```text
Why is this risky?
What are we assuming?
What could break?
What should I do next?
What evidence do we have?
What is missing?
Can I safely proceed?
```

Boundline should answer using its runtime substrate and Canon-governed context.

---

## 3.2 Delight Must Not Bypass Governance

This layer may summarize, explain, preview, and suggest.

It must not:

- bypass stop semantics
- override governance
- silently downgrade risk
- hide missing evidence
- replace deterministic validation
- fake confidence
- turn restricted work into a chat shortcut

---

## 3.3 Fast Path Matters

The experience must make the safe path feel faster than the unsafe path for ordinary work.

If every command feels like an enterprise committee, the product has failed.

---

# 4. User-Facing Cognitive Commands

Boundline should expose a compact set of high-value cognitive affordance commands.

These should be available in:

- CLI
- chat assistant commands
- IDE assistant surfaces where supported

The naming below is illustrative and may be adapted to host conventions.

---

## 4.1 `/boundline:why`

Explains why Boundline is making or recommending a decision.

Examples:

```text
/boundline:why this plan?
/boundline:why this reviewer?
/boundline:why did you stop?
/boundline:why is this red?
```

Output should include:

- decision
- evidence
- missing evidence
- relevant Canon artifacts
- runtime signals
- confidence level
- next action

---

## 4.2 `/boundline:risk`

Explains risk posture.

Examples:

```text
/boundline:risk this change
/boundline:risk current plan
/boundline:risk changed files
```

Output should include:

- authority zone
- change class
- affected surfaces
- public contract exposure
- security implications
- migration or data concerns
- recommended council profile
- stop semantics if applicable

---

## 4.3 `/boundline:assumptions`

Lists assumptions currently influencing plan or execution.

Output should group assumptions by:

- product assumption
- domain assumption
- architecture assumption
- implementation assumption
- validation assumption
- governance assumption

Each assumption should include:

```text
status: explicit | inferred | missing | contradicted
source: user | Canon | workspace | trace | model inference
risk: low | medium | high
```

---

## 4.4 `/boundline:hidden-impact`

Finds likely indirect impact.

Examples:

```text
/boundline:hidden-impact this PR
/boundline:hidden-impact src/auth/tokens.rs
```

Output should include:

- affected domains
- affected capabilities
- likely invariants
- relevant tests
- missing tests
- downstream services
- required reviewers

This command may use structured indexes initially and advanced context intelligence later.

---

## 4.5 `/boundline:challenge`

Asks Boundline to challenge the current plan, change, packet, or diff.

Output should include:

- strongest objection
- weakest assumption
- missing evidence
- likely failure mode
- reviewer roles that should challenge it
- whether a council is required

This command must not replace formal review councils for governed work.

---

## 4.6 `/boundline:evidence`

Shows what evidence exists and what is missing.

Output should include:

- tests
- traces
- Canon packets
- review findings
- verification results
- project memory refs
- stale or conflicting evidence

---

## 4.7 `/boundline:next-best`

Suggests the next safe action.

Output should include:

- recommended next action
- why this is next
- blocked alternatives
- expected command
- confidence
- required approval or review if any

---

## 4.8 `/boundline:explain-plan`

Explains the current plan in human terms.

Output should include:

- goal
- stages
- risks
- assumptions
- expected files or systems
- validation plan
- governance gates
- rollback or recovery posture

---

## 4.9 `/boundline:doctor-context`

Diagnoses missing setup, context, or readiness.

Output should include:

- missing Canon project memory
- missing Boundline config
- missing expert packs
- missing runtime routes
- missing provider readiness
- missing evidence
- stale indexes
- suggested fix commands

---

# 5. Assistant Surface Strategy

## 5.1 Always-Available Bootstrap

The user must be able to access a bootstrap command even when the workspace is not initialized.

Recommended names:

```text
/boundline:init
/boundline:doctor
```

These commands should be globally installed where host platforms allow global assistant commands.

If host platforms do not support global commands, Boundline must provide a documented fallback:

```text
boundline assistant install --host <host> --scope user
boundline init
```

---

## 5.2 Minimal Command Palette

The default visible assistant surface should be compact.

Recommended always-visible core:

```text
/boundline:init
/boundline:start
/boundline:capture
/boundline:plan
/boundline:run
/boundline:status
/boundline:next
/boundline:inspect
/boundline:recover
```

Recommended cognitive affordances:

```text
/boundline:why
/boundline:risk
/boundline:assumptions
/boundline:hidden-impact
/boundline:challenge
/boundline:evidence
/boundline:next-best
```

Advanced or contextual commands should not pollute the default palette.

---

## 5.3 Contextual Commands

Commands should become visible or recommended based on context.

Examples:

- show `/boundline:recover` after failed execution
- show `/boundline:evidence` when validation is weak
- show `/boundline:challenge` before red-zone work
- show `/boundline:hidden-impact` for schema/API/auth changes
- show `/boundline:doctor-context` when setup is incomplete

---

# 6. Instant Value Flows

## 6.1 Five-Minute First Value

A new user should be able to:

```text
install
→ initialize or diagnose
→ ask for risk
→ get a useful answer
```

within five minutes.

Minimum first-value scenario:

```text
User: /boundline:risk current change

Boundline:
- detects changed files
- infers likely surfaces
- reports risk and missing evidence
- suggests next safe action
```

This must work even before the full governance stack is configured.

---

## 6.2 Zero-Config Advisory Mode

If no project memory exists, Boundline should still provide advisory output.

It must say:

```text
Project memory is missing. This answer is based on repository signals only.
```

No silent confidence inflation.

---

## 6.3 Graceful Capability Disclosure

When features are unavailable, Boundline should explain:

- what is missing
- why it matters
- how to enable it
- what fallback is being used

Example:

```text
Advanced impact graph is not available.
Using structured file and Canon artifact signals instead.
Run `boundline index build` to improve results.
```

---

# 7. Product Personality

Boundline should feel:

- direct
- useful
- concrete
- skeptical
- bounded
- calm under uncertainty

It should not feel:

- theatrical
- mystical
- overconfident
- bureaucratic
- swarmy
- verbose by default

The assistant should prefer:

```text
Here is what I know.
Here is what I do not know.
Here is why it matters.
Here is the next safe action.
```

---

# 8. Explainability Templates

## 8.1 Decision Explanation Template

```text
Decision:
Evidence:
Missing Evidence:
Risk:
Confidence:
Next Safe Action:
```

---

## 8.2 Risk Explanation Template

```text
Risk Level:
Authority Zone:
Change Class:
Affected Surfaces:
Why It Matters:
Required Review:
Stop Conditions:
```

---

## 8.3 Assumption Template

```text
Assumption:
Source:
Status:
Risk:
How To Verify:
```

---

## 8.4 Hidden Impact Template

```text
Possible Impact:
Evidence:
Affected Area:
Confidence:
Recommended Reviewer:
Recommended Check:
```

---

# 9. Canon Integration

This layer may consume Canon:

- project memory
- domain language
- domain model
- architecture packets
- verification packets
- review findings
- security assessments
- promotion refs
- packet readiness

Canon remains the governed knowledge source.

Boundline remains the assistant-facing runtime.

Canon must not become responsible for assistant delight or command UX.

---

# 10. Boundline Runtime Integration

This layer should consume:

- S1 context packs
- S2 selected experts and role metadata
- S3 authority zones and council profiles
- S4 governance state, confidence, degradation, and escalation
- S5 retrieval signals where available
- S6 reasoning profiles only when explicitly active

It must never become a separate runtime.

It is a projection and interaction layer over the runtime.

---

# 11. Inspectability UX

`inspect` should become the beautiful surface for runtime understanding.

It should support views such as:

```text
inspect context
inspect risk
inspect assumptions
inspect evidence
inspect council
inspect timeline
inspect blockers
inspect next
```

The goal is to make runtime state navigable without reading raw traces.

---

# 12. Assistant Host Packages

Boundline should ship assistant packages for:

- Codex
- Claude
- Copilot
- Gemini CLI where applicable
- Cursor where applicable

Each package should expose:

- minimal core commands
- cognitive affordance commands
- host-specific command metadata
- global bootstrap path if supported
- workspace-local fallback path

---

# 13. Success Metrics

Product success should be measured by:

- time to first useful answer
- number of commands needed before first value
- setup failure clarity
- percentage of risk explanations with evidence refs
- percentage of assumptions classified by source
- user correction rate
- recovery success after failed setup
- command palette noise
- inspect usage
- next-action acceptance rate

---

# 14. Non-Goals

This specification does NOT:

- define new governance semantics
- replace S1-S6 runtime layers
- add new council algorithms
- add new reasoning profiles
- replace Canon project memory
- bypass stop semantics
- create another plugin system
- force all commands to be visible by default
- turn Boundline into a chat-only product

---

# 15. Acceptance Criteria

The implementation is complete when:

- Boundline exposes a compact assistant command surface
- bootstrap and doctor commands are available globally where hosts support it
- cognitive affordance commands produce useful output from partial setup
- risk, assumptions, evidence, hidden impact, and why explanations are available
- commands clearly disclose missing context and fallback mode
- inspect surfaces present runtime state in human-friendly views
- assistant packages remain compact and not noisy
- cognitive commands never bypass governance
- Canon remains the governed knowledge source
- Boundline feels immediately useful before the user understands the architecture

---

# 16. Final Thesis

Boundline's deep runtime will not matter if users cannot feel its intelligence quickly.

This specification makes Boundline legible, useful, and trusted at the assistant surface.

The goal is:

```text
deep runtime
simple questions
useful answers
safe action
```

The product should feel smart because it can explain, challenge, and guide, not because it speaks in abstractions.
