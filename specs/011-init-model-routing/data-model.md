# Data Model: Human-Friendly Init and Model Routing

## WorkspaceInitProfile

- Purpose: Represents the user-facing setup choice that seeds a repository with
  bounded Boundline files and optional assistant scaffolding.
- Fields:
  - `profile_id`: Stable identifier for the selected init template.
  - `template_kind`: `bug_fix`, `change`, or `delivery`.
  - `workspace_ref`: Absolute or canonical workspace reference used for file generation.
  - `selected_runtimes`: Ordered set of supported runtimes the operator chose to enable.
  - `assistant_setup_enabled`: Whether repo-local assistant assets should be scaffolded or refreshed.
  - `planned_changes`: Ordered list of `InitChangePreview` values.
  - `confirmation_state`: `pending`, `confirmed`, or `aborted`.
  - `created_at`: Millisecond timestamp for the init attempt.
- Validation rules:
  - `template_kind` must be one of the supported built-in templates.
  - `selected_runtimes` may be empty only when init is used purely to scaffold workspace files without runtime setup.
  - `confirmation_state = confirmed` requires at least one planned change.
  - All planned file writes must remain inside the workspace root.

## RuntimeCapability

- Purpose: Captures whether a supported runtime is available and ready for configuration on the current machine.
- Fields:
  - `runtime_id`: `claude`, `codex`, `copilot`, or `gemini`.
  - `transport_kind`: `native_client`, `cli`, or `extension_surface`.
  - `availability`: `available`, `partially_available`, or `missing`.
  - `detected_command`: Optional executable or surface reference that satisfied detection.
  - `missing_requirements`: Ordered list of unmet prerequisites.
  - `notes`: Optional operator-facing explanation.
- Validation rules:
  - Gemini must use `transport_kind = cli` in this slice.
  - `availability = available` requires either a detected command or a known extension surface.
  - `missing_requirements` must be non-empty when availability is not `available`.

## ModelRoute

- Purpose: Represents one runtime/model choice for a delivery or review slot.
- Fields:
  - `runtime_id`: Selected supported runtime.
  - `model_id`: User-visible model identifier.
  - `temperature_profile`: Optional qualitative behavior hint if the runtime supports it.
  - `notes`: Optional operator-facing description.
- Validation rules:
  - `runtime_id` must be one of the supported runtimes.
  - `model_id` must be non-empty.
  - A route may be saved only if the runtime is valid for the target slot.

## ReviewRoleRoute

- Purpose: Overrides review routing for one review participant.
- Fields:
  - `role_id`: Stable identifier for the reviewer or adjudicator slot.
  - `role_kind`: `reviewer` or `adjudicator`.
  - `default_route`: `ModelRoute` selected for that role.
  - `weight`: Optional review weight mirror for reviewer roles.
- Validation rules:
  - `role_id` must be unique within one review routing configuration.
  - `role_kind = adjudicator` may appear at most once per routing configuration.
  - Reviewer weights, when present, must stay consistent with the bounded review profile.

## RoutingConfiguration

- Purpose: Stores one human-editable set of routing defaults for either global or workspace scope.
- Fields:
  - `config_version`: Schema version for persisted config.
  - `scope`: `global` or `workspace`.
  - `planning_route`: Optional `ModelRoute`.
  - `implementation_route`: Optional `ModelRoute`.
  - `verification_route`: Optional `ModelRoute`.
  - `review_default_route`: Optional `ModelRoute`.
  - `review_role_routes`: Ordered list of `ReviewRoleRoute` overrides.
  - `assistant_runtimes`: Ordered list of runtimes enabled for repository-local assistant scaffolding.
  - `updated_at`: Millisecond timestamp.
- Validation rules:
  - Duplicate review role identifiers are invalid.
  - Workspace-scoped config may omit values that should inherit from global scope.
  - Global config must not contain workspace-specific file paths.

## EffectiveRoutingSnapshot

- Purpose: Materializes the resolved routing values after applying precedence.
- Fields:
  - `planning`: Resolved route and `ConfigValueSource`.
  - `implementation`: Resolved route and `ConfigValueSource`.
  - `verification`: Resolved route and `ConfigValueSource`.
  - `review_default`: Resolved route and `ConfigValueSource`.
  - `review_roles`: Ordered list of resolved review-role routes with sources.
  - `adjudication`: Optional resolved route and source.
  - `assistant_runtimes`: Resolved assistant runtime set and sources.
  - `resolved_at`: Millisecond timestamp.
- Validation rules:
  - Every resolved slot must identify exactly one source.
  - CLI-supplied overrides are ephemeral and must not mutate stored config unless the user runs a config mutation command.

## ConfigValueSource

- Purpose: Explains where one resolved value came from.
- Fields:
  - `source_kind`: `cli`, `workspace`, `global`, or `built_in`.
  - `source_path`: Optional file path or command context.
  - `source_key`: Optional logical config key.
- Validation rules:
  - `source_kind = cli` should reference the invoking command context rather than a file path.
  - File-backed sources must point only to the global or workspace config locations.

## InitChangePreview

- Purpose: Describes one file or config mutation init is proposing before confirmation.
- Fields:
  - `target_path`: Target file path.
  - `change_kind`: `create`, `update`, or `skip`.
  - `summary`: Human-facing explanation of the change.
  - `destructive`: Whether the change would overwrite or replace existing content.
- Validation rules:
  - `destructive = true` requires explicit user confirmation before apply.
  - `target_path` must remain inside the active workspace for repo-local changes.

## Relationships

- One `WorkspaceInitProfile` contains zero or more `RuntimeCapability` observations.
- One `WorkspaceInitProfile` proposes one or more `InitChangePreview` values.
- One `RoutingConfiguration` may exist at global scope and one at workspace scope.
- One `EffectiveRoutingSnapshot` is derived from CLI overrides plus zero or one workspace config plus zero or one global config.
- One `RoutingConfiguration` may contain multiple `ReviewRoleRoute` values.

## Persistence Notes

- Global and workspace routing configs should stay human-editable and separate from the execution manifest.
- Init preview state is transient, but applied results should be visible through CLI output and, when relevant, status or trace surfaces.
- Review-role routing must remain compatible with the existing bounded review profile model rather than replacing it wholesale.