# JavaScript And TypeScript Language Guidance

Use JavaScript and TypeScript to make runtime boundaries explicit instead of letting flexible syntax hide domain mistakes.

- Prefer strict TypeScript settings and avoid `any`, weak object bags, and ambient mutation.
- Validate runtime input at API or persistence boundaries with explicit schemas before it reaches domain logic.
- Model state with discriminated unions, branded identifiers, and narrow object shapes instead of scattered boolean flags.
- Keep reusable business rules in services or utilities, not buried inside components, hooks, or route handlers.
- Use effects only to synchronize with external systems; do not store derived values that can be computed directly.
- Keep async control flow explicit and surface failures with typed result objects or clearly mapped errors.
- Avoid hidden shared mutable state, implicit singleton caches, and prototype-based surprises in core logic.
