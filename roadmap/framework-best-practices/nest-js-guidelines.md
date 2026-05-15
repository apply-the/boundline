# NestJS Guidelines

## Principi

NestJS porta pattern enterprise nel mondo Node. Va usato per avere struttura e boundary chiari, non per creare classi pesanti e dependency graph opachi.

## Moduli

Organizzare per feature, non per tipo tecnico.

### Preferire

```text
orders/
  orders.module.ts
  orders.controller.ts
  orders.service.ts
  order.repository.ts
```

### Evitare

```text
controllers/
services/
repositories/
```

per progetti grandi, perché disperde la feature.

## Controller sottili

Controller:

- route
- DTO input
- auth guard
- chiamata service
- response mapping

Non business logic pesante.

## Provider e dependency injection

Usare DI Nest, ma mantenere dipendenze esplicite.

```ts
@Injectable()
export class OrderService {
  constructor(
    private readonly repository: OrderRepository,
    private readonly paymentClient: PaymentClient,
  ) {}
}
```

## DTO e validation

Usare validation pipe o schema validation. Non fidarsi dei DTO TypeScript a runtime.

```ts
export class CreateOrderDto {
  @IsUUID()
  customerId!: string;

  @ValidateNested({ each: true })
  lines!: CreateOrderLineDto[];
}
```

## Domain model separato

Non lasciare che DTO, entity ORM e domain model siano lo stesso oggetto.

Separare:

- request DTO
- command
- domain model
- persistence entity
- response DTO

## Error handling

Usare exception filter o mapping coerente.

Non lanciare `Error` generico per casi di dominio.

## Config

Usare config module validato.

Regole:

- leggere env al bootstrap
- validare
- passare config tipizzata
- non accedere a `process.env` ovunque

## Database

Con TypeORM/Prisma/Mongoose:

- repository/adapters separati
- evitare query nel controller
- gestire transazioni esplicitamente
- evitare N+1
- non esporre entity direttamente

## Async

Gestire errori promise. Non fire-and-forget senza logging e recovery.

## Testing

- unit test su service con fake repository
- e2e test su controller principali
- integration test su database con container
- evitare over-mocking del framework

## Anti-pattern

- moduli globali per tutto
- controller con business logic
- DTO usati come dominio
- `process.env` sparso
- provider ciclici
- exception generiche
- entity ORM esposte al client
