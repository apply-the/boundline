# JavaScript And Frontend Testing Guidance

Use Jest, Vitest, React Testing Library, Vue Test Utils, Cypress, and Playwright to verify behavior, not implementation trivia.

- Query UI by role, label, and visible text before falling back to brittle selectors.
- Keep unit tests small, parameterized, and behavior-focused; avoid `any` or contract-breaking shortcuts in tests.
- Mock network and browser boundaries, not component internals or framework primitives you are trying to verify.
- Keep snapshots small and intentional; large snapshots hide signal and age badly.
- Use end-to-end tests for a few critical user journeys, not as a substitute for unit and integration coverage.
- Reset timers, storage, and global state between tests so runs stay deterministic.
- Treat async expectations as first-class: await promises, flush updates explicitly, and avoid arbitrary sleeps.
