# Research: Canon Governance Expansion

**Feature**: 017-canon-governance-expansion  
**Date**: 2026-04-29

## R1: The first slice should add `security-assessment`, not full Canon mode parity

**Decision**: Extend Synod's Canon governance surface with `security-assessment` in the first slice and explicitly defer real `supply-chain-analysis` support and the rest of Canon's unsupported mode roster.

**Rationale**: `security-assessment` is the newest governance-hardened Canon mode that can deepen Synod's delivery-facing governance without introducing the larger clarification and tool-availability UX implied by `supply-chain-analysis`. It adds immediate value while preserving the smallest independently useful slice.

**Alternatives Considered**:
- Add both `security-assessment` and `supply-chain-analysis` immediately: rejected because the supply-chain slice widens scope into clarification and external tool posture.
- Add all missing Canon modes at once: rejected because it would turn the feature into a broad mode-parity rewrite rather than a bounded governance expansion.

## R2: Expand stage-to-mode selection on existing verification stages instead of changing the flow model

**Decision**: Keep Synod's built-in `bug-fix`, `change`, and `delivery` flows unchanged and extend the targeted verification stages so they may route through `security-assessment` as a governed Canon mode.

**Rationale**: The existing flow model is a stronger boundary than the Canon mode roster. Reusing the current `verify` stage preserves the session-native operator story and avoids reopening the built-in flow model just to support one newer governed analysis mode.

**Alternatives Considered**:
- Add new top-level Synod flow stages for security analysis: rejected because it expands the flow model before the underlying governance model needs it.
- Treat `security-assessment` as an out-of-band compatibility workflow only: rejected because that would weaken the session-native runtime story.

## R3: Reuse the existing Canon start and refresh lifecycle instead of introducing a new governance protocol

**Decision**: Keep Synod's Canon integration anchored on the current `governance start` and `governance refresh` request/response contract.

**Rationale**: Synod already performs real Canon start and refresh calls, packet-readiness validation, and approval refresh. The next slice should widen what those calls can represent, not add a second governance protocol or hidden background behavior.

**Alternatives Considered**:
- Introduce a new Canon-specific session sidecar: rejected because existing session and trace state already hold the required lifecycle data.
- Poll Canon asynchronously in the background: rejected because the constitution requires explicit, inspectable command-driven execution.

## R4: Keep Canon packet reuse bounded to packet references and summary metadata

**Decision**: Preserve the current bounded packet reuse model by carrying only packet refs, packet headlines, readiness, and missing-section metadata into later governance context and operator surfaces.

**Rationale**: The governed analysis packet should improve downstream reasoning without letting later steps inspect or depend on the full `.canon/` artifact tree. This matches Synod's existing bounded-context posture and keeps the flow inspectable.

**Alternatives Considered**:
- Expose the entire Canon packet tree to later steps: rejected because it widens context and weakens inspectability.
- Avoid packet reuse entirely: rejected because the governed packet should be reusable evidence inside bounded session workflows.

## R5: Operator surfaces should expose the selected Canon mode and governance condition through the existing session-native summaries

**Decision**: Extend `run`, `status`, `next`, and `inspect` so the selected Canon mode, approval or blocked state, packet provenance, and next action appear through the same session-native summary model already used for routing and execution condition.

**Rationale**: The product value is not just deeper governance; it is deeper governance that remains understandable in the same operator workflow. Reusing the session-native summary model avoids creating a second mental model for governed analysis.

**Alternatives Considered**:
- Emit the new mode only in traces or raw payloads: rejected because the operator would have to reconstruct the state manually.
- Add a Canon-only CLI surface: rejected because it would split the product story.

## R6: Update the Canon compatibility target to `0.25.0`, but treat the governance delta as `0.20.0` through `0.24.0`

**Decision**: Update Synod's documented Canon compatibility target to `0.25.0` while explicitly basing the governance-expansion design on the released Canon mode and authoring changes from `0.20.0` through `0.24.0`.

**Rationale**: Canon `0.25.0` primarily adds distribution-channel work. The governance-relevant feature delta that motivates this Synod slice is the newer governed analysis and authoring surface added between `0.20.0` and `0.24.0`.

**Alternatives Considered**:
- Treat `0.25.0` itself as the governance feature: rejected because it does not add a materially new governance runtime capability for Synod.
- Leave the compatibility target at `0.24.0`: rejected because the user's environment already relies on Canon `0.25.0` and Synod should document the validated target honestly.