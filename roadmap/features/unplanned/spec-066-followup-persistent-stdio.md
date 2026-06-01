# Follow-Up Specification: Persistent Stdio Transport For Boundline Framework Adapters

**Related Feature**: `066-agentic-framework-integration`  
**Proposed Follow-Up**: `066-followup-persistent-stdio`  
**Status**: Proposed  
**Scope Type**: Follow-up transport enhancement  
**Primary Owner**: Boundline core  
**Applies To**: Boundline framework adapters, `boundline-framework-template`, `boundline-adapter-speckit`, and future external adapters

---

## 1. Summary

The initial Agentic Framework Integration feature introduces external framework adapters through a command-oriented JSON stdin/stdout protocol. That V1 model is intentionally simple: Boundline invokes adapter commands such as `describe`, `preflight`, `execute-stage`, and `emit-hook`, receives one JSON response, and the adapter process exits.

This follow-up introduces an optional persistent stdio transport. Boundline starts an adapter process once, keeps it alive for the duration of a lifecycle run or bounded adapter session, and exchanges framed JSON-RPC messages over stdin/stdout.

The goal is not to change adapter semantics. The goal is to make the transport more efficient and capable when adapters benefit from warm state, cached analysis, loaded project data, or frequent lifecycle calls.

---

## 2. Motivation

Process-per-command JSON stdio is a good V1 transport because it is easy to debug, robust, cross-platform, and avoids server lifecycle complexity.

However, it becomes less efficient when an adapter needs to:

- keep an AST or repository model in memory
- keep a local database connection open
- reuse parsed Speckit artifacts across multiple stage calls
- receive many hook events during a run
- report progress for long-running analysis
- support cancellation
- avoid repeated process startup overhead
- share adapter-local caches across `preflight`, `execute-stage`, and `emit-hook`

Persistent stdio provides a natural upgrade path without moving to TCP, HTTP, MCP, or dynamic library plugins.

---

## 3. Design Principle

Separate the logical adapter protocol from the transport.

The logical protocol remains:

- `describe`
- `preflight`
- `execute-stage`
- `emit-hook`

The transport may be:

- `process-json-stdio` for V1 command-per-process calls
- `persistent-jsonrpc-stdio` for a long-running adapter server
- future local transports such as Unix sockets, named pipes, or localhost HTTP if needed

Boundline must not make MCP, HTTP, or dynamic linking the core adapter abstraction.

---

## 4. Non-Goals

This follow-up does not introduce:

- multi-adapter composition
- adapter permission mediation
- sandboxed adapter execution
- remote adapter services
- MCP as the core transport
- dynamic library loading
- adapter marketplace behavior
- background autonomous adapter workers
- a new lifecycle orchestration model
- Canon as an external adapter

Canon-aware built-in behavior remains the default Boundline behavior. Speckit and custom frameworks remain external adapters.

---

## 5. Transport Model

### 5.1 V1 Current Model: Process JSON Stdio

Current adapter calls look like this:

```text
Boundline -> spawn adapter describe
Adapter   -> write JSON response to stdout
Adapter   -> exit

Boundline -> spawn adapter preflight
Adapter   -> read JSON request from stdin
Adapter   -> write JSON response to stdout
Adapter   -> exit
```

This is still valid and must remain supported.

### 5.2 Follow-Up Model: Persistent JSON-RPC Stdio

Persistent mode starts the adapter once:

```text
Boundline -> spawn adapter serve
Adapter   -> stays alive

Boundline -> stdin: JSON-RPC request describe
Adapter   -> stdout: JSON-RPC response describe

Boundline -> stdin: JSON-RPC request preflight
Adapter   -> stdout: JSON-RPC response preflight

Boundline -> stdin: JSON-RPC request execute-stage
Adapter   -> stdout: JSON-RPC response execute-stage

Boundline -> stdin: JSON-RPC notification shutdown
Adapter   -> exits cleanly
```

The adapter still communicates over stdin/stdout. The difference is that the process remains alive across multiple requests.

---

## 6. Adapter Capability Declaration

Adapters should declare supported transports in `describe`.

Example V1 adapter response:

```json
{
  "protocol_line": "framework-adapter-v1",
  "adapter_id": "speckit",
  "adapter_version": "0.1.0",
  "supported_boundline_range": ">=0.66.0,<0.67.0",
  "supported_transports": ["process-json-stdio"],
  "preferred_transport": "process-json-stdio",
  "declared_stage_overrides": ["plan"],
  "declared_hook_subscriptions": ["stage_completed", "stage_failed"],
  "required_config_fields": []
}
```

Example persistent-capable adapter response:

```json
{
  "protocol_line": "framework-adapter-v1",
  "adapter_id": "speckit",
  "adapter_version": "0.2.0",
  "supported_boundline_range": ">=0.66.0,<0.67.0",
  "supported_transports": [
    "process-json-stdio",
    "persistent-jsonrpc-stdio"
  ],
  "preferred_transport": "persistent-jsonrpc-stdio",
  "declared_stage_overrides": ["plan", "review"],
  "declared_hook_subscriptions": ["stage_started", "stage_completed", "stage_failed"],
  "required_config_fields": []
}
```

