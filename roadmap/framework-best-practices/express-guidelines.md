# Express Guidelines

## Principi

Express è minimale. Questo è un vantaggio solo se il team definisce struttura, error handling, validazione e osservabilità. Altrimenti il progetto diventa rapidamente una somma di route caotiche.

## Struttura

Separare:

- routes
- handlers/controllers
- application services
- domain
- repositories/adapters
- middleware
- config

```text
src/
  routes/
  handlers/
  application/
  domain/
  infrastructure/
  middleware/
```

## Handler sottili

```ts
app.post("/orders", asyncHandler(async (req, res) => {
  const command = parseCreateOrderRequest(req.body);
  const result = await orderService.createOrder(command);
  res.status(201).json(toResponse(result));
}));
```

Non mettere tutto nella route.

## Async error handling

Express non gestisce sempre automaticamente errori async a seconda della versione e setup. Usare wrapper o middleware coerente.

```ts
const asyncHandler =
  (handler: RequestHandler): RequestHandler =>
  (req, res, next) => {
    Promise.resolve(handler(req, res, next)).catch(next);
  };
```

## Validazione

Usare schema validation per input.

```ts
const schema = z.object({
  customerId: z.string().uuid(),
  lines: z.array(orderLineSchema).min(1),
});
```

## Error middleware

Definire error middleware unico.

```ts
app.use((error, req, res, next) => {
  // map application errors to HTTP
});
```

Non rispondere con stack trace in produzione.

## Middleware

Middleware per cross-cutting concerns:

- auth
- correlation ID
- request logging
- rate limiting
- body parsing
- security headers

Non business logic.

## Security

Usare:

- helmet o equivalente
- CORS restrittivo
- rate limiting
- body size limit
- input validation
- cookie sicuri se usati
- CSRF se cookie-based

## Config

Validare env all’avvio. Non usare `process.env` in ogni modulo.

## Testing

Usare supertest per integration HTTP. Testare service senza Express.

## Anti-pattern

- route file da migliaia di righe
- validazione manuale sparsa
- error handling duplicato
- `try/catch` in ogni handler invece di middleware coerente
- business logic nei middleware
- `process.env` ovunque
