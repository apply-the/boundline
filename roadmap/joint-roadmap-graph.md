# Canon & Boundline Joint Feature Rollout

This document illustrates the operational sequence for the joint development of Canon and Boundline features. It encompasses all features from both roadmaps, grouping them by domain and showing critical execution dependencies.

## Dependency Graph

```mermaid
flowchart TD
    %% Styling
    classDef canon fill:#5b5b95,stroke:#333,stroke-width:2px,color:#fff
    classDef boundline fill:#1f6b4e,stroke:#333,stroke-width:2px,color:#fff

    subgraph Verification Integrity
        C02["Canon 02<br/>(Verification Gates)"]:::canon
        B18["Boundline 18<br/>(Verification Runtime)"]:::boundline
    end

    subgraph Execution & Orchestration
        B13["Boundline 13<br/>(Safe Command Exec)"]:::boundline
        B19["Boundline 19<br/>(Plan Orchestrator)"]:::boundline
        C03["Canon 03<br/>(Handoff Schemas)"]:::canon
    end

    subgraph Providers & Extensibility
        B07["Boundline 07<br/>(Provider Protocol)"]:::boundline
        C07["Canon 07<br/>(Integration Onboarding)"]:::canon
        B14["Boundline 14<br/>(AI Gateway)"]:::boundline
        B15["Boundline 15<br/>(Browser Testing)"]:::boundline
        B17["Boundline 17<br/>(Recursivemas Adapter)"]:::boundline
    end

    subgraph Deferred
        B13B["Boundline 13B<br/>(Sandbox Runtime, Deferred)"]:::boundline
    end

    subgraph Observability & Memory
        B16["Boundline 16<br/>(Session Memory)"]:::boundline
    end

    %% Key Dependencies
    C02 ---|Hard Pair| B18
    B13 -->|Provides execution evidence| B18
    B18 -->|Hard Dependency| B19
    B13 -->|Foundation for safe execution| B19
    B19 -->|Triggers Export| C03

    B07 -->|Provider permission vocabulary| B13B
    B13 -->|Execution safety foundation| B13B
    B13B -->|Future sandboxed provider setup| C07    
```

## Execution Order and Dependencies

1. **Canon 02 + Boundline 18 (Verification Pair)**
   - The first crucial execution juncture. Canon defines the `claim -> proof -> evidence_ref` contract, while Boundline implements the runtime that executes the proof and blocks task completion.
2. **Boundline 13 (Execution Safety Foundation)**
   - Boundline 13 establishes safe local command execution, evidence capture,
     artifact capture, redaction, and mutation boundaries. It supports
     verification and orchestration without requiring Docker sandboxing.
   - B13 is pulled earlier because safe command execution, evidence capture,
     artifact capture, and redaction are needed before orchestration becomes
     trustworthy.
3. **Boundline 19 (Execution Orchestrator)**
   - Depends directly on `Boundline 18` to ensure that task ordering, checkpointing, and resume logic rely on a solid verification gate.
   - Benefits from `B13` execution evidence and safety foundation.
4. **Canon 03 (Parallel to 19)**
   - Defines purely the handoff/progress schema. It can be developed in parallel to the Boundline execution engine, or right before its integration to allow Boundline to export compatible packets.
5. **Boundline 07 (Provider Protocol)**
   - The external provider setup (MCP, setup, activation, health). Establishes the plugin layer that powers B14, B15, and B17.
6. **Deferred: Boundline 13B (Sandbox Runtime)**
   - Boundline 13B adds local sandbox execution for high-risk provider-backed
     or mutation-heavy commands. It depends on the provider permission
     vocabulary from B07 and the execution evidence foundation from B13A.
   - B13B is deferred because Docker sandboxing, mount policy, network
     policy, and secret handle inheritance depend on provider permissions
     and execution policy foundations. It should not block the core
     verification and orchestration loop.
7. **Canon 07 (After provider setup)**
   - Arrives at the end to close the loop on the CLI side (Canon init) by gathering local routing choices, delegating execution back to Boundline.
8. **Independent Features (Boundline 16)**
   - These features cover autonomous workflows, policy, observability, and advanced orchestrator additions. They do not block the core engine loop and can be parallelized based on priority. 
