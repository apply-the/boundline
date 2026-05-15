# Mobile Testing Guidance

Use widget, component, and integration tests to verify mobile behavior without depending on device-only flakiness.

- Test user-visible behavior, navigation, and state transitions before internal implementation details.
- Fake network, storage, and platform services when verifying UI behavior; use integration tests for real platform boundaries.
- Keep accessibility, offline, retry, and permission-denied flows covered where they affect user trust.
- Reset app state, timers, and local storage between tests.
- Keep emulator or device lab suites small and focused on critical flows.
