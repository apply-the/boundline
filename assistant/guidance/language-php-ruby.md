# PHP And Ruby Language Guidance

Use PHP and Ruby with explicit types, visible boundaries, and domain objects that do not collapse into loose hashes.

- Prefer strict types, typed DTOs, and value objects over associative arrays or hashes as implicit domain models.
- Inject dependencies explicitly instead of reaching through globals, service locators, or framework singletons.
- Keep domain rules out of callbacks, metaprogramming hooks, and magic method chains when clarity matters.
- Use specific exceptions and explicit result semantics instead of broad rescue or silent failure paths.
- Keep mutable shared state and hidden defaults out of business logic.
- Favor clear command or service objects when a controller or model starts owning orchestration.
