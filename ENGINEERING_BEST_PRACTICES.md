# Engineering Excellence & Modern Stacks (2024-2026)

This report outlines the exhaustive idiomatic engineering standards, recommended libraries, and anti-patterns for the Boundline ecosystem. It serves as the authoritative source for `language-idiom-guardian`, `framework-guardian`, and `test-framework-guardian` capabilities.

---

## 1. Programming Languages (LTS & Modern Focused)

### 🦀 Rust
- **Version-Aware Mapping:**
  - **Active Support Window:** Rust 1.70+ (Stable toolchain).
  - **Legacy Warning:** Pre-2021 Edition codebases; missing modern edition migrations.
  - **Target Excellence:** **Rust 2024 Edition** (1.85.0+).
- **Module Organization:**
  - **No `mod.rs`:** Use the modern layout (e.g., `src/domain.rs` and `src/domain/user.rs`) to avoid duplicate file names in the IDE and improve navigation.
  - **Visibility:** Keep items private by default; use `pub(crate)` for internal sharing and `pub` only for the public API.
- **Modern Features (Rust 2024):**
  - **RPITIT (Return-Position impl Trait in Traits):** Implicitly captures all in-scope generics/lifetimes.
  - **AFIT (Async Functions in Traits):** Native support for `async fn` in traits (Static dispatch optimized).
  - **Precise Capturing:** Use of `use<..>` syntax for explicit lifetime control.
- **Idiomatic Stack:**
  - **Error Handling:** `anyhow` (Apps/Binaries), `thiserror` (Libraries/Crates), `snafu` (Large Systems).
  - **Domain Modeling:** Use **Newtypes** (Tuple Structs like `struct UserId(String)`) to wrap primitives. Never pass raw `String` or `i32` for domain IDs or quantities.
- **Zero-Panic Policy (Exhaustive):**
  - **Forbidden in Library/Production Code:** 
    - Macros: `panic!`, `todo!`, `unimplemented!`, `assert!`, `assert_eq!`, `assert_ne!`.
    - Methods: `.unwrap()`, `.expect()`, `.unwrap_err()`.
    - Indexing: Avoid `slice[index]` (may panic); use `.get(index)` or `.get_mut(index)` which return `Option`.
  - **Allowed Exceptions:** `main.rs` entry point, test code (`#[test]`), or documented "impossible" invariants (with clear comments).
- **Anti-Patterns:**
  - `Box<dyn Error>` in production logic (erases type safety).
  - `std::sync::Mutex` held across `await` points (use `tokio::sync::Mutex`).
  - `anyhow` in public library APIs (prohibits consumer matching).
  - "Primitive Obsession": Using `String` for everything instead of domain-specific types.

### 📘 TypeScript / Node.js
- **Version-Aware Mapping:**
  - **Active Support Window:** Node.js 18, TypeScript 5.0+.
  - **Legacy Warning:** Node.js 16 or older; TypeScript 4.x; missing `strict` mode.
  - **Target Excellence:** Node.js 20/22 (LTS), TypeScript 5.4+.
- **Modern Features:** `satisfies` operator, Discriminated Unions, Top-level `await`.
- **Idiomatic Stack:**
  - **Validation:** `Zod` (standard), `Valibot` (modular/tiny).
  - **Frameworks:** `Hono` (fast/type-safe), `tRPC` (end-to-end safety).
  - **ORM:** `Drizzle ORM` (TypeScript-first), `Kysely` (Type-safe SQL).
- **Anti-Patterns:**
  - `any` usage; manual `as` casting without validation.
  - Disabling `strict: true`; "Stringly-typed" state (use unions).
  - Duplicating Zod schemas and TypeScript interfaces (use `z.infer<T>`).

### 🐹 Go
- **Version-Aware Mapping:**
  - **Active Support Window:** Go 1.18+ (Generics support).
  - **Legacy Warning:** Pre-1.18 (lack of Generics); using `pkg/errors`.
  - **Target Excellence:** Go 1.22+ (Toolchain management, new `mux` routing).
- **Modern Features:** `errors.Join` (1.20), `log/slog` (1.21), Enhanced Routing (1.22).
- **Idiomatic Stack:**
  - **Error Handling:** Standard library `errors` (wrapping with `%w`, `errors.Is/As`).
  - **Logging:** `log/slog` (standard library).
  - **Concurrency:** `golang.org/x/sync/errgroup` for managing groups of tasks.
- **Anti-Patterns:**
  - "Log and Return": Logging an error and then returning it (choose one).
  - Capitalized or punctuated error strings (`errors.New("Failed.")` is wrong).
  - `panic` for business flow.

### 🐍 Python
- **Version-Aware Mapping:**
  - **Active Support Window:** Python 3.9+.
  - **Legacy Warning:** Python 3.8 or older; using `os.path` over `pathlib`; old-style type hints.
  - **Target Excellence:** Python 3.11+ (Exception Groups) / 3.12 (Performance).
- **Modern Features:** `ExceptionGroup` (for concurrent tasks), `asyncio.TaskGroup`, `add_note()`.
- **Idiomatic Stack:**
  - **Validation:** `Pydantic v2` (Rust-backed).
  - **Logging:** `structlog` (structured JSON).
  - **API:** `FastAPI`, `Django Ninja`.
  - **Package Management:** `uv` or `Poetry`.
