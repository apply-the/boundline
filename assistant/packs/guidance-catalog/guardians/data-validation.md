# Data Validation Guardian

Enforce schema validation at system boundaries and prevent sensitive data from leaking into logs or error messages.

## Rules

### unvalidated-external-input
Data from external sources must be validated against a schema before use in business logic. Trusting external shape without validation enables injection, type confusion, and corrupt state.

Triggers: HTTP request bodies used without schema validation, environment variables used without format checking, file contents parsed without structure verification, AI/LLM output consumed without format validation.

### missing-schema-at-boundary
Every system boundary (API endpoint, queue consumer, file importer, webhook handler) should have an explicit schema definition that documents and enforces the expected data contract.

Triggers: API handlers without request schema (OpenAPI, Zod, Pydantic, Bean Validation), queue consumers processing messages without schema validation, configuration loaded without type checking.

### log-contains-pii
Personally identifiable information (names, emails, phone numbers, addresses, tokens, credentials) must not appear in log output, error messages, or observability data without explicit masking.

Triggers: logging full request bodies that contain user data, error messages that include email addresses or names, traces that capture authentication tokens, debug logging of entire database records.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages. Cross-cutting; relevant at every point where external data enters the system or where data is emitted to logs, metrics, or error reporting.
