# Testing Go

Testing conventions for Go projects using the standard testing package and common tools.

## Test Organization

- Test files: `*_test.go` co-located with source
- Test functions: `TestXxx(t *testing.T)` naming convention
- Table-driven tests for multiple cases
- Subtests with `t.Run` for grouping

## Table-Driven Tests

```go
func TestParseName(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    Name
        wantErr bool
    }{
        {"valid", "Alice", Name{Value: "Alice"}, false},
        {"empty", "", Name{}, true},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := ParseName(tt.input)
            if (err != nil) != tt.wantErr {
                t.Fatalf("error = %v, wantErr %v", err, tt.wantErr)
            }
            if got != tt.want {
                t.Errorf("got %v, want %v", got, tt.want)
            }
        })
    }
}
```

## Test Helpers

Use `t.Helper()` in test helper functions. Use `testify` for complex assertions. Create test factories for domain objects.

## Mocking

Use interfaces at boundaries. Create test implementations (fakes) rather than mock generation libraries when possible. Keep interfaces small.

## Integration Tests

Use build tags or `testing.Short()` to separate fast and slow tests. Use `TestMain` for shared setup/teardown.

## Recommended Tools

| Tool | Purpose |
|------|---------|
| `testify` | Assertions, suite runner, mocks |
| `go-cmp` | Deep equality with diff output |
| `testcontainers-go` | Docker-based test dependencies |
| `goleak` | Goroutine leak detection in tests |
| `httptest` (stdlib) | HTTP handler and server testing |
| `gomock` | Interface-based mock generation |
| `gofakeit` | Realistic fake data generation |

## Anti-Patterns

- Not using table-driven tests for multiple cases
- Missing `t.Helper()` in helper functions
- Large interfaces that are hard to fake
- Tests that depend on file system paths or network
- Missing error case testing
- Tests that share mutable state

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `clean_code`: test readability
