# Framework Security Guidelines

## Principi

La sicurezza deve essere una proprietà del design, non un filtro finale. I framework aiutano, ma non proteggono da scelte sbagliate.

## Input non fidato

Considerare non fidato tutto ciò che arriva da:

- HTTP request
- query string
- header
- cookie
- webhook
- file upload
- code messaggi
- database legacy
- API esterne

## Authentication

Regole:

- centralizzare la verifica auth
- non fidarsi di campi utente provenienti dal client
- validare token, issuer, audience, expiration
- gestire refresh token con attenzione
- non loggare token

## Authorization

Autenticato non significa autorizzato.

Controllare permessi a livello di caso d’uso, non solo nel frontend.

## CSRF e CORS

Se si usano cookie per auth, proteggere da CSRF.

CORS non è un meccanismo di autenticazione. Configurarlo in modo restrittivo.

## XSS

Frontend:

- evitare HTML raw
- sanitizzare contenuto utente
- usare escaping del framework
- attenzione a markdown rendering
- Content Security Policy dove possibile

## SQL injection

Usare query parametrizzate, ORM o query builder sicuri. Non concatenare input utente.

## File upload

Validare:

- dimensione
- tipo effettivo
- estensione
- nome file
- path traversal
- antivirus/scanning se serve
- storage isolato

## Secrets

- mai hardcoded
- mai in repository
- mai nei log
- usare secret manager
- ruotare
- separare per ambiente

## Errori

Non esporre stack trace o dettagli infrastrutturali al client.

## Dependency security

- aggiornare dipendenze
- usare lockfile
- scanner SCA
- rimuovere pacchetti inutilizzati
- evitare pacchetti poco mantenuti per funzioni banali

## Anti-pattern

- autorizzazione solo frontend
- CORS wildcard con credenziali
- token in localStorage senza valutare rischio XSS
- raw HTML non sanitizzato
- stack trace esposte
- secret nei log
- endpoint admin senza audit
