# Shell And Automation Guidance

Use shell and automation code as production delivery logic, not as an unstructured scratchpad.

- Fail fast with explicit shell options and clear exit codes.
- Quote variables, validate inputs, and treat filesystem paths and environment variables as untrusted boundaries.
- Keep functions small and named by responsibility instead of encoding a whole workflow in one pipeline.
- Separate validation, side effects, cleanup, and logging so failures are diagnosable.
- Avoid interactive assumptions, hidden prompts, and environment-dependent behavior in automation paths.
- Prefer idempotent operations and explicit cleanup for rerunnable scripts.
