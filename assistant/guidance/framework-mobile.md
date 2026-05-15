# Mobile Framework Guidance

Apply Flutter and React Native as presentation shells around explicit state, service, and platform boundaries.

- Separate presentational widgets or components from orchestration logic, providers, and service clients.
- Keep loading, offline, retry, error, and success states explicit in screen flows.
- Keep business rules out of widget trees, hooks, and navigation callbacks.
- Make storage, permissions, platform channels, and push notification boundaries explicit and testable.
- Keep global state minimal and scoped to navigation, session, or app-level coordination.
- Prefer dependency injection or provider-style composition over hidden singleton services.
