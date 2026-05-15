# Shell And Automation

Shell is suited for glue code, simple automation, and orchestration. It is not suited for complex business logic. If a script grows beyond ~200 lines with branching logic, consider a more structured language.

## Bash Configuration

Use conservative settings:

```bash
#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
```

Note: `set -e` has edge cases. It does not replace explicit error handling.

## Quoting

Almost all variables must be quoted:

```bash
rm -rf -- "$TARGET_DIR"
```

Unquoted variables are a source of word-splitting bugs, glob expansion, and injection vulnerabilities.

## Input Validation

Validate arguments before use:

```bash
if [[ $# -ne 1 ]]; then
  echo "usage: $0 <target-dir>" >&2
  exit 2
fi
target_dir=$1
```

## Small Functions

Keep functions focused. Use local variables. Return meaningful exit codes.

```bash
log_info() {
  printf '[INFO] %s\n' "$*" >&2
}
```

## Temporary Files

Use `mktemp` for temporary files. Clean up with traps:

```bash
tmp_dir=$(mktemp -d)
trap 'rm -rf -- "$tmp_dir"' EXIT
```

## PowerShell

Use strict mode and explicit error handling:

```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
```

Prefer cmdlet parameters over positional arguments. Use `try/catch` for error handling. Output structured objects rather than text parsing.

## Recommended Ecosystem Tools

### Bash / POSIX Shell

| Category | Tool | Purpose |
|----------|------|---------|
| Linting | `shellcheck` | Static analysis for shell scripts |
| Testing | `bats-core` | TAP-compliant test framework |
| JSON | `jq` | JSON query and transformation |
| YAML | `yq` | YAML query and transformation |
| Templating | `envsubst` | Environment variable substitution |
| Parallelism | `GNU parallel` or `xargs -P` | Parallel command execution |

### PowerShell

| Category | Tool | Purpose |
|----------|------|---------|
| Testing | Pester | BDD-style test framework |
| Linting | PSScriptAnalyzer | Static analysis rules |
| Secrets | SecretManagement module | Vault-agnostic secret access |

## Anti-Patterns

- Missing `set -euo pipefail` in bash scripts
- Unquoted variables in commands
- Scripts that grow beyond their appropriate scope
- Missing cleanup for temporary resources
- Parsing structured data with grep/sed/awk when a real parser exists
- Hardcoded paths instead of configurable variables
- Silent failure on command errors
- Using `eval` with untrusted input

## Guardian Hooks

Guardians that apply to this guidance:
- `security_boundary`: command injection risks from unquoted variables or eval
- `clean_code`: no-hidden-side-effects (scripts with undisclosed mutations)
- `operations_readiness`: reproducibility and cleanup
