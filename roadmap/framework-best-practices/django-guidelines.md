# Django Guidelines

## Principi

Django è produttivo, ma tende a spingere logica in model, view, signal e admin. Serve disciplina per mantenere boundary chiari e testabili.

## App coese

Ogni app deve rappresentare una capacità di dominio, non una cartella tecnica generica.

### Preferire

```text
orders/
payments/
customers/
```

### Evitare

```text
utils/
helpers/
core/
```

come contenitori generici senza confini.

## Views sottili

View o DRF ViewSet devono gestire HTTP e delegare casi d’uso.

```python
class CreateOrderView(APIView):
    def post(self, request):
        serializer = CreateOrderSerializer(data=request.data)
        serializer.is_valid(raise_exception=True)

        result = create_order(serializer.to_command())
        return Response(to_response(result), status=201)
```

## Serializer non è dominio

DRF serializer valida input/output API. Non deve diventare il luogo principale della business logic.

Separare:

- serializer
- command
- service/use case
- model/domain logic

## Models

I model Django possono contenere invariant locali e metodi utili, ma evitare model obesi con orchestrazione di tutto il sistema.

## Query

Evitare N+1.

Usare:

- `select_related`
- `prefetch_related`
- annotation
- projection
- pagination

## Transactions

Usare `transaction.atomic` nei casi d’uso che richiedono atomicità.

Evitare chiamate remote dentro transazioni DB.

## Signals

Usare signals con grande cautela. Sono side effect nascosti.

Preferire eventi applicativi espliciti o chiamate dirette quando il flusso deve essere chiaro.

## Settings

Separare settings per ambiente. Non leggere environment ovunque. Validare config critica.

## Testing

- unit test su service/domain senza DB quando possibile
- integration test su ORM/query
- API tests su endpoint critici
- factory per dati test
- evitare fixture enormi opache

## Security

- CSRF attivo dove serve
- permission class corrette
- serializer output controllato
- non esporre campi sensibili
- query parametrizzate/ORM
- file upload validato

## Anti-pattern

- business logic nei serializer
- signals usati per flussi critici
- model enormi
- viewset con troppa logica
- N+1 ignorati
- settings magic non validati
- `raw SQL` non parametrizzato
