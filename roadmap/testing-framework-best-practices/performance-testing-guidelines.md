# Performance Testing Guidelines

## Principi

Performance testing serve a scoprire limiti, regressioni e colli di bottiglia prima della produzione. Non è solo “fare tante richieste”.

## Tipi

### Load test

Verifica comportamento sotto carico atteso.

### Stress test

Trova il punto di rottura.

### Soak test

Verifica stabilità su periodo lungo.

### Spike test

Verifica picchi improvvisi.

### Capacity test

Aiuta a dimensionare risorse.

## Metriche

Misurare:

- throughput
- latency p50/p95/p99
- error rate
- CPU
- memoria
- GC
- connessioni DB
- queue depth
- saturazione thread/event loop
- timeout/retry

## Tool

Esempi:

- k6
- Gatling
- JMeter
- Locust
- Artillery

## Test realistici

Un test performance deve simulare:

- mix di endpoint realistico
- dati realistici
- think time
- autenticazione
- cache warm/cold se rilevante
- limiti esterni

## Ambiente

Idealmente ambiente simile alla produzione.

Se non possibile, documentare differenze e non vendere numeri come assoluti.

## SLO

Legare test a obiettivi.

Esempio:

- p95 sotto 300 ms per ricerca
- error rate sotto 0.1%
- throughput 500 rps sostenuti

## CI

Eseguire smoke performance leggeri in CI e test completi in pipeline dedicata.

## Anti-pattern

- testare solo homepage
- ignorare p95/p99
- usare dati irrealistici
- ambiente condiviso rumoroso
- nessuna baseline
- nessun confronto storico
- test senza osservabilità lato server
