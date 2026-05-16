# CLI Contract: Guided Init UX And Diagnostic Messaging

## Guided Init Prompt Contract

`boundline init` in guided mode must expose these user
visible guarantees:

1. Assistant selection prompt lists supported values inline.
2. Route prompt states that the field is optional when defaults are available.
3. Route prompt lists supported slot names.
4. Route prompt includes at least one valid `SLOT=RUNTIME:MODEL` example.
5. Successful init summary reports seeded routes, explicit overrides, and where
   the operator can inspect or change them later.

## Validation Error Contract

For malformed route input, unsupported assistant values, unavailable default
capability, overwrite conflicts, or non-interactive limitations:

1. The error must name the failing input or state.
2. The error must describe the expected shape or requirement in plain language.
3. The error must provide at least one corrective action, example, or retry
   command.
4. The command must exit non-zero without mutating the workspace state.

## Summary Layout Contract

For `init` and `doctor` on the primary first-run path:

1. Output must be grouped into semantically understandable sections.
2. Rich formatting may be used only when the terminal supports it.
3. Plain text mode must preserve identical meaning and next-step guidance.
4. Assistant scaffolding status must be visible when assistant surfaces are
   selected.

## Compatibility Contract

1. Non-interactive init flags remain supported and automation-safe.
2. Existing `.boundline` compatibility/bootstrap files remain part of the init
   flow.
3. Repository-local assistant scaffolding remains bounded to the active
   workspace root.