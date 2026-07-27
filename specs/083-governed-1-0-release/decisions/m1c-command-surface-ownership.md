# M1C Decision: Command-Surface Ownership

## Context

M1C was blocked because seven existing commands had no explicit 0.90/1.0
classification and because the documented 1.0 stable target included commands
whose operational handlers belong to later tasks. Registering those commands
now would create parser-only stable stubs.

## Decision

`StableTargetPending` is a planning and inventory state only. It reserves a
command for the 1.0 target but is not a public runtime command classification:
it must not be registered in the parser, listed by stable help, emitted in
completion metadata, or used to claim operational compatibility.

`StableOperational` requires a parser, real operational handler, fail-closed
behavior, contract tests, help, completion metadata, and documentation. An
owning task promotes a pending command only atomically with all of those
elements. Placeholder and generic not-yet-implemented handlers are rejected.

## Existing command classification

| Command | 0.90 | 1.0 | Ownership condition |
|---|---|---|---|
| `checkpoint` | Internal | Internal | Session/runtime machinery remains behind status, inspect, resume, and recovery. |
| `cluster` | Preview | Preview / post-1.0 | Cluster and multi-agent orchestration remain outside stable 1.0. |
| `council` | Preview | Preview / post-1.0 | Advanced reasoning remains outside the stable lifecycle surface. |
| `override` | Preview transitional | Removed, replacement `approve` | Remove only after T037 delivers operational approval and authority handling; never present it as stable. |
| `evals` | Preview | Preview | Evaluation tooling remains outside the stable control-plane CLI. |
| `trace` | Preview transitional | Removed as a root command | Replace with `inspect` and read-only trace projections only when that surface is operational; never present it as stable. |
| `exec` | Internal | Internal | Direct execution remains hidden because it bypasses the admitted lifecycle. |

The amendment also removes `orchestrate`, `step`, `continue`, `next`, `probe`,
`help-next`, and `govern` immediately in 0.90 with migration diagnostics and
no compatibility aliases. `flow` and `workflow` remain preview.

## Pending implementation ownership

| Command | Owner |
|---|---|
| `approve`, `session abort`, `session cleanup` | T037 |
| `recover inspect`, `recover complete`, `recover restore`, `recover abandon` | T047 and T048 |
| `rpc` | T095 |
| `serve --transport mcp-stdio` | T096 |

T076 qualifies host and projection behavior after T095 and T096. It does not
own RPC or MCP stdio runtime implementation.

## Consequence

M1C may proceed with pruning and classifying the command tree, but it must not
register a pending command or add a placeholder handler. The 0.90 help surface
is an additive subset of the final 1.0 target inventory, not an early claim
that all target commands are operational.
