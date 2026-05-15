# Security Boundaries

Security must be enforced at system boundaries where trust transitions occur. Internal code trusts validated types; external input is hostile until proven otherwise.

## Trust Boundaries

Identify where data crosses trust levels:
- Public internet to application (HTTP, WebSocket, gRPC)
- Application to database (queries, stored procedures)
- Service to service (internal APIs, queues, events)
- User input to file system or command execution
- Configuration and environment variables at startup

## Input Validation

Validate shape, type, length, and range at every trust boundary. Reject invalid input early. Use allowlists over denylists.

## Authentication And Authorization

Authenticate at the edge. Authorize at the resource. Keep authorization logic centralized and testable. Do not scatter permission checks through business logic.

## Secrets Management

Never store secrets in source code, environment variables checked into version control, or logs. Use a secrets manager. Rotate credentials regularly. Encrypt at rest.

## Injection Prevention

Use parameterized queries for databases. Escape output for the target context (HTML, shell, SQL, LDAP). Never construct commands from raw user input.

## Transport Security

Use TLS for all network communication. Validate certificates. Use HSTS headers. Do not downgrade to HTTP.

## Anti-Patterns

- Validation only on the client side
- Secrets in source code or environment files committed to git
- SQL string concatenation
- Shell command construction from user input
- Missing authorization on internal endpoints
- Overly permissive CORS configuration
- Security logic duplicated across services inconsistently
- Logging sensitive data (passwords, tokens, PII)

## Guardian Hooks

Guardians that apply to this guidance:
- `security_boundary`: sql-injection-risk, command-injection-risk, secret-in-source
- `clean_code`: no-hidden-side-effects (when security checks have side effects)
- `architecture_boundary`: security logic placement
