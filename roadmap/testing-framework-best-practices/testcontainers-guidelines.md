# Testcontainers Guidelines

## Principi

Testcontainers permette integration test realistici con database, broker e servizi esterni containerizzati. Va usato quando il comportamento reale conta più della velocità assoluta.

## Quando usarlo

Buoni candidati:

- repository SQL
- migration
- transaction behavior
- message broker
- cache
- object storage locale
- servizi con protocollo complesso

Cattivi candidati:

- funzioni pure
- validazione semplice
- test che devono essere millisecond-level

## Container lifecycle

Bilanciare isolamento e velocità.

- container per suite se dati isolati bene
- container per test se isolamento assoluto richiesto
- cleanup database tra test
- schema migration controllata

## Readiness

Aspettare readiness reale, non sleep.

```java
waitingFor(Wait.forListeningPort())
```

## Dati

Ogni test deve creare i propri dati.

Non dipendere da dati pre-caricati non ovvi.

## CI

Assicurarsi che CI supporti Docker/container runtime.

Gestire:

- timeout
- pull image
- cache immagini
- parallelismo
- porte dinamiche

## Versioni

Usare versioni immagini esplicite.

### Da evitare

```text
postgres:latest
```

### Preferibile

```text
postgres:16
```

## Anti-pattern

- Testcontainers per test unitari
- `latest`
- sleep per readiness
- dati condivisi tra test
- container avviato inutilmente per ogni test banale
- integration test mescolati alla suite unit senza tag
