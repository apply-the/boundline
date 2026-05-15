# Clean Code Safety Guardian

Check changed files for fail-fast shortcuts that usually signal incomplete delivery or panic-prone control flow.

- Flag panic or placeholder shortcuts when they appear in production code instead of bounded error handling.
- Treat findings as design feedback: the change should make common failure paths explicit, typed, and reviewable.
- Keep evidence tied to the changed file so the remediation stays local and actionable.
