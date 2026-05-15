# Laravel Guidelines

## Principi

Laravel è produttivo e ricco di convenzioni. Il rischio è mettere troppa logica in controller, model Eloquent, facade e helper globali. Tenere espliciti casi d’uso e dipendenze.

## Controller sottili

Controller:

- request validation
- authorization
- chiamata action/service
- response

```php
final class CreateOrderController
{
    public function __invoke(CreateOrderRequest $request, CreateOrderAction $action): JsonResponse
    {
        $result = $action->execute($request->toCommand());

        return response()->json(CreateOrderResource::fromResult($result), 201);
    }
}
```

## Form Request

Usare Form Request per validazione boundary e autorizzazione semplice. Non mettere business logic complessa nella Form Request.

## Action/Application Service

Casi d’uso complessi in action/service dedicati.

```php
final readonly class CreateOrderAction
{
    public function __construct(
        private OrderRepository $orders,
        private PaymentClient $paymentClient,
    ) {}
}
```

## Eloquent

Regole:

- evitare model giganteschi
- evitare query complesse nei controller
- usare eager loading per evitare N+1
- non esporre model direttamente se l’API richiede controllo
- usare casts/value object dove utile
- attenzione a mass assignment

## Facades

Le Facades sono comode, ma nascondono dipendenze. Nel dominio e nei service preferire injection.

## Transactions

Usare transazioni nei casi atomici. Evitare chiamate remote dentro transazioni DB.

## Events e jobs

Regole:

- eventi espliciti
- listener idempotenti
- job con retry controllato
- payload piccolo
- error handling e logging

## Resources

Usare API Resources per controllare output e non esporre campi sensibili.

## Testing

- unit test su action/domain
- feature test su endpoint
- database transaction/reset coerente
- factory leggibili
- evitare test che dipendono da Facade magic se si può iniettare

## Anti-pattern

- controller enormi
- model Eloquent con tutto
- Facades ovunque nella business logic
- helper globali per logica importante
- N+1
- mass assignment non controllato
- job non idempotenti
