# Contract Testing Guidelines

## Principi

I contract test verificano che consumer e provider concordino su richieste e risposte. Sono particolarmente utili quando i servizi vengono rilasciati indipendentemente.

## Quando usarli

Usare contract testing se:

- API usata da più consumer
- team diversi
- deploy indipendenti
- E2E costoso o fragile
- breaking change frequenti

## Consumer-driven contracts

Il consumer descrive aspettative minime.

Esempio:

- endpoint
- metodo
- headers
- request body
- response status
- response body shape

## Provider verification

Il provider verifica di rispettare i contratti dei consumer.

## Cosa mettere nel contratto

- campi necessari al consumer
- tipi
- status code
- headers rilevanti
- casi errore importanti

## Cosa evitare

- dettagli non usati dal consumer
- payload completi con campi irrilevanti
- contratti troppo rigidi su ordinamento non garantito
- testare business logic completa via contract

## Pact

Pact è una scelta comune per consumer-driven contract testing.

Regole:

- contratti versionati
- broker o storage condiviso
- verifica in CI provider
- pubblicazione contratti in CI consumer
- backwards compatibility esplicita

## Anti-pattern

- contract test usati come E2E
- contratti enormi
- provider che ignora verifica
- consumer che non pubblica contratti aggiornati
- contratti derivati manualmente e non dai test
