# TypeScript Guidelines

## Principi

TypeScript deve essere usato per modellare vincoli, non solo per aggiungere tipi superficiali a JavaScript. L’obiettivo è spostare errori dal runtime al compile time.

## Configurazione TypeScript

Usare una configurazione strict.

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true
  }
}
```

Disabilitare `strict` rende TypeScript molto meno utile.

## Evitare `any`

`any` disattiva il type checking.

### Da evitare

```ts
function handle(payload: any) {
  return payload.user.id;
}
```

### Preferibile

```ts
type Payload = {
  user: {
    id: UserId;
  };
};

function handle(payload: Payload) {
  return payload.user.id;
}
```

Usare `unknown` quando il tipo non è noto e validarlo.

## Tipi semantici

TypeScript non ha newtype nativi, ma si possono usare branded types.

```ts
type Brand<T, Name extends string> = T & { readonly __brand: Name };

type OrderId = Brand<string, "OrderId">;
type CustomerId = Brand<string, "CustomerId">;
```

```ts
function findOrder(orderId: OrderId) {}
```

## Validazione ai boundary

I dati esterni non sono tipizzati solo perché TypeScript li descrive. Validare input da HTTP, code, database, file e ambiente.

Esempio con schema validator:

```ts
const CreateOrderSchema = z.object({
  customerId: z.string().min(1),
  lines: z.array(z.object({
    productId: z.string().min(1),
    quantity: z.number().int().positive(),
  })),
});

type CreateOrderRequest = z.infer<typeof CreateOrderSchema>;
```

## Union discriminata

Preferire union discriminate per stati alternativi.

```ts
type PaymentStatus =
  | { kind: "pending" }
  | { kind: "completed"; transactionId: TransactionId }
  | { kind: "failed"; reason: string };
```

Questo rende più sicuro il controllo esaustivo.

```ts
function describe(status: PaymentStatus): string {
  switch (status.kind) {
    case "pending":
      return "Pending";
    case "completed":
      return `Completed: ${status.transactionId}`;
    case "failed":
      return `Failed: ${status.reason}`;
  }
}
```

## Dependency injection

Passare dipendenze dall’esterno.

```ts
type OrderServiceDeps = {
  repository: OrderRepository;
  paymentClient: PaymentClient;
};

function createOrderService(deps: OrderServiceDeps) {
  return {
    async createOrder(command: CreateOrderCommand): Promise<OrderId> {
      // ...
    },
  };
}
```

Evitare import diretti di singleton dentro la logica di dominio.

## Error handling

Non affidarsi a `throw` non tipizzato in ogni punto del sistema.

### Approcci possibili

- eccezioni domain-specific ai boundary
- `Result<T, E>` nella logica di dominio
- validazione esplicita input

Esempio `Result`:

```ts
type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };
```

## Async

Gestire sempre le promise.

### Da evitare

```ts
doSomethingAsync();
```

### Preferibile

```ts
await doSomethingAsync();
```

Oppure, se intenzionalmente fire-and-forget:

```ts
void doSomethingAsync().catch((error) => {
  logger.error({ error }, "Background task failed");
});
```

## Resource management

JavaScript non ha RAII diffuso come Rust o C++. Usare `try/finally` per cleanup.

```ts
const connection = await pool.connect();

try {
  await connection.query("BEGIN");
  // ...
  await connection.query("COMMIT");
} catch (error) {
  await connection.query("ROLLBACK");
  throw error;
} finally {
  connection.release();
}
```

## Logging

Usare structured logging.

```ts
logger.info({ orderId, customerId }, "Order created");
```

Regole:

- non loggare token o segreti
- includere correlation ID
- non usare `console.log` in codice applicativo server-side
- non serializzare errori perdendo stack trace

## Test

Preferire test su funzioni pure e adapter fake.

```ts
const service = createOrderService({
  repository: new FakeOrderRepository(),
  paymentClient: new FakePaymentClient(),
});
```

## Cose da evitare

- `any`
- `as` usato per zittire il compilatore
- singleton importati nella business logic
- promise non awaitate
- union senza discriminante
- DTO esterni usati come dominio senza validazione
- `console.log` in servizi
- configurazione letta ovunque da `process.env`