Boundline must select the preferred transport only when it supports that transport and the adapter declares compatibility.

---

## 7. Persistent Message Envelope

Persistent stdio should use a JSON-RPC-compatible envelope.

### 7.1 Request

```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "preflight",
  "params": {
    "boundline_version": "0.66.0",
    "workspace_ref": "/workspace",
    "non_interactive": false,
    "config_values": []
  }
}
```

### 7.2 Response

```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "status": "ready",
    "normalized_config_values": [],
    "warnings": []
  }
}
```

### 7.3 Error Response

```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "error": {
    "code": "missing_required_config",
    "message": "The adapter requires `spec_root` before execution.",
    "recoverable": true,
    "recommended_action": "Run `boundline adapter configure speckit` or set the required field in workspace configuration."
  }
}
```

### 7.4 Notification

Notifications do not require a response.

```json
{
  "jsonrpc": "2.0",
  "method": "shutdown",
  "params": {
    "reason": "run_completed"
  }
}
```

---

## 8. Framing

Persistent stdio needs explicit message framing because stdin/stdout is a byte stream.

Recommended framing: LSP-style headers.

```text
Content-Length: 123

{"jsonrpc":"2.0","id":"req-001","method":"describe","params":{}}
```

Rules:

- Each message must include `Content-Length`.
- Message payload must be UTF-8 JSON.
- Adapter stdout must contain only framed protocol messages.
- Adapter stderr may contain logs, diagnostics, and human-readable debug output.
- Boundline must reject unframed or malformed protocol messages in persistent mode.

---

## 9. Process Lifecycle

Boundline owns the persistent adapter lifecycle.

### 9.1 Start

Boundline starts the adapter with:

```bash
boundline-adapter-speckit serve
```

or equivalent configured command and arguments.

### 9.2 Initialize

After process start, Boundline sends an `initialize` request.

```json
{
  "jsonrpc": "2.0",
  "id": "init-001",
  "method": "initialize",
  "params": {
    "boundline_version": "0.66.0",
    "protocol_line": "framework-adapter-v1",
    "workspace_ref": "/workspace",
    "session_ref": "session-123",
    "transport": "persistent-jsonrpc-stdio"
  }
}
```

Adapter response:

```json
{
  "jsonrpc": "2.0",
  "id": "init-001",
  "result": {
    "status": "ready",
    "adapter_id": "speckit",
    "adapter_version": "0.2.0"
  }
}
```

### 9.3 Execute Requests

Boundline sends `describe`, `preflight`, `execute-stage`, and `emit-hook` as JSON-RPC requests or notifications.

### 9.4 Shutdown

Boundline sends `shutdown`, waits for the adapter to exit, and kills the process if it does not exit within the configured timeout.

### 9.5 Crash Recovery

If the adapter process exits unexpectedly:

- Boundline marks the active adapter call as failed.
- If the adapter had not yet taken ownership of a stage, Boundline may fall back to built-in behavior where existing V1 semantics allow it.
- If the adapter had already taken ownership of an overridden stage, Boundline marks the stage failed, stops the run, and requires operator intervention.
- Boundline records the crash and transport failure in the stage execution record.

---

## 10. Request Ordering And Concurrency

V1 persistent stdio should be sequential-first.

Rules:

- Boundline may send only one in-flight request at a time in the first persistent version.
- The adapter must respond before Boundline sends the next request.
- Notifications such as `shutdown` may interrupt only where explicitly allowed.
- Parallel in-flight requests are out of scope for the first persistent stdio slice.

This preserves the existing Boundline sequential lifecycle model and reduces adapter complexity.

---

## 11. Timeout And Cancellation

Persistent stdio requires explicit timeout and cancellation semantics.

### 11.1 Request Timeout

Each request must have an effective timeout.

If timeout expires:

- Boundline sends a `cancel` notification when possible.
- Boundline waits for a short cancellation grace period.
- If the adapter does not respond, Boundline kills and restarts or fails the adapter session depending on stage ownership.

### 11.2 Cancellation Notification

```json
{
  "jsonrpc": "2.0",
  "method": "cancel",
  "params": {
    "request_id": "req-001",
    "reason": "timeout"
  }
}
```

### 11.3 Stage Ownership Rule

Cancellation cannot silently revert a stage that the adapter already owns.

If a timeout occurs after adapter stage ownership begins, Boundline must mark the stage failed and require operator intervention.

---

## 12. Logging Rules

Strict output separation is required.

- stdout is reserved for protocol messages only.
- stderr is reserved for logs and human diagnostics.
- Logs must never be interleaved with JSON protocol messages on stdout.
- Boundline may capture stderr and attach it to diagnostics, traces, or adapter failure records.
- Sensitive data must be redacted from captured logs where possible.

This rule must be explicit in `boundline-framework-template` and every generated adapter scaffold.

---

## 13. State And Warm Cache Rules

Persistent adapters may keep internal warm state, but that state must not become the authoritative Boundline state.

Allowed internal state examples:

