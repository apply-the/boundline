# API Contracts

APIs are contracts between producers and consumers. Stability, discoverability, and backward compatibility are non-negotiable for sustainable systems.

## Contract Stability

Published API surfaces must not break existing consumers. Use semantic versioning. Plan for evolution from the first version.

## Design Principles

- Resource-oriented for REST; service-oriented for RPC
- Consistent naming conventions across all endpoints
- Explicit versioning strategy (URL path, header, content negotiation)
- Pagination for list endpoints
- Idempotency keys for mutating operations

## Schema Definition

Use schema-first design where possible: OpenAPI for REST, Protobuf for gRPC, GraphQL SDL. Generate client and server code from schemas.

## API Specification Standards

### OpenAPI (REST / HTTP APIs)

OpenAPI (formerly Swagger) is the standard machine-readable contract for synchronous HTTP APIs.

- Write the OpenAPI spec first, then generate server stubs and client SDKs.
- Keep the spec in version control alongside the implementation; CI should validate that code and spec do not drift.
- Use `$ref` to share common schemas (error responses, pagination, standard headers) instead of duplicating them across endpoints.
- Declare `operationId` on every operation for stable client generation.
- Include `examples` or `example` on request/response schemas so generated docs are immediately usable.
- Pin the spec version (`3.0.x` or `3.1.x`) and do not mix versions within a service.

### AsyncAPI (Event-driven / Message APIs)

AsyncAPI is the equivalent contract standard for asynchronous, message-based APIs (Kafka, AMQP, MQTT, WebSocket, SNS/SQS).

- Declare channel, message, and payload schemas in an AsyncAPI document when the service produces or consumes events.
- Use `$ref` for shared payload schemas that appear in both OpenAPI and AsyncAPI surfaces of the same domain.
- Include `correlationId` bindings so consumers can trace message flows back to originating requests.
- Declare message `contentType` (usually `application/json` or Avro/Protobuf) and use schema validation at the consumer boundary.
- Version async contracts with the same discipline as synchronous ones: additive changes are safe; removal or type changes require a new channel or schema version.

### When Neither Applies

For gRPC services use `.proto` files as the authoritative schema. For GraphQL services use the SDL schema. The principles are the same: schema-first, version-controlled, CI-validated, generated artifacts.

## Error Responses

Use consistent error shapes across all endpoints. Include machine-readable error codes, human-readable messages, and correlation IDs. Map internal errors to appropriate HTTP/gRPC status codes.

## Evolution Strategy

Additive changes are safe: new optional fields, new endpoints, new enum values (with consumer tolerance). Removal or type changes require versioning or deprecation periods.

## Documentation

Keep API documentation generated from schemas. Include example requests and responses. Document rate limits, authentication requirements, and error scenarios.

## Anti-Patterns

- Breaking changes without version bump
- Inconsistent error response shapes across endpoints
- Missing pagination on unbounded collections
- Internal types exposed as API contracts
- Documentation that drifts from implementation
- Missing idempotency on non-idempotent operations
- Overly chatty APIs requiring many round trips

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: public-contract-stability, dependency-direction
- `clean_code`: no-primitive-obsession (raw strings as API identifiers)
