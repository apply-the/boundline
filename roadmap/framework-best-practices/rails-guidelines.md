# Rails Guidelines

## Principi

Rails premia convenzione e velocità, ma senza disciplina produce fat model, callback invisibili e coupling forte al framework. Mantenere casi d’uso e dominio leggibili.

## Controller sottili

Controller:

- autorizzazione
- parametri
- chiamata use case/service
- response/render/redirect

Non business logic.

## Fat model con criterio

“Fat model, skinny controller” non significa mettere tutto in ActiveRecord.

Il model può contenere invariant locali e comportamenti vicini ai dati. Orchestrazioni complesse vanno in service/use case.

## Service object/use case

Usare oggetti applicativi per casi d’uso complessi.

```ruby
class CreateOrder
  def initialize(repository:, payment_client:)
    @repository = repository
    @payment_client = payment_client
  end

  def call(command)
    # orchestration
  end
end
```

## ActiveRecord

Regole:

- evitare query sparse in view/controller
- evitare N+1 con `includes`, `preload`, `eager_load`
- usare transaction nei casi d’uso atomici
- evitare callback per side effect esterni
- non inviare email o chiamate HTTP in callback model

## Callbacks

Usare callback con cautela.

Accettabili per invariant locali semplici. Per side effect importanti, preferire flusso esplicito.

## Validations

Le validations ActiveRecord proteggono persistenza. Le regole di dominio critiche devono essere comprensibili e testabili anche come comportamento applicativo.

## Strong parameters

Non fidarsi mai dei parametri raw.

## Background jobs

Regole:

- job idempotenti
- retry consapevole
- payload piccolo
- evitare serializzare oggetti grandi
- log e correlation ID
- dead letter/retry exhaustion gestiti

## Testing

- model tests per invariant
- request specs per endpoint
- service tests per use case
- system specs solo flussi critici
- factory leggibili
- evitare callback opache nei test

## Anti-pattern

- controller pieni di logica
- model ActiveRecord giganteschi
- callback che inviano email/API
- service object chiamati `SomethingManager`
- N+1 ignorati
- job non idempotenti
- monkey patch globali
