# Boundline as Generic Agentic Framework: Integration Report

> Analysis of how Boundline can evolve from an orchestrator strettamente accoppiato a Canon a un
> **motore di orchestrazione agnostico (Agentic Framework Engine)**, capace di supportare
> framework proprietari (harness) tramite un sistema di adapter e override.
>
> **Constraints**:
> - Boundline è **open source**.
> - L'integrazione di framework specifici o proprietari avviene tramite **binary adapters esterni** (repository separati).
> - È presente un template di riferimento locale in `/Users/rt/workspace/apply-the/boundline-framework-template`.
> - **Niente dipendenza da MCP come strato architetturale core**: l'astrazione sulle capabilities usa il nostro Provider Protocol nativo.

---

## 1. Visione Architetturale: Canon come Default, Adapters come Override

Boundline non deve perdere il suo valore "out-of-the-box". 

**La regola d'oro:**
Boundline viene distribuito sempre con **Canon come default**. Se nessun adapter è configurato, le fasi del ciclo di vita (`goal`, `plan`, `run`, `review`) vengono processate dalla logica nativa legata a Canon.

**L'astrazione ad Override Parziale:**
Un adapter esterno (es. un binario Rust compilato custom) non deve necessariamente riscrivere l'intero universo. Può "registrasi" per fare l'override di un singolo step.
Ad esempio: un adapter aziendale potrebbe dichiarare: *"Uso Canon per `goal` e `plan`, ma intercetto la fase di `run` per applicare i miei hook distruttivi e le mie policy".*

---

## 2. Il Sistema di Injection e Registrazione

Come fa un binario Rust esterno a farsi riconoscere da Boundline e a farsi passare la configurazione?

### A. Discovery e Registrazione
Boundline adotterà un modello ispirato ai plugin di Git o Terraform:
1. **Config-based**: Nel `.boundline/config.toml` l'operatore dichiara l'adapter:
   ```toml
   [framework.adapter]
   command = "boundline-harness-gridspertise" # o path assoluto
   ```
2. **Naming Convention (Opzionale)**: Boundline può cercare automaticamente nel PATH binari che iniziano per `boundline-plugin-*`.

### B. L'Handshake (Capabilities & Config Injection)
Quando Boundline fa boot di una sessione, invoca il binario adapter con un comando di handshake (es. tramite stdin JSON-RPC inviando `{"method": "capabilities"}`).

L'adapter risponde con il suo "manifesto":
```json
{
  "name": "system-harness-template",
  "overrides": ["plan", "run"],          // Dichiara quali stage vuole intercettare
  "hooks": ["on_error", "on_step_pre"],  // Quali hook globali vuole ascoltare
  "config_schema": {                     // Chiede a Boundline di fornirgli delle configurazioni
    "harness_repo": "string",
    "strict_mode": "boolean"
  }
}
```

### C. Autoconfigurazione
Sulla base del `config_schema` restituito, Boundline si occupa di:
- Verificare se il `.boundline/config.toml` contiene già quei campi.
- In caso contrario, durante `boundline init` o all'avvio, chiedere interattivamente all'utente i valori mancanti o scriverli con dei default.
- Passare l'intero blocco di configurazione popolato ad ogni successiva invocazione dell'adapter.

---

## 3. Il Protocollo JSON su Stdin/Stdout

La comunicazione non avviene linkando librerie dinamiche (troppo fragile, problemi di ABI), ma tramite **Subprocess Protocol (stdin/stdout JSON)**, esattamente lo stesso approccio di design robusto che si usa tra LSP (Language Servers) e IDE.

**Richiesta di Boundline all'Adapter (Override della fase `plan`):**
```json
{
  "method": "execute_stage",
  "params": {
    "stage": "plan",
    "session_id": "abc-123",
    "workspace_ref": "/path/to/workspace",
    "adapter_config": {
      "harness_repo": "https://github.com/org/repo",
      "strict_mode": true
    },
    "context": { ... } // Informazioni di stato raccolte finora da Canon o da Boundline
  }
}
```

**Risposta dell'Adapter:**
```json
{
  "result": {
    "status": "success",
    "artifacts_produced": ["/path/to/plan.md"],
    "phase_request": null // se l'adapter avesse bisogno di chiedere all'utente, lo restituirebbe qui
  }
}
```

---

## 4. Architettura dei Repository (Il Modello a 3 Repo)

Questo design conferma l'utilità del template che hai creato localmente:

1. **`boundline` (Open Source)**: 
   Contiene l'orchestratore, l'engine JSON-RPC, l'implementazione **di default** di Canon. Nessuna logica proprietaria di framework terzi.
2. **`boundline-framework-template` (Open Source Template)**: 
   Lo scaffolding (il repo che hai appena creato). Contiene un server JSON-RPC pronto all'uso, i tipi Rust corretti e i metodi vuoti (`fn execute_stage()`, `fn on_error()`). Chiunque voglia crearsi un agentic framework aziendale custom fa un fork di questo repo.
3. **`my-company-harness-adapter` (Proprietario / Custom)**: 
   Il binario finale compilato dal cliente a partire dal template. Conterrà le regole custom, la lettura di `.github/hooks/`, o l'integrazione con pipeline chiuse aziendali.

---

## 5. Mappatura tra Harness Proprietario e Boundline

Un framework adapter può coprire facilmente le logiche di un *system-harness-template* aziendale mappando le funzionalità di Boundline:

| Necessità Framework Esterno | Soluzione via Adapter in Boundline |
|---|---|
| Fasi Custom del ciclo di vita | L'adapter dichiara `overrides: ["goal", "plan", "run"]` e inietta la propria logica. |
| Audit Log personalizzati | L'adapter si registra agli hook `on_step_post` e `on_session_end` e scrive i propri log. |
| Sensori / Qualità / Linting | L'adapter mappa i propri script distruttivi dentro le risposte di `evaluate_gate` o `on_step_pre`. |
| Gestione Errori (Triage) | L'adapter si registra a `on_error`, legge la telemetria e decide se riprovare, bloccare o correggere. |
| Integrazioni Piattaforma (Jira/CI) | **Niente MCP**. L'adapter usa l'External Capability Provider Protocol nativo di Boundline o esegue binari/script diretti. |

---

## 6. Prossimi Passi (Action Items per implementare questa Spec)

Per concretizzare questa visione servirà:
1. **Definire il trait in Boundline**: Isolare la logica attuale in un trait `FrameworkAdapter` (che ha come implementazione concreta e di default il codice esistente legato a Canon).
2. **Sviluppare il Subprocess Host**: Il modulo che spawna il binario indicato nella configurazione e orchestra il json-rpc.
3. **Implementare l'Handshake**: Aggiungere in `boundline init` la logica di discovery `capabilities()` e la generazione del file di configurazione automatico.
4. **Allineare il Template Locale**: Adattare `/Users/rt/workspace/apply-the/boundline-framework-template` per consumare correttamente questi contratti JSON.

Questo design garantisce che Boundline rimanga **completamente riutilizzabile**, agnostico se lo si desidera, mantenendo la UX impeccabile e sicura (Canon) qualora l'utente non configuri nulla.
