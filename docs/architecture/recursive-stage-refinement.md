# Recursive Stage Refinement

AI-assisted planning often degrades into hallucinations if the model is asked to generate a massive, complex implementation plan in a single shot. Boundline solves this using **Recursive Stage Refinement**.

Rather than trusting a single generative pass, the runtime orchestrates specialized reasoning profiles and independent review councils to iteratively critique and refine plans before they are cleared for execution.

## Reasoning Profiles

Boundline ships with built-in reasoning profiles that govern how an agent attempts a task:

- **`bounded_self_consistency`**: The planner generates multiple approaches to the same goal and algorithmically selects the one with the fewest projected side effects.
- **`independent_pair_review`**: A classic Maker-Checker pattern. One agent drafts the plan, and a strictly isolated secondary agent acts as the critic.
- **`heterogeneous_security_review`**: A specialized profile where the critic is heavily biased toward finding security vulnerabilities, path traversals, or destructive regressions in the proposed plan.
- **`bounded_reflexion`**: The planner is allowed to simulate the execution, read the projected errors, and refine the plan iteratively.

## Review Councils & Voting

For high-risk operations, Boundline utilizes **Review Councils**. A council is an assembly of independent evaluator profiles that review a proposed plan stage.

The council uses bounded adjudication logic:
- The plan must achieve a consensus vote to proceed.
- If rejected, the council attaches irrefutable *findings* to the session state.
- The planner is invoked again, strictly forced to address the attached findings.

## Preventing Infinite Loops

To ensure recursive refinement doesn't devolve into an infinite loop of arguing agents, Boundline enforces rigid **stop semantics**:

1. **Max Iterations**: A hard cap on refinement cycles (e.g., 3 attempts).
2. **Degradation**: If the council cannot reach consensus within the limits, the session transitions to a `blocked` state.
3. **Operator Handoff**: The runtime stops execution and prompts the human operator via `boundline status` to manually resolve the impasse.