# Research: External Capability Provider Protocol

## Provider Catalog Refresh

Public provider documentation was rechecked on 2026-06-05 as required by the
constitution:

- OpenAI's current models documentation still surfaces the GPT-5.5 and GPT-5.4
  families already represented in Boundline's bundled catalog, including the
  latest GPT-5.5 guide and the current models reference:
  [OpenAI models](https://developers.openai.com/api/docs/models).
- Anthropic's current models overview still surfaces the Claude 4.x family
  already represented in the bundled catalog, including Claude Opus 4.8,
  Claude Sonnet 4.6, and Claude Haiku 4.5:
  [Anthropic models overview](https://platform.claude.com/docs/en/about-claude/models/overview).
- Google's current Gemini models page still surfaces the Gemini 2.5 and 3.x
  family members already represented in the bundled catalog, including Gemini
  2.5 Pro, Gemini 2.5 Flash, Gemini 3.1 Pro, Gemini 3 Flash, and Gemini 3.5
  Flash:
  [Gemini models](https://ai.google.dev/gemini-api/docs/models).

Result: no feature-driven model-family change is required for this packet.
`assistant/catalog/model-catalog.toml` is refreshed to the `0.72.0` release
line during implementation closure, but the 2026-06-05 official docs check did
not require a catalog-shape delta for this feature.

## Decision 1: Keep capability providers separate from framework adapters

**Decision**: Introduce a dedicated Boundline-owned capability-provider
contract instead of reusing the framework-adapter domain vocabulary.

**Rationale**: Framework adapters can claim whole Boundline stages and own
bounded stage execution flows. Capability providers are smaller-grained bounded
tools or services that Boundline invokes inside its own execution control.
Merging them would blur ownership and make Canon or future provider profiles
look more authoritative than they should be.

**Alternatives considered**:

- Reuse the framework-adapter contract directly: rejected because stage
  ownership and capability execution are different trust and lifecycle shapes.
- Push capability providers into Canon-facing contracts: rejected because
  Canon is explicitly outside provider activation semantics.

## Decision 2: Standardize the protocol as typed JSON envelopes

**Decision**: Represent `capabilities`, `health`, `prepare`, `execute`, and
`collect_evidence` as typed JSON request/response envelopes with stable
identifiers and machine-readable failure classes.

**Rationale**: The first slice needs one contract that command and HTTP
transports can share. Typed envelopes keep Boundline validation, traces, and
host surfaces deterministic while avoiding provider-specific parser logic.

**Alternatives considered**:

- Free-form text protocol with post-hoc parsing: rejected because it would
  violate the no-hidden-intelligence rule and be hard to validate.
- Transport-specific payload shapes: rejected because the generic provider
  boundary should not vary by transport.

## Decision 3: Support command and HTTP endpoint transports in the first slice

**Decision**: The generic protocol should be transport-neutral but support two
concrete transports in the first slice: local command/stdio and HTTP endpoint.

**Rationale**: The roadmap item explicitly replaces MCP as the generic
capability boundary and must be usable by both local executables and remotely
hosted provider endpoints. Supporting both transports up front keeps later
browser, sandbox, and company-harness providers from requiring a second
protocol shape.

**Alternatives considered**:

- Command-only in the first slice: rejected because remote capability providers
  would immediately need a follow-up protocol fork.
- HTTP-only in the first slice: rejected because local executable discovery and
  explicit operator registration are already first-class roadmap requirements.

## Decision 4: Store registration metadata in Boundline config, not in traces

**Decision**: Persist provider registration and activation metadata in
Boundline-owned configuration while keeping execution evidence and validation
dispositions in session and trace state.

**Rationale**: Registration is operator intent and machine configuration, not
execution history. Traces should record which registration was used and why a
request passed or failed, but they should not become the source of truth for
which provider is active.

**Alternatives considered**:

- Store provider registrations only in traces: rejected because activation
  would be impossible to manage declaratively across runs.
- Store raw secrets in config: rejected because the feature requires secret
  handles or auth references, not prompt-visible secret values.

## Decision 5: Make setup and activation atomic

**Decision**: Registration may exist without activation, and activation must
only become authoritative after config validation, secret-handle resolution,
and dry-run health checks all succeed.

**Rationale**: An interrupted setup flow must not corrupt an already working
provider configuration. Atomic activation also gives operators a clear mental
model for when a provider is truly ready.

**Alternatives considered**:

- Mutate the active provider incrementally during setup: rejected because it
  makes interruption and rollback ambiguous.
- Treat registration as automatic activation: rejected because discoverability
  and trust are intentionally separate.

## Decision 6: Keep provider execution non-authoritative

**Decision**: Provider `execute` results may include findings, artifacts,
limitations, and patch proposals, but those remain proposals until Boundline
records an explicit validation disposition.

**Rationale**: The core principle of the roadmap item is that provider output
is not truth. If providers can mutate Boundline-owned state directly, the
protocol becomes an unbounded trust tunnel rather than a safe capability
boundary.

**Alternatives considered**:

- Let trusted providers write state directly: rejected because it bypasses the
  validation and evidence rule that the spec requires.
- Strip patch proposals from the protocol entirely: rejected because they are
  still valuable signals when treated as proposals.

## Decision 7: Represent admission and validation failures explicitly

**Decision**: Boundline should classify provider lifecycle failures as
readiness failures, permission admission failures, execution failures, or
post-execution validation failures, and persist the class in runtime-visible
state.

**Rationale**: The spec promises that operators can understand what failed from
`status` or `inspect` without opening raw trace files. Stable failure classes
are the minimum contract needed to satisfy that promise.

**Alternatives considered**:

- One generic provider failure bucket: rejected because it hides where
  corrective action belongs.
- Trace-only failure classes with no status projection: rejected because it
  weakens operator recovery.

## Decision 8: Fail closed on metadata conflicts

**Decision**: When provider metadata, specialized profile metadata, and
Boundline runtime policy disagree, the stricter Boundline runtime policy wins
and provider-backed execution fails closed before execution starts.

**Rationale**: Provider overlays are intentionally subordinate to the generic
protocol. A fail-open or silently merged policy would create invisible
privilege escalation and make runtime decisions hard to trust.

**Alternatives considered**:

- Let specialized profile metadata override generic metadata: rejected because
  it would bypass the explicit conflict rule in the spec.
- Try to merge conflicts heuristically: rejected because the result would be
  harder to explain and validate.

## Decision 9: Reuse existing auth and transport primitives

**Decision**: Reuse the existing `reqwest` HTTP client path, auth-profile
storage, config-store patterns, and runtime projection helpers already present
in the workspace instead of introducing a second transport or secret storage
stack.

**Rationale**: The repository already knows how to perform bounded HTTP calls,
persist non-secret config, and surface provider readiness in CLI output. The
new slice should add the provider protocol, not a parallel infrastructure
subsystem.

**Alternatives considered**:

- Add a new transport dependency or secret-management library: rejected because
  the first slice can reuse current primitives.
- Encode provider setup entirely in assistant assets: rejected because the
  runtime, not assistant prompts, must own provider activation.
