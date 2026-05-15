# Domain Language

Ubiquitous language aligns code vocabulary with business concepts. When code and domain experts use the same terms, misunderstandings decrease and the model becomes self-documenting.

## Naming

Name types, functions, and modules using the language of the business domain. Avoid generic names that obscure intent.

Prefer `OrderFulfilled` over `StatusChanged`.
Prefer `ShippingAddress` over `Address2`.
Prefer `approve(review)` over `updateStatus(entity, APPROVED)`.

## Bounded Contexts

Each bounded context owns its vocabulary. The same real-world concept may have different names and shapes in different contexts. Make these boundaries explicit.

`Customer` in billing may differ from `Customer` in shipping. Do not force a single model across contexts.

## Context Mapping

When concepts cross boundaries, use explicit translation layers: anti-corruption layers, published language, or shared kernel. Never leak internal vocabulary across context boundaries.

## Consistency

Use the same term for the same concept everywhere: code, tests, documentation, conversations. Update all references when vocabulary evolves.

## Anti-Patterns

- Technical names for domain concepts (e.g., `StringField` for `CustomerName`)
- Same name for different concepts across bounded contexts
- Domain terms that do not appear in code
- Generic CRUD vocabulary hiding domain semantics
- Inconsistent naming between code and documentation
- Forcing one model to serve multiple bounded contexts

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession (domain concepts as raw types)
- `architecture_boundary`: bounded context violations
