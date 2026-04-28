# Contract: Routing Resolution and Source Attribution

## Purpose

Define how Synod resolves effective runtime/model routing when values exist at
multiple scopes and how the CLI explains that resolution to the operator.

## Resolution Precedence

Synod resolves every supported routing slot independently with this precedence:

1. Explicit CLI override for the current command
2. Workspace-local config
3. User-scoped global config
4. Built-in default

## Supported Routing Slots

- `planning`
- `implementation`
- `verification`
- `review` default
- one or more named reviewer roles
- `adjudication`

## Resolution Rules

- Each resolved value MUST include both the selected `runtime:model` pair and
  the source that supplied it.
- Named reviewer routes override the review default only for that reviewer role.
- The adjudicator route is resolved independently from the review default and
  reviewer-role routes.
- CLI overrides are ephemeral and affect only the current command unless the
  user explicitly saves them through a config mutation command.
- If no saved value exists at any scope, Synod must show the built-in default
  rather than a blank or implied route.

## Failure Rules

- If a higher-precedence value is invalid, Synod must fail explicitly instead of
  silently falling through to a lower-precedence value.
- If a route references an unavailable runtime, the resolved output must state
  that the chosen route is unavailable and why.
- If duplicate reviewer-role routes exist at one scope, Synod must reject that
  scope as invalid rather than guess which one wins.

## Output Rules

- `synod config show --scope effective` must render one human-readable table or
  list that includes slot, resolved route, availability, and source.
- Init summaries must show the initial resolved routing snapshot after applying
  any chosen global or workspace values.
- When runtime commands later consume routing, status or inspect output should
  surface the effective route when it materially affects the operator’s next
  action.