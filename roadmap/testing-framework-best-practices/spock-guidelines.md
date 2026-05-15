# Spock Guidelines

## Principi

Spock rende i test JVM molto leggibili grazie a `given/when/then` e data tables. Il rischio è abusare di mock interaction testing e creare specifiche troppo accoppiate all’implementazione.

## Struttura

```groovy
def 'rejects empty order'() {
    given:
    def command = CreateOrderCommand.empty()

    when:
    def result = service.createOrder(command)

    then:
    result == CreateOrderResult.invalid('empty_order')
}
```

## Data tables

Usare data tables per casi tabellari.

```groovy
def 'rejects invalid emails'() {
    expect:
    !EmailAddress.isValid(value)

    where:
    value << ['', 'invalid', 'missing-at.example.com']
}
```

## Mocking

Spock mock è potente. Usarlo solo quando le interazioni sono il comportamento.

```groovy
1 * eventPublisher.publish({ it.type == 'OrderCreated' })
```

Non verificare ogni chiamata interna se il risultato osservabile basta.

## Setup

`setup` deve essere breve. Se cresce, usare factory/builder.

## Spring con Spock

Non avviare Spring context per unit test puri. Usare Spring integration test solo quando si testa wiring/framework.

## Anti-pattern

- troppe interaction assertions
- test che verificano ordine di chiamate irrilevante
- `setup` enorme
- data table con troppe colonne
- specifiche che duplicano implementazione
