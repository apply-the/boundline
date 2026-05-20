# Research: Interactive Delivery Dashboard

## Decision 1: Isolate The Dashboard In A Dedicated Workspace Component

**Decision**: Implement the interactive dashboard in a dedicated workspace component while keeping shared dashboard data contracts in the core domain and state assembly in adapters.

**Rationale**: The dashboard needs terminal rendering and input dependencies that should not become part of the normal CLI path. A separate component preserves the existing command surface as authoritative and avoids turning the dashboard into a second runtime.

**Alternatives considered**:

- Put all dashboard code in the normal CLI crate. Rejected because it couples interactive dependencies and rendering state to the primary scriptable command surface.
- Build an external tool outside the workspace. Rejected because it would make shared state contracts harder to test and release with Boundline.
- Make the dashboard read raw command output only. Rejected because human-readable output is not a stable data contract.

## Decision 2: Use Typed Snapshots Instead Of Parsing Human Output

**Decision**: Add typed dashboard snapshot and event projections over existing session, trace, checkpoint, finding, and governed-reference state.

**Rationale**: The dashboard must match authoritative runtime truth and remain testable. Typed projections can be contract-tested, serialized for fixtures, and reused by both interactive and degraded paths without depending on fragile text parsing.

**Alternatives considered**:

- Parse `status` and `inspect` text. Rejected because formatting changes would break the dashboard and hide schema drift.
- Read every underlying file directly from the dashboard. Rejected because it would duplicate state interpretation rules already owned by Boundline.
- Introduce a dashboard database. Rejected because the feature explicitly forbids a separate state store.

## Decision 3: Reuse Existing Runtime Boundaries For Actions

**Decision**: Dashboard actions become bounded action requests that validate the current session revision and then invoke the same Boundline-owned behavior as the normal command path.

**Rationale**: Confirm, reject, replan, recover, launch, and continue must produce the same session and trace outcomes regardless of whether they are initiated from the dashboard or from a command. Revision checks prevent stale UI state from applying invalid actions.

**Alternatives considered**:

- Let the dashboard mutate session files directly. Rejected because it would bypass runtime validation and trace behavior.
- Add dashboard-only action semantics. Rejected because it creates a second control plane.
- Allow queued background actions. Rejected because the constitution requires sequential-first, inspectable delivery.

## Decision 4: Use Mature Terminal UI Rendering With Explicit Fallbacks

**Decision**: Use a mature terminal UI stack for the interactive component and require a degraded non-interactive fallback that points operators back to normal Boundline commands when rendering or input support is unavailable.

**Rationale**: The feature needs keyboard-driven panels, stable layout tests, colored text, and terminal-size fallback behavior. Degraded fallback keeps Boundline usable in CI, limited terminals, and environments where interactive rendering cannot start.

**Alternatives considered**:

- Build raw terminal rendering manually. Rejected because it increases layout and input complexity without delivery value.
- Use wide ANSI art as the brand source. Rejected because it is too wide for ordinary terminals and conflicts with the agreed simple wordmark.
- Require an interactive terminal for all dashboard behavior. Rejected because diagnostics and fallback are part of the operator trust story.

## Decision 5: Keep Governed References Read-Only And Optional

**Decision**: The dashboard displays governed artifact and project-memory references only when existing Boundline state already exposes them. Missing, stale, unreadable, or incompatible governed references become explicit degraded facts, not blockers for dashboard operation.

**Rationale**: Boundline must remain independently usable and must not require Canon runtime changes for this feature. Read-only projection preserves the ownership boundary while making available governed context visible.

**Alternatives considered**:

- Require governed metadata for dashboard launch. Rejected because ordinary Boundline delivery must work without Canon.
- Add Canon-specific dashboard write actions. Rejected because this feature is a Boundline operator surface and not a Canon runtime change.
- Hide missing governed references. Rejected because invisible fallback weakens trust.

## Decision 6: Reconcile The Assistant Model Catalog During Release Closure

**Decision**: Treat the bundled assistant model catalog as needing release-time reconciliation before this feature closes.

**Rationale**: Public provider documentation reviewed on 2026-05-19 shows likely drift from the bundled catalog: GitHub Copilot lists newer Codex choices and retirements, Anthropic lists current Claude Opus, Sonnet, and Haiku options, Google lists Gemini 3.1 and Gemini 3 families with deprecation notes, and OpenAI lists current GPT-5.5, GPT-5.4 variants, and GPT-5-Codex.

**Alternatives considered**:

- Leave the catalog unchanged because the dashboard does not change routing. Rejected because the constitution requires catalog currency evidence for every feature.
- Update only docs without changing the catalog. Rejected unless implementation confirms no runtime catalog delta is required after detailed reconciliation.
- Defer catalog reconciliation to a later feature. Rejected because the release checklist requires the delta or no-change rationale in this feature line.

## Decision 7: Make Branding A Static Terminal Wordmark

**Decision**: Render a simple colored `boundline` ASCII wordmark with a plain fallback. Do not use image assets, SVG rendering, or wide ANSI banner art in the first dashboard screen.

**Rationale**: Static text branding is reliable across terminals, easy to test, and consistent with the operator-first dashboard goal. It avoids heavy image conversion and width-sensitive artifacts.

**Alternatives considered**:

- Render the full banner as ANSI art. Rejected because the available banner art is too wide for normal terminals.
- Render SVG assets directly. Rejected because terminal dashboard rendering should not depend on image processing.
- Omit branding entirely. Rejected because a small wordmark helps identify the operator surface without compromising utility.
