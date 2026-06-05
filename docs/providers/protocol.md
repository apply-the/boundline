# Provider Protocol

The external capability-provider protocol is a Boundline-owned contract for one
bounded capability surface at a time.

## V1 Operations

Every provider speaks the same small operation set:

- `capabilities`
- `health`
- `prepare`
- `execute`
- `collect_evidence`

Boundline chooses the transport, shapes the request envelope, and validates the
response before any result is treated as actionable runtime evidence.

## Transport Shapes

The first slice supports:

- command or stdio transport
- HTTP transport

The protocol stays intentionally small. V1 does not introduce long-lived
daemons, hidden background sessions, or provider-owned workflow state.

## Permission Envelope

Boundline admits provider-backed execution only when the declared permission
envelope is compatible with runtime policy.

Typical permission categories include:

- read files
- write files
- run commands
- network
- read secrets
- write artifacts

When provider metadata, specialized-profile metadata, and Boundline runtime
policy disagree, the stricter Boundline runtime policy wins. If the conflict
touches permissions, capability identity, lifecycle support, or evidence
requirements, execution fails closed before it starts.

## Evidence And Validation

Provider output is proposal-only until Boundline validates it.

Runtime projections distinguish:

- accepted evidence refs
- rejected evidence refs
- validation disposition
- failure class
- provider limitations

That distinction is visible through `status`, `inspect`, host JSON, and trace
projections.
