# Security Boundary Guardian

Enforce validation, authentication, and safe data handling at trust boundaries where data crosses between security domains.

## Rules

### sql-injection-risk
Database queries must use parameterized statements. String concatenation or interpolation of external input into query strings creates injection vulnerabilities.

Triggers: string formatting into SQL queries, user input concatenated with query fragments, ORM raw query methods with interpolated values.

### command-injection-risk
External input must never be interpolated into shell commands. Use structured argument passing, parameterized APIs, or allowlist-based command construction.

Triggers: user input in shell command strings, `eval` with external data, `exec` or `system` calls with interpolated arguments.

### secret-in-source
Secrets (API keys, passwords, tokens, private keys) must not appear in source code, committed configuration files, or logs. Use secrets managers and environment-based injection.

Triggers: hardcoded credential strings, API keys in configuration files under version control, tokens in log output, secrets in error messages.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages. Cross-cutting; relevant at every trust boundary.
