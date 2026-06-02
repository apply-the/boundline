# Persistent Stdio

Boundline's **Persistent Stdio** architecture is the backbone of the V1 Framework Adapter Protocol. It allows Boundline to seamlessly integrate with external subprocess adapters (like `speckit`) without tying the runtime to complex remote APIs, brittle RPC frameworks, or heavy orchestrator SDKs.

## The V1 Wire Contract

Integration with external frameworks is achieved exclusively through trusted local subprocess execution using standard input (`stdin`) and standard output (`stdout`). 

The rules of the contract are strict:

- **One-Shot Execution**: Adapters are spawned for a specific bounded task, do their work, and exit. There is no persistent daemon or graceful shutdown logic required.
- **UTF-8 JSON Payload**: Boundline writes the entire execution context (including session state, config, and goals) to the adapter's `stdin` as a single, validated JSON payload.
- **Deterministic Envelopes**: The adapter MUST emit exactly one JSON payload to `stdout` before exiting. This payload must match a strict schema representing either a `Success` (with mutated state or plans) or an `Error` (with structured failure reasons).

## Handling Stderr

Standard error (`stderr`) is treated strictly as an out-of-band enrichment channel:

- Adapters can stream logs, debugging info, and raw LLM reasoning traces to `stderr`.
- Boundline captures this stream but **never** uses it to mutate the session state machine.
- The `stderr` stream is saved into the `.boundline/traces/` directory for human auditability and debugging purposes.

## Security & Independence

Because the adapter surface relies entirely on local `stdio`, Boundline achieves total execution isolation:

- **No Network Dependencies**: Adapters don't need to expose ports, handle TCP connections, or manage authentication.
- **Language Agnostic**: An adapter can be written in Rust, Python, Node.js, or Go. As long as it can parse JSON from `stdin` and print JSON to `stdout`, it is a fully compatible framework adapter.
- **Strict Host Ownership**: Boundline (the host) retains absolute control over capability validation, config persistence, and operator-visible output. If an adapter crashes, Boundline simply records the failure and recovers the session.