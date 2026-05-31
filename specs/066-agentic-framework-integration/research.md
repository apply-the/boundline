# Research: Agentic Framework Integration

## Provider Catalog Refresh

Public provider docs were rechecked on 2026-05-31 as required by the
constitution.

- OpenAI's current models page still leads with `gpt-5.5`, `gpt-5.4`,
  `gpt-5.4-mini`, and `gpt-5.4-nano` for general-purpose coding and reasoning.
- Anthropic's current Claude models overview still leads with Claude Opus 4.8,
  Claude Sonnet 4.6, and Claude Haiku 4.5 as the current comparison set.
- Google's Gemini models page still lists Gemini 3.1 Pro Preview,
  Gemini 3.5 Flash, Gemini 3 Flash Preview, Gemini 3.1 Flash-Lite,
  Gemini 2.5 Pro, Gemini 2.5 Flash, and Gemini 2.5 Flash-Lite among the active
  text-generation families.

The bundled catalog in `assistant/catalog/model-catalog.toml` was updated on
2026-05-30 and already carries those current families for the repo's supported
runtime surfaces, so this planning slice requires no catalog delta.

## Decision 1: Keep Canon-aware behavior as the built-in default and make external adapters opt-in only

**Decision**: Preserve the existing Canon-aware path as the built-in default
whenever no external adapter is selected, and treat every external framework
adapter as an explicit workspace-level opt-in.

**Rationale**: The feature must preserve Boundline's out-of-the-box delivery
value. Operators who never register an adapter should continue to use the same
goal/plan/run flow without extra installation or behavior changes. This also
keeps the product boundary explicit: Boundline remains the orchestrator, while
external adapters only override declared stages or receive declared hook events.

**Alternatives considered**:

- Externalize Canon into an adapter too: rejected because it would make the
  default product path depend on an external binary.
- Prefer a known adapter automatically when it is discovered on `PATH`: rejected
  because the spec requires explicit operator selection.
- Require an adapter for all lifecycle runs: rejected because it would break the
  baseline delivery experience and violate the default-behavior requirements.

## Decision 2: Add an explicit adapter-management command family backed by workspace config

**Decision**: Introduce a dedicated host-owned adapter-management surface with
`boundline adapter add`, `boundline adapter show`, and `boundline adapter remove`,
and make `.boundline/config.toml` the authoritative persisted source for the
active adapter selection and resolved adapter-specific values. Guided `init`
flows should call the same underlying registration logic rather than inventing a
second setup path.

**Rationale**: The spec requires explicit registration and activation through
initialization or adapter-management surfaces, guided interactive setup, and
deterministic non-interactive failure when required fields are missing. A
dedicated command family is clearer than hiding adapter registration behind
generic config mutation, and reusing the same service from `init` avoids drift
between first-run bootstrap and later maintenance.

**Alternatives considered**:

- Reuse only `boundline config set` with raw keys: rejected because it makes the
  workflow hard to discover and too error-prone for known profiles.
- Put all registration only inside `boundline init`: rejected because operators
  need to add, change, or remove adapters after workspace bootstrap.
- Persist adapter state outside `.boundline/config.toml`: rejected because the
  workspace config already owns authoritative lifecycle preferences.

## Decision 3: Use one-shot subprocess commands with typed JSON over stdin/stdout

**Decision**: Use a one-shot subprocess protocol over JSON stdin/stdout rather
than a long-lived daemon. The host invokes bounded commands such as `describe`,
`preflight`, `execute-stage`, and `emit-hook`, with request and response bodies
modeled as typed serde structs owned by the Boundline contract.

**Rationale**: One-shot commands fit the constitution's sequential-first and
bounded-execution rules better than a resident adapter process. They make host
timeouts, retry policy, and failure ownership explicit, simplify test harnesses,
and let the template repo ship a minimal binary scaffold without background
process management.

**Alternatives considered**:

- Long-lived JSON-RPC session or daemon: rejected because it adds connection
  lifecycle, reconnection, and hidden-state complexity before the first useful
  slice ships.
- Dynamic libraries or ABI-loaded plugins: rejected because they create fragile
  compatibility and distribution concerns across repositories.
- Ad hoc shell-script execution: rejected because it would not provide the typed
  capability, config-schema, and stage-outcome contracts the host needs.

## Decision 4: Keep the stage and hook catalog host-owned and reject undeclared or unknown claims

**Decision**: Boundline owns the stable lifecycle stage catalog and hook-event
catalog. Adapters may only claim stage overrides and hook subscriptions from
that catalog, and the host rejects unknown or unsupported stage/hook IDs during
capability parsing or preflight.

**Rationale**: The spec requires selective stage overrides, ignored undeclared
hooks, and actionable feedback for malformed capability declarations. A host-
owned catalog keeps the default lifecycle authoritative and prevents external
repos from silently inventing new control-flow semantics.

**Alternatives considered**:

- Let adapters define arbitrary stage IDs: rejected because it would let external
  repos redefine the lifecycle surface invisibly.
- Treat unknown stage IDs as no-ops: rejected because malformed capabilities
  must block activation with actionable feedback.