- **Anti-Patterns:**
  - Swallowing exceptions (`except Exception: pass`).
  - Missing `raise ... from exc` when re-raising.
  - Legacy string formatting (`%` or `.format()`); use `f-strings`.

### ☕ Java
- **Version-Aware Mapping:**
  - **Active Support Window:** Java 11+.
  - **Legacy Warning:** Java 8 (EOL); heavy use of traditional Threads over modern concurrency; complex `if-else` chains over `switch`.
  - **Target Excellence:** **Java 21 (LTS)** (Virtual Threads, Pattern Matching).
- **Modern Features:** **Sealed Classes**, **Pattern Matching**, **Virtual Threads** (Java 21).
- **Idiomatic Stack:**
  - **Error Modeling:** Sealed hierarchies for `Result<T>`.
  - **Resilience:** `Failsafe`, `Resilience4j`.
  - **Functional:** `Vavr` (Either/Try).
  - **Frameworks:** Spring Boot 3.x, Micronaut, Quarkus.
- **Anti-Patterns:**
  - `Optional` for parameters/fields (use only as return type).
  - `Checked Exceptions` for expected logic.
  - `synchronized` on Virtual Threads (use `ReentrantLock`).

### 🎯 C# / .NET
- **Version-Aware Mapping:**
  - **Active Support Window:** .NET 6+.
  - **Legacy Warning:** .NET Framework; .NET 5 or older; lack of async/await.
  - **Target Excellence:** **.NET 8 (LTS)**.
- **Modern Features:** Primary Constructors, `System.Collections.Frozen`, `TimeProvider`.
- **Idiomatic Stack:**
  - **Error Handling:** `Result Pattern` (OneOf, FluentResults).
  - **Resilience:** `Polly`.
  - **Validation:** `FluentValidation`.
  - **API Errors:** `ProblemDetails` (built-in in .NET 8).
- **Anti-Patterns:**
  - Hardcoded `DateTime.Now` (use `TimeProvider` for testability).
  - Ignoring `CancellationToken` in async methods.
  - Logic inside `catch` blocks (should only log, notify, or wrap).

---

## 2. Framework-Specific Excellence

### 🌐 Frontend
- **Angular (v17+):**
  - **Signals:** Prefer over Zone.js for reactivity.
  - **Standalone:** Mandatory; `NgModules` are legacy.
- **React:**
  - **Server Components:** Default for Next.js 14/15.
  - **TanStack Query:** Standard for server state.
- **Svelte (v5):**
  - **Runes:** Use `$state`, `$derived`.
- **Vue (v3.4+):**
  - **Composition API:** Default; `defineModel` for two-way binding.

### ⚙️ Backend
- **Next.js:** **App Router** is mandatory.
- **NestJS:** SWC Builder for speed; functional interceptors.
- **FastAPI:** `Annotated` dependencies; Pydantic v2.
- **Django:** Use **HTMX** for interactivity; Django Ninja for APIs.
- **Rails (7.1+):** **Hotwire** (Turbo/Stimulus); `Solid Queue` (Redis-free).
- **Laravel (11):** **Livewire/Volt**; Pest for testing.
- **Express:** Transition to **Hono** or **Fastify** for modern type-safety.

---

## 3. Testing Frameworks & Strategies

### 🧪 Unit & Integration
- **Vitest:** Primary for JS/TS; use MSW (Mock Service Worker) for network.
- **Pytest:** Use fixtures; avoid `unittest` style classes.
- **JUnit 5:** Use `@ParameterizedTest` and AssertJ.
- **Rust-testing:** Avoid `Box<dyn Error>` in `#[test]`; use `anyhow::Result`.

### 🎭 End-to-End (E2E)
- **Playwright:** **Web-First Assertions**; User-centric locators (`getByRole`).
- **Cypress:** Use `data-testid`; avoid `cy.wait(N)`.

### 📊 Strategy & Data
- **Contract-First:** Use **Pact** or **Prism** for API contracts.
- **Data Patterns:** **Data Builders** over static JSON fixtures.
- **API Seeding:** Seed state via API calls in `beforeEach` instead of UI navigation.

---

## 4. Methodology, Tooling & Infrastructure

### 🛠️ Tooling
- **Package Management:** `uv` (Python), `pnpm` (JS), `cargo` (Rust).
- **Observability:** **OpenTelemetry** for all telemetry.
- **Static Analysis:** **PHPStan** (PHP), **Error Prone** (Java), **Clippy** (Rust).

### 🏗️ Design & Delivery
- **Shift-Left:** Early architecture review (S2.1 Guardians) before code.
- **Trunk-Based:** Single-branch development with short-lived feature flags.
- **Clean Code (Internal Policy):**
  - Names express **Intent** (why), not implementation.
  - No magic numbers/strings in domain logic.
  - Validate input at the **Boundary**.
  - **Log OR Return**, never both.
  - **Sovereign Data:** Bounded contexts own their own persistence.
  - **Resilience:** Circuit Breakers/Bulkheads built into architecture.
