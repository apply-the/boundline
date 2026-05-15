# TypeScript And Node

TypeScript must be used to model constraints, not just to add superficial types to JavaScript. The goal is to shift errors from runtime to compile time.

## Configuration

Use a strict configuration: `strict: true`, `noUncheckedIndexedAccess: true`, `exactOptionalPropertyTypes: true`, `noImplicitOverride: true`. Disabling strict makes TypeScript significantly less useful.

## Avoid `any`

`any` disables type checking. Use `unknown` when the type is not known and validate it explicitly. Never use `any` at system boundaries where external data enters.

## Semantic Types

TypeScript lacks native newtypes but branded types provide the same safety:

```ts
type Brand<T, Name extends string> = T & { readonly __brand: Name };
type OrderId = Brand<string, "OrderId">;
type CustomerId = Brand<string, "CustomerId">;
```

## Boundary Validation

External data is not typed just because TypeScript describes it. Validate input from HTTP, queues, database, file, and environment using schema validators:

```ts
const CreateOrderSchema = z.object({
  customerId: z.string().min(1),
  lines: z.array(z.object({
    productId: z.string().min(1),
    quantity: z.number().int().positive(),
  })),
});
```

## Discriminated Unions

Prefer discriminated unions for alternative states:

```ts
type PaymentStatus =
  | { kind: "pending" }
  | { kind: "completed"; transactionId: TransactionId }
  | { kind: "failed"; reason: string };
```

This enables exhaustive checking and makes state transitions explicit.

## Dependency Injection

Pass dependencies from the outside. Avoid direct imports of singletons inside domain logic.

```ts
type OrderServiceDeps = {
  repository: OrderRepository;
  paymentClient: PaymentClient;
};
```

## Error Handling

Do not rely on untyped `throw` everywhere. Use domain-specific exceptions at boundaries or `Result<T, E>` in domain logic. Validate input explicitly.

## Async

Always handle promises. Never fire-and-forget without error observation:

```ts
await doSomethingAsync();
```

If intentionally fire-and-forget, catch and log errors explicitly.

## Recommended Ecosystem Libraries

| Category | Package | Purpose |
|----------|---------|---------|
| Validation | `zod` | Runtime schema validation with type inference |
| Logging | `pino` | Structured, high-performance JSON logging |
| HTTP client | native `fetch` or `ky` | Prefer platform fetch; `ky` for retry/hooks |
| Database | `prisma` or `drizzle-orm` | Type-safe database access |
| Testing | `vitest` or `jest` | Unit and integration test runner |
| HTTP mocks | `msw` | Service worker-based request interception |
| API testing | `supertest` | HTTP assertion library for Express/Fastify |
| Date/time | `date-fns` or `dayjs` | Immutable, tree-shakeable date utilities |
| Linting | `eslint`, `prettier` | Code quality and formatting |
| Type checking | `tsc --noEmit` | Type verification without build step |

Prefer packages with TypeScript-first APIs and minimal dependency footprint.

## Anti-Patterns

- `any` usage at system boundaries
- Manual `as` casting without runtime validation
- Unvalidated external JSON parsed as typed objects
- Duplicated schema and type definitions that drift apart
- Direct singleton imports in domain logic
- Unhandled promise rejections
- `strict: false` in production configuration
- Old-style callbacks when async/await is available

## Guardian Hooks

Guardians that apply to this guidance:
- `ts_runtime_validation`: unchecked-external-json, any-at-boundary, duplicated-schema-and-type
- `clean_code`: no-primitive-obsession, no-hidden-side-effects
- `architecture_boundary`: dependency-direction
