# Contract: Stage Mode Expansion

## Purpose

Define how the first bounded Canon governance expansion widens Boundline's supported stage-to-mode surface without changing the built-in flow model.

## Rules

### Rule 1: Built-in flow names remain unchanged

The first slice MUST keep Boundline's existing built-in flows:

- `bug-fix`
- `change`
- `delivery`

No new top-level flow family or new stage id is introduced in this slice.

### Rule 2: The bounded expansion is explicit

The first slice may add only the explicitly approved newer Canon mode:

- `security-assessment`

All other currently unsupported Canon modes remain invalid unless a later feature explicitly reprioritizes them.

### Rule 3: Security analysis is limited to credible existing-system verification stages

The first slice MUST only allow `security-assessment` on the targeted verification stages that can satisfy existing-system context credibly.

### Rule 4: Unsupported mode requests fail explicitly

If a stage binds a Canon mode outside the bounded expansion, Boundline MUST reject it explicitly instead of forwarding it as unchecked Canon configuration.

### Rule 5: The model must stay extensible for `supply-chain-analysis`

The widened mode-selection and operator-surface model MUST leave room for a later `supply-chain-analysis` slice without requiring a new top-level flow family.

## Acceptance Examples

### Supported verify-stage mapping

**Given** a `bug-fix:verify` stage with Canon governance enabled

**When** Boundline validates the stage policy

**Then** `security-assessment` may be selected when the stage satisfies the bounded expansion rules

### Unsupported operational mode

**Given** a stage policy that requests `incident` or `migration`

**When** Boundline validates the policy

**Then** it rejects the configuration explicitly because those modes are outside the current slice

### Future-compatible model

**Given** the first slice has already added `security-assessment`

**When** a later feature adds `supply-chain-analysis`

**Then** the operator-surface and mode-selection model can widen without changing Boundline's top-level flow names