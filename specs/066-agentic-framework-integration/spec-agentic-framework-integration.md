# Boundline as Generic Agentic Framework: Integration Report

> Analysis of how Boundline can evolve from an orchestrator tightly coupled to Canon into an
> **agnostic orchestration engine (Agentic Framework Engine)**, capable of supporting
> proprietary frameworks (harnesses) through an adapter and override system.
>
> **Constraints**:
> - Boundline is **open source**.
> - Integration with specific or proprietary frameworks happens through **external binary adapters** (separate repositories).
> - A local reference template exists in the sibling `boundline-framework-template` repo.
> - **No MCP dependency as a core architectural layer**: capability abstraction uses our native Provider Protocol.

## Delivery Status

- Status: Delivered in Boundline `0.66.0`
- Primary implementation: `specs/066-agentic-framework-integration/`
- Outcome: Boundline now ships one explicit framework-adapter slot per
  workspace, the `speckit` known profile, custom-adapter registration,
  operator-visible routing and compatibility inspection, and the sibling
  template plus Speckit scaffolds aligned to the released V1 stdio contract.

---

## 1. Architectural Vision: Canon as Default, Adapters as Overrides

Boundline must not lose its out-of-the-box value.

**The golden rule:**
Boundline always ships with **Canon as the default**. If no adapter is configured, the lifecycle phases (`goal`, `plan`, `run`, `review`) are handled by the native Canon-backed logic.

**The partial-override abstraction:**
An external adapter (for example, a custom compiled Rust binary) does not need to replace the whole system. It can register to override a single step.
For example, a company adapter might declare: *"Use Canon for `goal` and `plan`, but intercept the `run` phase to apply my own destructive hooks and policies."*

---

## 2. Injection and Registration System

How does an external Rust binary get recognized by Boundline and receive its configuration?

### A. Discovery and Registration
Boundline should adopt a model inspired by Git or Terraform plugins:
1. **Config-based**: In `.boundline/config.toml`, the operator declares the adapter:
   ```toml
   [framework.adapter]
   command = "boundline-harness-gridspertise" # or an absolute path
   ```
2. **Naming Convention (Optional)**: Boundline can also automatically search `PATH` for binaries whose names start with `boundline-plugin-*`.

### B. Handshake (Capabilities and Config Injection)
When Boundline boots a session, it invokes the adapter binary with a handshake command (for example, via JSON-RPC over stdin by sending `{"method": "capabilities"}`).

The adapter responds with its manifest:
```json
{
  "name": "system-harness-template",
  "overrides": ["plan", "run"],          // Declares which stages it wants to intercept
  "hooks": ["on_error", "on_step_pre"],  // Declares which global hooks it listens to
  "config_schema": {                        // Requests configuration that Boundline must supply
    "harness_repo": "string",
    "strict_mode": "boolean"
  }
}
```

### C. Auto-configuration
Based on the returned `config_schema`, Boundline is responsible for:
- Checking whether `.boundline/config.toml` already contains those fields.
- If not, prompting the user for missing values during `boundline init` or at startup, or writing defaults.
- Passing the fully populated configuration block to the adapter on every subsequent invocation.

---

## 3. JSON Protocol over Stdin/Stdout

Communication should not happen through linked dynamic libraries (too fragile, ABI issues), but through a **Subprocess Protocol (JSON over stdin/stdout)**, following the same robust design approach used between LSPs (Language Servers) and IDEs.

**Boundline request to the adapter (override of the `plan` phase):**
```json
{
  "method": "execute_stage",
  "params": {
    "stage": "plan",
    "session_id": "abc-123",
    "workspace_ref": "/path/to/workspace",
    "adapter_config": {
      "harness_repo": "https://github.com/org/repo",
      "strict_mode": true
    },
    "context": { ... } // State gathered so far from Canon or Boundline
  }
}
```

**Adapter response:**
```json
{
  "result": {
    "status": "success",
    "artifacts_produced": ["/path/to/plan.md"],
    "phase_request": null // If the adapter needs user input, it returns that here
  }
}
```

---

## 4. Repository Architecture (The 3-Repo Model)

This design confirms the usefulness of the local template you created:

1. **`boundline` (Open Source)**:
   Contains the orchestrator, the JSON-RPC engine, and the **default** Canon implementation. No proprietary third-party framework logic lives here.
2. **`boundline-framework-template` (Open Source Template)**:
   The scaffolding repository you already created. It contains a ready-to-use JSON-RPC server, the correct Rust types, and empty methods (`fn execute_stage()`, `fn on_error()`). Anyone who wants to build a custom company-specific agentic framework can fork this repo.
3. **`my-company-harness-adapter` (Proprietary / Custom)**:
   The final binary compiled by the customer from the template. It would contain custom rules, `.github/hooks/` handling, or integrations with closed internal pipelines.

---

## 5. Mapping a Proprietary Harness to Boundline

A framework adapter can cover the logic of a company-specific `system-harness-template` by mapping its needs onto Boundline capabilities:

| External Framework Need | Boundline Adapter Solution |
|---|---|
| Custom lifecycle phases | The adapter declares `overrides: ["goal", "plan", "run"]` and injects its own logic. |
| Custom audit logs | The adapter registers for `on_step_post` and `on_session_end` hooks and writes its own logs. |
| Sensors / Quality / Linting | The adapter maps its own destructive scripts into `evaluate_gate` responses or `on_step_pre`. |
| Error handling (triage) | The adapter registers for `on_error`, reads telemetry, and decides whether to retry, block, or repair. |
| Platform integrations (Jira/CI) | No MCP. The adapter uses Boundline's native External Capability Provider Protocol or executes direct binaries/scripts. |

---

## 6. Next Steps (Action Items To Extend This Delivered Spec)

The baseline feature is now shipped. Follow-up work belongs in new feature
seeds or specs when one of these expansions becomes a bounded delivery slice:

1. broaden the adapter stage or hook catalog beyond the initial bounded set
2. replace duplicated sibling-repo protocol scaffolds with a released shared
  dependency line when the packaging policy is ready
3. add additional known profiles beyond `speckit`
4. introduce future transports or graceful-shutdown semantics beyond the
  current one-shot stdio contract

This design keeps Boundline fully reusable and optionally framework-agnostic, while preserving a safe and polished default UX through Canon when no adapter is configured.