# Research: Human-Facing Brief Ingestion

## Decision 1: Extend the existing `capture` and direct-input `run` surfaces instead of adding a new ingest command

- Decision: Reuse the existing human-facing entry points by changing `capture` and direct-input `run` to accept optional direct text plus repeated Markdown brief paths and business-level governance flags. The first-slice CLI contract is `--goal <text>` plus repeated `--brief <path>` with optional `--governance <local|canon>`, `--risk <value>`, `--zone <value>`, and `--owner <value>`. Assistant-driven usage must call the same normalization path with inline text and resolved file references.
- Rationale: Boundline already teaches `start`, `capture`, `plan`, and `run` as the bounded delivery loop. Reusing those commands removes manifest authoring without adding another concept to learn, preserves existing goal-only usage, and keeps governance expressed in human business terms rather than a second JSON-shaped payload.
- Alternatives considered:
  - Add a new `ingest` command: rejected because it duplicates the role of `capture` and would fragment the entry surface for the same bounded workflow.
  - Require governance JSON on the CLI: rejected because it reintroduces machine-shaped authoring after the spec explicitly removed JSON from the normal human path.
  - Accept only text that mentions file paths: rejected because explicit repeated `--brief` flags are more reliable and auditable for normal CLI usage.

## Decision 2: Normalize all accepted human input into one persisted `AuthoredBriefBundle`

- Decision: Convert direct text, explicit Markdown briefs, text-referenced workspace documents, and optional governance intent into one ordered, inspectable `AuthoredBriefBundle` persisted alongside the active task and projected into session-visible summaries.
- Rationale: Boundline already persists bounded execution state through `ActiveSessionRecord` and `TaskContext.state`. A normalized bundle lets later `plan`, `status`, `next`, `inspect`, and `run` commands reuse the accepted authored input without restating it and without inventing a new workspace-sidecar manifest.
- Alternatives considered:
  - Persist a new `.boundline/briefs.json`: rejected because it adds another user-visible contract and splits one active task across multiple files.
  - Store only the raw `goal` string: rejected because it loses provenance, source order, deduplication, and governance intent required by the spec.
  - Use only `task.input`: rejected because it is the internal execution payload and does not by itself give a durable, inspectable human-authored source model.

## Decision 3: Resolve source precedence and deduplication deterministically inside the workspace boundary

- Decision: Resolve explicit `--brief` paths first in command order, resolve Markdown paths mentioned inside direct text second in first-mention order, canonicalize every file path against the workspace root, accept only `.md` and `.markdown` sources in the first slice, and deduplicate repeated documents by canonical workspace-relative path while preserving the first accepted precedence.
- Rationale: The feature must make source choice visible and reproducible. Canonical workspace-relative paths give stable provenance, explicit ordering keeps merges understandable, and workspace-only Markdown resolution keeps the slice bounded and independently testable.
- Alternatives considered:
  - Merge text and files silently into one blob: rejected because it hides precedence and makes conflicts hard to inspect.
  - Accept arbitrary file types in the first slice: rejected because the spec bounds the initial capability to human-authored text and Markdown.
  - Permit outside-workspace paths: rejected because it weakens bounded execution and makes provenance and safety harder to reason about.

## Decision 4: Treat clarification as an explicit blocking state instead of guessing missing structure

- Decision: When Boundline cannot derive a credible bounded task because the brief is vague, source material conflicts, referenced files are missing, or required business governance values are absent, it records one explicit `ClarificationRecord`, reports only the missing external information, and stops before planning or execution. The first slice allows at most two clarification turns before explicit stop.
- Rationale: The constitution forbids hidden intelligence and the spec forbids asking for internal nouns. An explicit clarification state keeps the stop condition inspectable and prevents Boundline from inventing task scope or governance wiring.
- Alternatives considered:
  - Infer missing context heuristically and continue: rejected because it would hide a critical product decision and reduce trust.
  - Ask the user for internal stage IDs or manifest fields: rejected because that would violate the human-facing requirement.
  - Continue with warnings only: rejected because bounded delivery requires a credible start condition before planning.

## Decision 5: Keep manifest-driven execution as the advanced path while deriving the same internal runtime input from the human path

- Decision: Preserve `<workspace>/.boundline/execution.json` and legacy `.boundline/fixture.json` as advanced or automation-only entry points, while the new human path derives the same internal task request and optional governance overlay from the normalized brief bundle without forcing those files to exist.
- Rationale: Existing tests, assistant packs, and automation already rely on the manifest-driven path. Keeping it intact avoids breaking mature flows, while the human path removes the need for users to author those files manually.
- Alternatives considered:
  - Always write a synthetic execution manifest to disk from human input: rejected because it leaks internal representation back into the normal workflow.
  - Replace manifest-driven execution entirely: rejected because the spec explicitly keeps the advanced manifest path for automation and tests.

## Decision 6: Map human governance intent through the existing governance runtime abstraction instead of exposing stage wiring

- Decision: Treat `--governance`, `--risk`, `--zone`, and `--owner` as business-level intent that is mapped into the existing governance runtime and stage policy machinery only after human input normalization. Canon remains optional, and missing governance values trigger clarification only when governed execution is requested and the missing value blocks credible execution.
- Rationale: The codebase already has a local-first `GovernanceRuntime` abstraction and `input_documents` support. Reusing that path lets Boundline expose governed execution in human terms while keeping Canon optional and hiding stage IDs, packet references, and Canon mode wiring.
- Alternatives considered:
  - Expose stage IDs and Canon modes directly on the normal CLI path: rejected because it forces users to learn internal nouns.
  - Make Canon mandatory whenever governance is requested: rejected because the constitution requires Boundline to remain independently usable without Canon.
  - Drop governance from the first slice: rejected because the spec includes governed human-facing runs as a required scenario.

## Decision 7: Reuse existing `status`, `next`, `inspect`, and trace surfaces for input observability

- Decision: Extend the current session and trace projections with input summary, resolved source provenance, clarification headline, and human governance intent rather than introducing a new inspection command or raw-log-only debugging path.
- Rationale: Boundline already has status and inspect surfaces that developers use to understand execution state. Keeping input provenance on those same surfaces satisfies inspectability requirements and avoids creating another user workflow just to see what brief Boundline accepted.
- Alternatives considered:
  - Emit provenance only to tracing logs: rejected because it would hide important decisions from normal CLI usage.
  - Add a dedicated `brief inspect` command: rejected because the current status and inspect surfaces already own explainability for the active session.

## Decision 8: Validate the slice through CLI contract, session persistence, and governance-projection tests

- Decision: Cover the feature with contract tests for CLI validation and command output, integration tests for session capture-to-plan flow with Markdown inputs and clarification stops, and unit tests for source normalization, deduplication, and governance intent mapping.
- Rationale: The feature changes the human entry surface and persisted state. Contract tests lock the command shape, integration tests prove that the active session remains resumable, and unit tests keep precedence, deduplication, and clarification rules deterministic.
- Alternatives considered:
  - Rely only on end-to-end tests: rejected because most failure modes are in argument validation and normalization logic.
  - Rely only on unit tests: rejected because session continuity and status projection are core delivery behavior, not isolated helpers.