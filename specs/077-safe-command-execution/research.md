# Research: Safe Command Execution and Evidence Capture

## Decision 1: Command Intent Classification

**Decision**: Deterministic rule engine — whitelist + argument heuristics.

**Rationale**: The spec requires deterministic, fast, testable, auditable classification. A two-pass approach satisfies all constraints: pass 1 matches the command name (e.g., `rm` → `mutate`, `cargo test` → `test`), pass 2 refines by scanning argument flags (`--dry-run` downgrades, `--force` escalates). Unknown commands default to `mutate` (fail-safe). Path-based inspection is secondary context only.

**Alternatives considered**:
- LLM-based classification: rejected — non-deterministic, adds latency, hard to test
- Pure whitelist only: rejected — misses argument-level refinements (e.g., `git status` vs `git push`)
- Path-based primary: rejected — fragile (files can be anywhere)

## Decision 2: Dry-Run Implementation

**Decision**: Tiered deterministic mechanism with four result statuses.

**Rationale**: Full OS-level virtualization (containers, snapshots, syscall interception) is deferred to B13B. V1 uses: `native_dry_run_executed` for commands with known safe modes (e.g., `cargo check`, `terraform plan`), `read_only_executed` for read-only commands, `plan_only` for mutating commands with no native dry-run (emit plan, do NOT execute), `unsupported_for_safe_dry_run` for unknown commands.

**Alternatives considered**:
- Filesystem snapshot/rollback: rejected — complex, OS-dependent, performance cost
- Strace/ptrace interception: rejected — requires root/docker, fragile across OS
- Pre-execution analysis only: rejected — cannot predict all side effects

## Decision 3: Execution Policy Format

**Decision**: TOML-based Intent × Zone matrix with optional command-pattern overrides.

**Rationale**: TOML is already used by Boundline (`.boundline/config.toml`). The matrix is simple (6 intents × 3 zones = 18 cells), auditable, and aligned with Canon's risk zone model. Resolution order: classify intent → apply overrides → resolve matrix → apply safety flags → final mode.

**Alternatives considered**:
- Pure command-pattern rules: rejected — infinite list, fragile
- JSON schema: rejected — TOML is more readable for config files
- Hardcoded policy: rejected — must be configurable per project

## Decision 4: Secret Redaction

**Decision**: Curated regex patterns with built-in defaults + `.boundline/redaction.toml`.

**Rationale**: Regex is deterministic, fast, and covers the most common secret formats (API keys, tokens, JWT). Built-in defaults reduce initial configuration burden. Per-pattern severity, replacement strategy, and allowlist rules handle false positives. Redaction metadata (pattern ID, count) is recorded without storing the original value.

**Alternatives considered**:
- Pattern DSL: rejected — custom language adds learning curve without benefit over regex
- External scanner (trufflehog/gitleaks): rejected — external dependency, adds latency
- No built-in defaults: rejected — every project would need to configure from scratch

## Decision 5: Evidence Capture Limits

**Decision**: Configurable hard limits with safe defaults.

**Rationale**: 1MB per stream covers most real-world commands without unbounded growth. SHA-256 (first 12 hex chars) for trace IDs is collision-resistant enough for per-machine scope. 10K mutation boundary entries handles large codebases without memory issues. All limits are defaults, overridable in `.boundline/evidence-limits.toml`.

**Alternatives considered**:
- No limits: rejected — unbounded growth risk
- Hardcoded limits: rejected — different projects have different needs
- UUID trace IDs: rejected — less debuggable than timestamp+hash
