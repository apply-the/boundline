# PHP And Ruby Service Framework Guidance

Apply Laravel and Rails as delivery frameworks around explicit use cases and narrow persistence boundaries.

- Keep controllers thin: validate input, authorize, delegate to a use case, map the response.
- Do not hide business rules in model callbacks, concerns, helpers, or framework macros.
- Keep transactions explicit and narrow; do not mix database state changes with slow external I/O in the same boundary.
- Map request data into typed commands or service inputs before domain logic sees it.
- Use policies and authorization checks explicitly where the use case executes, not only in the UI.
- Move long-running notifications, webhooks, and fan-out side effects to background jobs.
