# PHP Guidelines

## Principi

PHP moderno può essere robusto se si usano strict types, value object, dependency injection e validazione ai boundary. Evitare codice globale, array associativi non tipizzati e service statici.

## Strict types

Abilitare strict types in ogni file applicativo.

```php
<?php

declare(strict_types=1);
```

## Tipi e proprietà readonly

Usare type declarations e `readonly` dove possibile.

```php
final readonly class OrderId
{
    public function __construct(public string $value)
    {
        if ($value === '') {
            throw new InvalidArgumentException('OrderId cannot be empty');
        }
    }
}
```

## Evitare array come modello di dominio

### Da evitare

```php
function createOrder(array $data): void
{
    $customerId = $data['customer_id'];
}
```

### Preferibile

```php
final readonly class CreateOrderCommand
{
    public function __construct(
        public CustomerId $customerId,
        /** @var list<OrderLine> */
        public array $lines,
    ) {}
}
```

Gli array vanno bene ai boundary, non come dominio interno.

## Dependency injection

Passare dipendenze nel costruttore.

```php
final class OrderService
{
    public function __construct(
        private OrderRepository $repository,
        private PaymentClient $paymentClient,
    ) {}
}
```

Evitare `new` di client infrastrutturali dentro i service.

## Error handling

Usare eccezioni specifiche.

```php
final class OrderNotFoundException extends RuntimeException
{
    public function __construct(OrderId $orderId)
    {
        parent::__construct("Order not found: {$orderId->value}");
    }
}
```

Regole:

- non catturare `Throwable` genericamente senza motivo
- non nascondere errori
- non usare error suppression operator `@`
- distinguere errori di dominio da infrastruttura

## Null

Non usare `null` come stato generico. Usare nullable solo quando l’assenza è parte del modello.

```php
public function findById(OrderId $id): ?Order
```

Per risultati più ricchi, usare result object.

## Resource management

Usare `try/finally` per cleanup.

```php
$connection = $pool->getConnection();

try {
    // ...
} finally {
    $connection->close();
}
```

## Logging

Usare PSR-3 logger.

```php
$this->logger->info('Order created', [
    'order_id' => $orderId->value,
    'customer_id' => $customerId->value,
]);
```

Non usare `var_dump`, `print_r` o `echo` in codice applicativo.

## Test

Constructor injection permette fake o mock semplici.

```php
$service = new OrderService(
    new FakeOrderRepository(),
    new FakePaymentClient(),
);
```

Usare test su value object e servizi senza bootstrap completo del framework quando possibile.

## Cose da evitare

- file con codice globale
- array associativi ovunque
- `mixed` senza necessità
- `@` error suppression
- service statici
- dipendenze create dentro service
- `var_dump` e `print_r`
- mutabilità diffusa
- magic methods usati per nascondere comportamento importante
