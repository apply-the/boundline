# Synod Roadmap

Canon resta fuori dal perimetro di questa roadmap: e il runtime che persiste artifact strutturati. Synod e il sistema che pensa, decide, orchestra ed esegue.

## Obiettivo

Fare evolvere Synod in un sistema capace di prendere un problema e trasformarlo in codice funzionante, con controllo qualita multi-agente.

## Spec 1 — Delivery Orchestrator Core

### Outcome

Synod introduce un orchestratore centrale capace di eseguire task complessi come sequenze governate di step, non piu come singole invocazioni LLM.

### Perche ora

Senza orchestrator non puoi avere:

- flow end-to-end
- iterazioni
- agent coordination

### In scope

- orchestrator loop: plan -> execute -> evaluate -> continue/stop
- stato di sessione
- step execution model
- routing tra agenti
- gestione errori e retry

### Fuori scope

- multi-agent voting avanzato
- strategie avanzate
- provider complexity

### Risultato tangibile

Synod passa da LLM wrapper a engine che porta a termine task multi-step.

## Spec 2 — Delivery Flows (SDLC Backbone)

### Outcome

Synod supporta flow espliciti per la delivery:

- requirements -> architecture -> backlog -> implementation
- change -> implementation
- bug -> investigate -> fix

### Perche ora

E il cuore del valore: portare codice in produzione, non solo analizzare.

### In scope

- definizione di flow predefiniti
- step sequencing
- handoff tra step
- checkpoint e approval hooks
- stato condiviso tra step

### Fuori scope

- adattivita avanzata
- multi-strategy selection
- voting

### Risultato tangibile

Un comando Synod puo eseguire un intero percorso di delivery, non solo uno step.

## Spec 3 — Execution Engine (Code Delivery)

### Outcome

Synod esegue davvero lavoro di sviluppo:

- scrive codice
- modifica file
- esegue test
- valida output

### Perche ora

Senza execution reale, non deliveri.

### In scope

- workspace interaction:
- read/write file
- diff generation
- test execution hooks
- validation loop: generate -> run -> fix -> retry
- gestione errori runtime

### Fuori scope

- full CI/CD
- deploy
- governance profonda, che resta in Canon

### Risultato tangibile

Synod puo prendere una slice e produrre codice funzionante, non solo suggerimenti.

## Spec 4 — Multi-Agent Review & Voting

### Outcome

Synod introduce councils multi-agente per validare output prima di considerarlo done.

### Perche ora

Quando inizi a generare codice in automatico, ti serve controllo qualita serio.

### In scope

- reviewer multipli
- provider diversi: GPT, Claude, Gemini e altri
- structured findings
- voting: majority e weighted
- adjudication base
- trigger su:
- high risk
- failing validation
- PR generation

### Fuori scope

- governance artifact, che resta in Canon
- full debate simulation

### Risultato tangibile

Synod non si fida di un singolo modello e produce output piu robusto.

## Sequenza Consigliata

1. Orchestrator Core
2. Delivery Flows
3. Execution Engine
4. Review & Voting

## Architettura Risultante

```text
User / Copilot / Claude
        ↓
      Synod
  ┌───────────────┐
  │ Orchestrator  │
  │ Flows         │
  │ Agents        │
  │ Execution     │
  │ Review        │
  └───────────────┘
        ↓
     Canon
 (artifact + governance)
```

## In Una Frase

Synod deve diventare un sistema che prende un problema e lo trasforma in codice funzionante, con controllo qualita multi-agente.