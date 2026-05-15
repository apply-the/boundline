# Security And Boundary Guidance

Treat security as a design property of boundaries, not a final-stage checklist.

- Consider all external input untrusted: HTTP bodies, headers, cookies, webhooks, files, messages, env vars, and legacy data.
- Separate authentication from authorization and enforce both at the use-case boundary.
- Use parameterized queries, safe serializers, and framework escaping instead of manual string assembly.
- Avoid raw HTML or markdown rendering without sanitization and clear ownership of trusted content.
- Never hardcode, expose, or log secrets; use managed secret sources and environment separation.
- Keep file uploads, path handling, and storage isolated from path traversal or unsafe execution.
- Return safe client-facing errors while keeping correlation IDs and internal details in logs or traces.
- Treat dependency hygiene and supply-chain review as part of release readiness.
