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
        B10["Boundline 10<br/>(Review Councils)"]:::boundline
        B11["Boundline 11<br/>(Adaptive Governance)"]:::boundline
        B12["Boundline 12<br/>(Recursive Refinement)"]:::boundline
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
        C06["Canon 06<br/>(Observability Design)"]:::canon
        B08["Boundline 08<br/>(Evals & Observability)"]:::boundline
        B16["Boundline 16<br/>(Session Memory)"]:::boundline
    end

    subgraph Advanced Workflows & Policy
        B09["Boundline 09<br/>(Contextual Help)"]:::boundline
    end

    %% Key Dependencies
    C02 ---|Hard Pair| B18
    B18 -->|Hard Dependency| B19
    B19 -->|Triggers Export| C03
    B19 --> B07
    B07 -->|Activates| B13
    B13 -->|Enables| C07
    
    B07 -.-> B14
    B07 -.-> B15
    B07 -.-> B17
    
    C06 -.->|Design for| B08
    C05 -.->|Pairs well| C06
```

## Execution Order and Dependencies

1. **Core Foundations**
   - Boundline `04`, `05`, `06`, and `07` are now shipped. The next hard rollout dependency starts at verification.
2. **Canon 02 + Boundline 18 (Verification Pair)**
   - The first crucial execution juncture. Canon defines the `claim -> proof -> evidence_ref` contract, while Boundline implements the runtime that executes the proof and blocks task completion.
3. **Boundline 19 (Execution Orchestrator)**
   - Depends directly on `Boundline 18` to ensure that task ordering, checkpointing, and resume logic rely on a solid verification gate.
4. **Canon 03 (Parallel to 19)**
   - Defines purely the handoff/progress schema. It can be developed in parallel to the Boundline execution engine, or right before its integration to allow Boundline to export compatible packets.
5. **Boundline 07 -> Boundline 13 (Provider Layer)**
   - `Boundline 07` is now shipped and owns the external provider setup contract: explicit registration, setup, activation, health, permission admission, and evidence handling. `Boundline 13` follows as the security layer (secret inheritance and sandbox) and builds on that provider boundary to power B14, B15, and B17.
6. **Canon 07 (After provider setup)**
   - Arrives at the end to close the loop on the CLI side (Canon init) by gathering local routing choices, delegating execution back to Boundline.
7. **Independent Features (Canon 04, 05, 06 & Boundline 08-12, 16)**
   - These features cover autonomous workflows, policy, observability, and advanced orchestrator additions. They do not block the core engine loop and can be parallelized based on priority. 