- Hard-code the catalog only in documentation: rejected because the host needs a
  typed runtime validator and test fixtures.

## Decision 5: Put shared protocol types and golden fixtures in `boundline-adapters`

**Decision**: Extend `crates/boundline-adapters` with the framework-adapter
protocol types, validation helpers, and golden JSON fixtures, and have the
sibling template and Speckit repos consume that crate through a versioned git-tag
dependency pinned to Boundline releases.

**Rationale**: The current workspace already exposes member crates intended for
shared consumption. Reusing `boundline-adapters` avoids copying request/response
types across repos, while a git-tag dependency keeps the initial slice small and
avoids committing local path dependencies or standing up new package-publishing
infrastructure immediately.

**Alternatives considered**:

- Duplicate protocol structs in each repo: rejected because drift would be likely
  and expensive to test.
- Commit a path dependency from sibling repos into this workspace: rejected
  because it would leak local machine topology into versioned files.
- Publish a new dedicated crate before the first slice lands: rejected because a
  versioned git dependency is sufficient for the initial compatibility line.

## Decision 6: Split ownership cleanly across the three repositories

**Decision**: Keep host runtime, config, lifecycle routing, audit surfaces,
known-profile metadata, and release documentation in the Boundline repo;
bootstrap `../boundline-framework-template/` as the reusable adapter scaffold;
and keep all Speckit-specific logic, tests, and release docs in
`../boundline-adapter-speckit/`.

**Rationale**: The spec explicitly separates Boundline core, the reusable
template, and the Speckit adapter. The current repo states reinforce that split:
`../boundline-framework-template/` is still an empty Git repo that needs a first
real scaffold, while `../boundline-adapter-speckit/` exists but only contains an
initial README-level commit. Keeping the boundaries explicit prevents template or
Speckit implementation details from leaking into the host runtime.

**Alternatives considered**:

- Keep the template inside this repository: rejected because the clarified scope
  says reusable adapter-template work belongs in the sibling repo.
- Implement Speckit inside this repository: rejected because the clarified scope
  says Speckit lives in its own sibling repo.
- Delay the template repo until after Speckit ships: rejected because custom
  adapters are part of the initial business framing and need a reusable starting
  point.

## Decision 7: Ship a first-class known `speckit` profile with explicit discovery hints

**Decision**: Boundline should ship a known profile definition for adapter ID
`speckit`, default binary `boundline-adapter-speckit`, and registration command
`boundline adapter add speckit`. That profile includes display metadata,
discovery hints, default command resolution, links to the sibling adapter and
template repos, and any fixed fields the guided setup flow can prefill.

**Rationale**: The user explicitly identified Speckit as the known external
adapter for this slice. Treating it as a first-class profile makes setup faster,
lets `init` and `adapter add` prefill bounded defaults, and still preserves the
spec's requirement that activation remains operator-controlled.

**Alternatives considered**:

- Treat Speckit as a custom adapter only: rejected because it would throw away
  the known-profile requirement and force avoidable manual setup.
- Auto-activate Speckit when the binary exists: rejected because discoverability
  must not become activation.
- Store the known-profile definition only in the Speckit repo: rejected because
  the host CLI must know how to guide setup before the adapter has been selected.

## Decision 8: Make hook delivery observable but non-owning unless a stage has already been claimed

**Decision**: Hook subscriptions stay additive and observable. A hook delivery
failure records a hook-level warning or error in session and trace state, but it
does not retroactively transfer stage ownership or fail a built-in stage unless
the adapter had already claimed the current stage through a declared override.

**Rationale**: The spec makes stage-claim failure semantics strict, but it does
not require hook-only observers to become stage owners. Keeping hook delivery
additive preserves the smallest useful slice and avoids turning optional
observability hooks into surprising stop-the-world controls.

**Alternatives considered**:

- Treat every hook failure as a lifecycle failure: rejected because it would make
  non-owning hooks disproportionately risky.
- Hide hook failures completely: rejected because observability and actionable
  feedback are required.
- Disable hooks in v1: rejected because hook subscriptions are an explicit
  requirement.

## Decision 9: Carry cross-repo compatibility through an explicit protocol line and version range

**Decision**: Add explicit compatibility metadata to the host-known profile and
the adapter capability manifest: a stable protocol line such as
`framework-adapter-v1`, the adapter's own version, and a supported Boundline
version range. The template repo and Speckit repo should document the same line
and pin their `boundline-adapters` dependency to released Boundline tags.

**Rationale**: The three repositories release independently, so the plan needs a
clear place to express when a given adapter build is safe to run against a given
Boundline version. A visible protocol line keeps release notes, docs, and runtime
diagnostics aligned without forcing lockstep releases.

**Alternatives considered**:

- Assume the latest adapter always works with the latest Boundline: rejected
  because it hides compatibility risk and gives operators no recovery path.
- Enforce lockstep releases across all three repos: rejected because it is too
  rigid for the initial slice and not required for safety.
- Express compatibility only in README text: rejected because the host needs a
  machine-readable gate for preflight and diagnostics.