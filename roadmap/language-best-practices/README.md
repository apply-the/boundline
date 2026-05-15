# Language Best Practices

Questa raccolta contiene linee guida pratiche per scrivere codice più manutenibile, testabile e sicuro in più linguaggi.

L'obiettivo non è imporre regole dogmatiche, ma creare un baseline comune per team che lavorano su servizi, librerie condivise, tooling e applicazioni backend/frontend.

## Principi generali

1. Preferire codice esplicito a codice “magico”.
2. Rendere impossibili gli stati non validi quando il linguaggio lo permette.
3. Inizializzare le dipendenze dall’esterno, non crearle nascoste dentro la business logic.
4. Gestire gli errori in modo strutturato, evitando crash non intenzionali.
5. Separare I/O, side effect e logica pura.
6. Scrivere codice testabile senza dover ricorrere a hack, reflection o monkey patching.
7. Usare tipi semantici invece di primitive generiche quando il dominio lo richiede.
8. Preferire scope ownership, RAII o cleanup deterministico dove disponibile.
9. Rendere logging, tracing e correlation ID parte del design, non un dettaglio finale.
10. Evitare ottimizzazioni premature, ma non ignorare complessità algoritmica e allocazioni inutili.

## Documenti inclusi

- `general-engineering-guidelines.md`
- `rust-guidelines.md`
- `go-guidelines.md`
- `java-guidelines.md`
- `typescript-guidelines.md`
- `python-guidelines.md`
- `csharp-guidelines.md`
- `c-guidelines.md`
- `cpp-guidelines.md`
- `zig-guidelines.md`
- `kotlin-guidelines.md`
- `scala-guidelines.md`
- `php-guidelines.md`
- `ruby-guidelines.md`
- `shell-guidelines.md`
- `powershell-guidelines.md`
- `groovy-guidelines.md`

## Convenzioni

Ogni documento segue questa struttura:

- Principi
- Organizzazione del codice
- Gestione errori
- Tipi e modellazione del dominio
- Testabilità
- Concurrency e risorse
- Logging e osservabilità
- Cose da evitare


## Principi comuni

- Rendere esplicite ownership, responsabilità e lifetime delle risorse.
- Evitare global state e inizializzazione nascosta.
- Modellare il dominio con tipi o value object quando il linguaggio lo permette.
- Gestire errori attesi come valori o eccezioni specifiche, non come crash generici.
- Separare logica pura da I/O, ambiente, filesystem, rete e clock.
- Rendere il codice testabile senza patch invasive.
- Usare logging strutturato quando disponibile.
- Non nascondere errori con fallback silenziosi.

