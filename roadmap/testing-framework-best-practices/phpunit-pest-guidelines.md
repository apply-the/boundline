# PHPUnit and Pest Guidelines

## Principi

PHPUnit e Pest devono testare comportamento applicativo e dominio senza avviare l’intero framework quando non serve. Pest migliora leggibilità, ma non corregge design poco testabile.

## PHPUnit

```php
final class OrderServiceTest extends TestCase
{
    public function testItRejectsEmptyOrder(): void
    {
        // ...
    }
}
```

## Pest

```php
it('rejects empty order', function (): void {
    // ...
});
```

## Assertions

Usare assertion specifiche.

```php
$this->assertEquals($expected, $actual);
$this->expectException(InvalidOrderException::class);
```

## Data providers

Usare data provider/dataset per casi tabellari.

## Laravel/Symfony

Non usare test framework full-stack per logica pura.

- unit test per value object/service
- feature test per endpoint
- integration test per DB
- browser test solo flussi critici

## Mock

Usare mock per boundary esterni. Preferire fake repository semplici.

## Database

Per test DB:

- transazioni/refresh database
- factory leggibili
- niente dati manuali condivisi
- evitare test dipendenti dall’ordine

## Anti-pattern

- test che avviano Laravel per ogni value object
- mock di tutto
- factory troppo magiche
- assertion vaghe
- dipendenza da stato DB preesistente