- parsed Speckit files
- adapter-local AST or document maps
- local database connection
- cached config validation
- precomputed plan fragments
- internal capability cache

Not allowed:

- hidden lifecycle state that Boundline cannot reconstruct or explain
- hidden approval state
- hidden run ownership state
- hidden source-of-truth packet state
- hidden decisions that are not returned as artifacts, findings, evidence, or execution records

Boundline remains the owner of:

- session state
- stage state
- stop semantics
- audit records
- adapter selection
- fallback decisions
- run continuation behavior

---

## 14. Audit And Observability Requirements

Persistent transport must not reduce auditability.

Boundline must record:

- selected adapter transport
- adapter process start
- adapter initialize result
- each request method
- each request id
- request duration
- timeout or cancellation events
- adapter stderr summary when failures occur
- process exit code
- stage ownership boundary
- fallback or failure decision

Stage execution records must identify:

- `execution_source`: `built_in` or `adapter`
- `adapter_id`
- `transport`: `process-json-stdio` or `persistent-jsonrpc-stdio`
- `request_id`
- `started_at`
- `finished_at`
- `status`
- `failure_reason` when applicable

---

## 15. Configuration

Workspace configuration may allow transport selection.

Example:

```toml
[framework]
active_adapter = "speckit"

[framework.adapters.speckit]
kind = "external-process"
command = "boundline-adapter-speckit"
enabled = true
transport = "auto"

[framework.adapters.speckit.config]
spec_root = ".specify"
```

Allowed values:

- `auto`
- `process-json-stdio`
- `persistent-jsonrpc-stdio`

Rules:

- `auto` uses the adapter preferred transport when Boundline supports it.
- Explicit transport values override adapter preference.
- If an explicit transport is unsupported by the adapter, Boundline fails before execution with actionable feedback.
- If `persistent-jsonrpc-stdio` initialization fails before stage ownership, Boundline may fall back to `process-json-stdio` only when configured to allow transport fallback.
- Transport fallback must be visible in audit records.

---

## 16. Template Repository Updates

`boundline-framework-template` should add persistent stdio support as an optional path.

Required updates:

- keep existing command-per-process commands
- add `serve`
- add JSON-RPC framing parser
- add JSON-RPC response writer
- add typed request and response models
- add initialize/shutdown handling
- add request timeout simulation tests
- add stdout-only-protocol validation tests
- document stderr logging rules
- add fixtures for persistent messages

The template must continue to build as a standalone Rust crate.

---

## 17. Speckit Adapter Updates

`boundline-adapter-speckit` may adopt persistent stdio after the template supports it.

Suggested use cases:

- cache parsed `.specify` files
- cache contract discovery
- avoid repeated preflight scans
- reuse stage-level context during a run
- receive repeated hook events without process respawn overhead

The adapter must still support `process-json-stdio` unless Boundline explicitly drops that compatibility in a future major adapter protocol line.

---

## 18. Acceptance Criteria

- Boundline can discover that an adapter supports `persistent-jsonrpc-stdio`.
- Boundline can start an adapter in persistent mode.
- Boundline can initialize the adapter with protocol, workspace, and session context.
- Boundline can send `describe`, `preflight`, `execute-stage`, and `emit-hook` requests over one persistent stdio session.
- Boundline enforces stdout-only protocol messages and captures stderr as logs.
- Boundline applies timeouts and cancellation semantics.
- Boundline records transport choice and request-level audit events.
- Boundline preserves existing failure semantics for pre-stage and mid-stage adapter failures.
- The template repository supports both command-per-process and persistent stdio modes.
- The Speckit adapter can opt into persistent mode without forcing all adapters to migrate.

---

## 19. Rollout Strategy

### Phase 1: Protocol Readiness

- Add `supported_transports` and `preferred_transport` to adapter describe output.
- Keep `process-json-stdio` as default.
- Add tests proving transport negotiation works.

### Phase 2: Persistent Transport Host

- Implement Boundline persistent adapter host.
- Add process lifecycle, framing, initialize, shutdown, timeout, cancellation, and audit.

### Phase 3: Template Support

- Add `serve` mode to `boundline-framework-template`.
- Add persistent JSON-RPC test fixtures.
- Document transport rules.

### Phase 4: Speckit Adapter Adoption

- Add persistent support to `boundline-adapter-speckit`.
- Use cached parsing where beneficial.
- Keep process-per-command fallback.

### Phase 5: Evaluation

- Measure adapter call latency.
- Measure run time on Speckit-heavy flows.
- Compare failure clarity against process-per-command mode.
- Promote persistent transport only if it improves real workflows without degrading reliability.

---

## 20. Open Questions

1. Should persistent stdio support be implemented in Boundline before or after the first production Speckit adapter release?
2. Should transport fallback from persistent to process-per-command be allowed by default, or only when configured?
3. Should the first persistent implementation support progress notifications, or defer them to a later follow-up?
4. Should adapter stderr be persisted in full, summarized, or captured only on failure?
5. Should adapters be allowed to keep warm state across an entire lifecycle run only, or across multiple runs in the same workspace?
