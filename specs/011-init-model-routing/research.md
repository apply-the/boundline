# Research: Human-Friendly Init and Model Routing

## Decision 1: Add a dedicated `boundline init` command instead of overloading `doctor`, `start`, or `capture`

- Decision: Introduce a first-class `boundline init` command that prepares workspace files, detects supported runtimes, previews proposed changes, and optionally scaffolds repository-local assistant assets. Keep `doctor`, `start`, `capture`, `plan`, and `run` focused on readiness and delivery flow after setup exists.
- Rationale: Setup is a distinct operator journey from delivery execution. Reusing existing commands would keep leaking internal setup concerns into commands that should remain delivery-focused, while a dedicated init surface can be both guided and explicit.
- Alternatives considered:
  - Overload `doctor` to create missing files: rejected because diagnostics should remain read-mostly and explain readiness rather than mutating the repository implicitly.
  - Overload `start` to scaffold workspace files: rejected because session start is a runtime concern, not repository bootstrap.
  - Keep manual manifest authoring and only improve docs: rejected because usability is a product requirement, not a documentation workaround.

## Decision 2: Keep `.boundline/execution.json` as the bounded execution contract, but move human-editable preferences into TOML config files

- Decision: Continue generating and consuming `.boundline/execution.json` for bounded execution policy and add human-editable TOML config files for user preferences and routing. The workspace-local config lives at `.boundline/config.toml`. The user-scoped global config lives at `$XDG_CONFIG_HOME/boundline/config.toml` with fallback to `$HOME/.config/boundline/config.toml`.
- Rationale: Execution policy is already encoded, tested, and backward compatible as JSON. The operator pain comes from having to hand-author setup, not from the runtime reading JSON internally. Using TOML for new editable preferences makes the user-facing config easier to inspect and reason about while preserving the existing execution engine contract.
- Alternatives considered:
  - Replace execution.json entirely with a new user-facing format: rejected because it would force a wider migration and risk breaking existing automation.
  - Store all new preferences inside execution.json: rejected because it keeps user-facing configuration trapped inside a machine-shaped file and mixes operator defaults with runtime policy.
  - Use YAML for new config: rejected because TOML maps cleanly to Rust serde usage and is already familiar to the Cargo-centered developer audience.

## Decision 3: Resolve configuration with explicit precedence and show the source of every resolved value

- Decision: Use deterministic precedence `CLI input > workspace-local config > user-scoped global config > built-in defaults`, and make the effective resolved value inspectable from the CLI together with its source.
- Rationale: Global installation only becomes trustworthy when developers can override it per repository and still understand why a given value won. Showing the source of each resolved value prevents the feature from becoming another hidden-intelligence surface.
- Alternatives considered:
  - Workspace config always overrides everything, including explicit CLI input: rejected because it removes operator control at the point of use.
  - Merge config files silently without source attribution: rejected because it hides the most important usability decision in the feature.
  - Support only one config location: rejected because the user explicitly wants reusable defaults plus local overrides.

## Decision 4: Model routing should separate delivery stages from review participants and keep vote resolution deterministic

- Decision: Add routing slots for planning, implementation, verification, review default, per-reviewer overrides, and adjudication. Review councils may assign different runtime/model profiles to different reviewer roles, but vote resolution remains the existing deterministic logic rather than becoming model-driven.
- Rationale: The user explicitly wants voting differentiated from the rest of the delivery flow. The current review model already distinguishes reviewer identities and adjudication, so adding routing slots matches the existing bounded review abstraction without inventing a new decision engine.
- Alternatives considered:
  - One model profile for all delivery and review activity: rejected because it removes the main value of the feature.
  - Model-driven vote resolution: rejected because the current review engine treats voting as deterministic policy, and changing that would broaden scope far beyond routing.
  - Arbitrary free-form stage names for routing: rejected because the first slice should align with existing delivery and review surfaces rather than invent a generalized workflow DSL.

## Decision 5: Detect runtime capability locally and treat Gemini as CLI-only in the first slice

- Decision: Detect supported runtimes through local capability probes for Claude, Codex, Copilot, and Gemini, and expose Gemini only as a CLI-backed runtime in this slice. Runtime availability is advisory for setup and validation, and missing runtimes block only the routes that require them.
- Rationale: The feature must stay independently usable and not depend on remote control planes or hidden probes. The user also explicitly noted that Gemini currently has only CLI support, so the system should represent that limitation honestly.
- Alternatives considered:
  - Pretend all supported runtimes are equally available once configured: rejected because that hides actual operator prerequisites.
  - Add a richer Gemini client abstraction now: rejected because the user asked for feature completeness in usability, not speculative runtime framework expansion.
  - Block init entirely when no runtime is installed: rejected because the workspace can still be scaffolded and made ready for a later runtime install.

## Decision 6: Repository-local assistant setup should be opt-in, bounded, and rooted inside the active repository

- Decision: Let init offer repository-local assistant scaffolding for the selected supported surfaces, limited to files under the active repository root. Reuse and update the existing `assistant/` asset tree for Claude, Codex, and Copilot, and add a Gemini CLI surface that follows the same repository-local command-pack pattern where applicable.
- Rationale: Some runtimes only become usable in practice when the repository carries the right command packs or prompts. Keeping setup opt-in and repository-bounded preserves trust while still reducing manual setup burden.
- Alternatives considered:
  - Always overwrite assistant assets during init: rejected because it is destructive and surprising.
  - Keep assistant setup completely manual: rejected because the user explicitly asked for repo setup support when needed.
  - Write setup files outside the repository: rejected because it weakens inspectability and violates bounded workspace expectations.

## Decision 7: Introduce dedicated `boundline config` inspection and mutation commands rather than asking users to edit files directly

- Decision: Add CLI surfaces to inspect, set, and unset config values at global or workspace scope, including effective routing resolution and review-role assignments.
- Rationale: Human-friendly configuration is not complete if the user must drop into a text editor for every change. A CLI-managed path also makes validation and precedence visible at the same time.
- Alternatives considered:
  - Document manual file editing as the main workflow: rejected because it reproduces the same usability failure that motivated `boundline init`.
  - Hide mutation behind init only: rejected because routing needs to evolve after first setup.
  - Support only one bulk import/export command: rejected because normal users need small targeted changes more often than full config rewrites.

## Decision 8: Treat docs and assistant guidance as part of the feature contract, not as optional polish

- Decision: Update README, getting-started, review-voting, adaptive-execution references where needed, assistant command-pack docs, and add a dedicated configuration guide so the documented workflow matches the shipped init and config behavior.
- Rationale: This feature is mostly about usability. If the documentation still teaches manual JSON editing as the normal path, the product remains effectively incomplete even if the code ships.
- Alternatives considered:
  - Update only README: rejected because `docs/getting-started.md` and review-oriented docs currently carry more detailed workflow guidance.
  - Defer assistant doc updates to a later slice: rejected because assistant-driven usage is part of the feature’s declared support surface.