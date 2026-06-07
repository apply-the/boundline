# Recursive Stage Refinement

AI-assisted planning often degrades into hallucinations if the model is asked to
generate a massive, complex implementation plan in a single shot. Boundline
solves this using **Recursive Stage Refinement**: bounded, inspectable
refinement loops over a specific planning stage.

Rather than trusting a single generative pass, the runtime orchestrates a
`planner → critic → planner → finalizer` pattern with structured round packets,
a runtime-validated confidence model, and a closed set of stop reasons.

## Refinement Profiles

Boundline ships with one refinement profile and an open extension point:

- **`plan_refinement`** (built-in): The canonical refinement profile for the
  plan stage. Activates the four-role refinement loop described below.
- Additional profiles for other stages can be declared through the same profile
  contract.

Refinement is opt-in: use `--refine` to enable it, `--no-refine` to disable it,
and `--max-rounds N` to set an explicit round cap. The default profile is
`plan_refinement`.

## The Refinement Loop

Each refinement round follows a strict four-role cycle:

1. **Planner**: Generates a draft plan for the current goal.
2. **Critic**: Reviews the draft and assigns a confidence score (`None`,
   `Low`, `Sufficient`, `High`) plus a set of structured findings.
3. **Planner** (second pass): Revises the plan, addressing the critic's
   findings.
4. **Finalizer**: Produces the final plan, marking the round complete.

Each round produces a compact **Round Packet** persisted in the trace store:

- Round number and trace-linked `candidate_ref`
- Critic-proposed confidence and runtime-resolved effective confidence
- Structured findings with deduplication
- Material-delta detection (did the plan actually change?)
- Exactly one stop reason from the closed vocabulary

## Confidence Model

Confidence flows through a four-level scale validated by the runtime:

| Level | Meaning |
|---|---|
| `None` | No confidence assessment provided |
| `Low` | Critic identified material gaps |
| `Sufficient` | Plan is adequate for execution |
| `High` | Plan exceeds quality thresholds |

The runtime enforces two key rules:

- **Confidence upgrade**: Effective confidence may only increase across rounds;
  the critic cannot downgrade a previously higher rating.
- **High + findings**: A plan rated `High` confidence must have zero findings;
  this is a hard validation invariant.

## Stop Reasons

The refinement loop terminates with exactly one of 9 stop reasons:

| # | Stop Reason | Description |
|---|---|---|
| 1 | `MaxRoundsReached` | Hard round limit exhausted |
| 2 | `TimeBudgetExhausted` | Wall-clock budget consumed |
| 3 | `SufficientConfidenceNoFindings` | Plan is adequate with no open findings |
| 4 | `HighConfidence` | Plan exceeds quality thresholds |
| 5 | `NoMaterialDelta` | Critic and planner produced no meaningful change |
| 6 | `CriticApprovedAsIs` | Critic accepted the plan without revision |
| 7 | `FinalizerCompleted` | Finalizer produced a terminal plan |
| 8 | `OperatorInterrupt` | Operator requested stop mid-refinement |
| 9 | `DegradedToBlocked` | Loop cannot make progress within constraints |

## CLI Integration

Refinement state is visible through the standard operator surfaces:

- **`boundline status`**: Shows the refinement summary (rounds completed, final
  stop reason, effective confidence) in the session status view.
- **`boundline next`**: Suggests next actions based on refinement state
  (continue refinement, proceed to run, or operator decision needed).
- **`boundline inspect`**: Drills into individual round packets with
  round-level detail including stop reason, confidence, findings, and
  artifact references.

## Bounded Execution Guarantees

To ensure refinement doesn't devolve into an infinite loop:

1. **Hard round cap**: `max_rounds` (default 3) enforced by the runtime.
2. **Time budget**: Configurable wall-clock budget with exhaustion detection.
3. **No-progress detection**: Material-delta check prevents empty rounds from
   consuming budget.
4. **Degradation path**: If the loop cannot reach a terminal stop reason, it
   degrades to `DegradedToBlocked` and surfaces the impasse through status and
   next commands for operator resolution.