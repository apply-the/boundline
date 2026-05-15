# Concurrency Safety Guardian

Detect unsafe concurrent access patterns that cause data races, deadlocks, or resource leaks.

## Rules

### shared-mutable-state-without-sync
Mutable state accessed from multiple threads or async tasks must be protected by explicit synchronization (mutex, atomic, channel, actor). Undocumented shared mutation is a data race.

Triggers: global mutable variables accessed from multiple goroutines/threads/tasks, struct fields mutated without lock documentation, concurrent map access without sync primitives.

### blocking-in-async-context
Blocking operations (disk I/O, thread sleep, CPU-heavy computation, synchronous HTTP calls) inside async executors starve the event loop and degrade throughput for all tasks.

Triggers: `std::thread::sleep` inside `async fn` (Rust), synchronous file I/O inside `async def` (Python), blocking HTTP calls inside async handlers (Node.js/Go).

### missing-cancellation-handling
Long-running operations must respect cancellation signals (context cancellation in Go, `CancellationToken` in .NET, `AbortSignal` in JS, drop in Rust). Ignoring cancellation wastes resources and delays shutdown.

Triggers: Go functions that accept `context.Context` but never check `ctx.Done()`, .NET async methods without `CancellationToken` parameter, spawned tasks without abort handling.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to all languages with concurrent execution models. Cross-cutting; relevant when code uses threads, async, goroutines, or parallel processing.
