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
        B19["Boundline 19<br/>(Plan Orchestrator)"]:::boundline
        C03["Canon 03<br/>(Handoff Schemas)"]:::canon
    end

    subgraph Providers & Extensibility
        B07["Boundline 07<br/>(Provider Protocol)"]:::boundline
        B13["Boundline 13<br/>(Sandbox Exec)"]:::boundline
        C07["Canon 07<br/>(Integration Onboarding)"]:::canon
        B14["Boundline 14<br/>(AI Gateway)"]:::boundline
        B15["Boundline 15<br/>(Browser Testing)"]:::boundline
        B17["Boundline 17<br/>(Recursivemas Adapter)"]:::boundline
    end

    subgraph Observability & Memory
        B16["Boundline 16<br/>(Session Memory)"]:::boundline
    end

    %% Key Dependencies
    C02 ---|Hard Pair| B18
    B18 -->|Hard Dependency| B19
    B19 -->|Triggers Export| C03
    B13 -->|Enables| C07    
```

## Execution Order and Dependencies

1. **Canon 02 + Boundline 18 (Verification Pair)**
   - The first crucial execution juncture. Canon defines the `claim -> proof -> evidence_ref` contract, while Boundline implements the runtime that executes the proof and blocks task completion.
2. **Boundline 19 (Execution Orchestrator)**
   - Depends directly on `Boundline 18` to ensure that task ordering, checkpointing, and resume logic rely on a solid verification gate.
3. **Canon 03 (Parallel to 19)**
   - Defines purely the handoff/progress schema. It can be developed in parallel to the Boundline execution engine, or right before its integration to allow Boundline to export compatible packets.
4. **Boundline 07 -> Boundline 13 (Provider Layer)**
   - The actual external provider setup (MCP, setup, activation, health). `Boundline 07` comes first, followed by the security layer `Boundline 13` (secret inheritance and sandbox). It establishes the plugin layer that powers B14, B15, and B17.
5. **Canon 07 (After provider setup)**
   - Arrives at the end to close the loop on the CLI side (Canon init) by gathering local routing choices, delegating execution back to Boundline.
6. **Independent Features (Boundline 16)**
   - These features cover autonomous workflows, policy, observability, and advanced orchestrator additions. They do not block the core engine loop and can be parallelized based on priority. 
