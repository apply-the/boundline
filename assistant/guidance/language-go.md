# Go Language Guidance

Use Go with explicit packages, small interfaces, and visible concurrency boundaries.

- Organize packages by capability or domain instead of dumping behavior into `service`, `util`, or `common` buckets.
- Pass `context.Context` first for I/O, remote calls, and cancellable work.
- Check and wrap errors with context; use sentinel or typed errors when callers need to branch on them.
- Keep interfaces small and consumer-owned; do not abstract concrete code before you need substitution.
- Keep goroutines, channels, and shutdown semantics explicit instead of spawning hidden background work.
- Prefer readable control flow over clever indirection or framework magic.
