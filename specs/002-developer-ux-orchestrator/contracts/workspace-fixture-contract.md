# Contract: Workspace Fixture

## Purpose

Defines the repository-local workspace fixture manifest used by `boundline run` and the session runtime.

## Manifest Shape

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Stable fixture name shown in run output and traces |
| `test_command.program` | Yes | Local command used to verify the workspace state |
| `test_command.args` | No | Ordered arguments passed to the verification command |
| `limits` | Yes | Explicit bounded step, retry, and replanning limits |
| `file_patches` | Yes | Ordered patch instructions with `path`, `find`, and `replace` |

## Behavioral Guarantees

- The fixture is fully local and deterministic.
- The verification command must fail before implementation begins and pass after the patch set is applied for a successful red-to-green run.
- The fixture remains bounded by the same run-limit rules as the core orchestrator.
- Successful and non-success runs both produce traces suitable for later inspection with `boundline inspect`.

## Failure Contract

- If the fixture manifest is missing or invalid, the run must fail before task execution starts.
- If a patch target cannot be found or verification still fails after patches are applied, the run must report the final reason and leave behind an inspectable trace.