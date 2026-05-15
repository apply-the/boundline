# Phase 2 Manifest — Language Guidance

## Included Guidance

```text
guidance/language-go.md
guidance/language-python.md
guidance/language-jvm.md
guidance/language-dotnet.md
guardians/language-guardian-rule-seeds.md
```

## Pillars Covered

- error handling
- concurrency
- type safety
- runtime validation
- testing
- observability
- framework boundaries
- maintainability
- legacy warnings
- target excellence

## Suggested Pack Mapping

```toml
[guidance.language_go]
path = "guidance/language-go.md"
applies_to = ["planning", "implementation", "review", "testing", "refactor"]

[guidance.language_python]
path = "guidance/language-python.md"
applies_to = ["planning", "implementation", "review", "testing", "refactor"]

[guidance.language_jvm]
path = "guidance/language-jvm.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "refactor"]

[guidance.language_dotnet]
path = "guidance/language-dotnet.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "refactor"]
```

## Suggested Guardians

```toml
[guardians.go_error_ownership]
path = "guardians/language-guardian-rule-seeds.md"
rules = ["go-log-or-return", "go-error-wrapping", "go-panic-policy"]

[guardians.python_runtime_boundary]
path = "guardians/language-guardian-rule-seeds.md"
rules = ["python-exception-chaining", "python-boundary-validation", "python-no-swallowed-exceptions"]

[guardians.jvm_modernization]
path = "guardians/language-guardian-rule-seeds.md"
rules = ["java-legacy-runtime", "java-virtual-thread-locking", "java-optional-misuse"]

[guardians.dotnet_async_testability]
path = "guardians/language-guardian-rule-seeds.md"
rules = ["dotnet-cancellation-token", "dotnet-timeprovider", "dotnet-problemdetails"]
```

## Authority Strength

Default strength: recommendation.

Rules may become warning, blocker, or mandatory only when promoted by:

- workspace override
- Canon-governed standard
- organization policy
- explicit expert pack configuration
