# Data And AI Systems Guidance

## Purpose

This guidance defines practices for data pipelines, analytics systems, machine learning systems, LLM integrations, retrieval systems, and AI-assisted product features.

It applies to:

- ETL/ELT pipelines
- feature pipelines
- analytics jobs
- model inference services
- LLM tool integrations
- RAG systems
- evaluation pipelines
- prompt/model boundaries
- data quality workflows

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, data governance policy, AI governance policy, or Canon-governed standard.

## Core Thesis

AI and data systems fail quietly.

They often produce plausible output while drifting, leaking, duplicating, or corrupting meaning.

Engineering guidance must cover:

- data provenance
- schema contracts
- quality checks
- reproducibility
- evaluation validity
- privacy
- model output validation
- prompt and tool boundaries
- drift and monitoring
- human review where needed

## Data Provenance

Data artifacts should answer:

- where did this data come from?
- when was it produced?
- by which job/version?
- from which source schema?
- with what transformations?
- what quality checks passed?
- who owns it?

Guardians should flag datasets, features, or derived tables without provenance.

## Schema And Contract Ownership

Data schemas are contracts.

Check:

- backward compatibility
- nullable vs required fields
- semantic meaning changes
- type changes
- partition changes
- event schema changes
- downstream consumers
- data ownership

A pipeline that passes tests but silently changes meaning is unsafe.

## Data Quality

Data quality checks should cover:

- completeness
- uniqueness
- freshness
- validity
- distribution shifts
- referential integrity
- duplicate rates
- null rates
- range checks
- business invariants

Quality checks must be tied to operational decisions.

## Reproducibility

For important data/AI outputs, preserve:

- code version
- config version
- model version
- prompt version where applicable
- dataset version
- feature version
- evaluation version
- runtime environment

AI-generated notebooks or scripts without reproducibility are weak evidence.

## Evaluation Validity

AI/ML systems require meaningful evaluation.

Check:

- train/test leakage
- benchmark representativeness
- golden set ownership
- human evaluation criteria
- regression tests
- failure case tracking
- adversarial examples where relevant
- metric gaming risk

Do not treat aggregate accuracy as full system confidence.

## LLM Output Validation

LLM output is untrusted input.

Validate:

- JSON/schema shape
- tool arguments
- permissions
- policy constraints
- domain invariants
- citations/evidence where required
- hallucination-sensitive claims
- unsafe actions

Never let LLM output directly mutate critical state without validation and authorization.

## Prompt And Tool Boundaries

Prompt/tool integrations should define:

- allowed tools
- forbidden tools
- input schemas
- output schemas
- approval requirements
- side-effect boundaries
- audit logging
- failure handling

Guardians should flag broad tool access without side-effect control.

## Retrieval Systems

RAG/retrieval systems require:

- source provenance
- freshness
- chunking strategy
- retrieval evaluation
- stale document handling
- authority ranking
- citation/evidence linking
- privacy boundary
- index update strategy

Vector similarity is not authority.

## Drift And Monitoring

Monitor:

- data drift
- concept drift
- prompt drift
- model version drift
- retrieval quality drift
- tool failure rate
- human override rate
- hallucination reports
- evaluation regression

## Privacy And Sensitive Data

Check:

- PII handling
- data minimization
- retention
- access controls
- embedding leakage
- external provider transmission
- logging of sensitive prompts/responses
- training data policy

Remote embeddings or model calls must be opt-in for sensitive repositories.

## AI-Assisted Delivery Risks

AI-generated data/AI code often:

- lacks evaluation
- trusts model output
- ignores schema evolution
- creates unowned derived data
- leaks PII into logs or embeddings
- hardcodes prompts without versioning
- adds retrieval without source authority
- creates metrics that do not map to product risk

## Anti-Patterns

- dataset without provenance
- pipeline without freshness check
- model output used without validation
- prompt change without evaluation
- vector search treated as truth
- embedding sensitive data without policy
- train/test leakage
- benchmark not representative of production
- tool access broader than task needs
- generated notebook as production logic
- no drift monitoring for critical model behavior

## Guardian Hooks

Recommended guardians:

- data-provenance-guardian
- data-quality-guardian
- schema-contract-guardian
- ai-evaluation-guardian
- llm-output-validation-guardian
- prompt-versioning-guardian
- retrieval-authority-guardian
- embedding-privacy-guardian
- tool-boundary-guardian
- drift-monitoring-guardian

## Structured Finding Example

```json
{
  "guardian": "llm-output-validation",
  "rule": "model-output-mutates-state-without-validation",
  "disposition": "blocker",
  "summary": "The LLM-generated tool arguments are passed directly to an account update operation without schema validation or authorization re-check.",
  "evidence_refs": ["src/agents/account_tools.ts"],
  "recommended_action": "Validate tool arguments against a schema and re-check authorization before mutation."
}
```

## Lifecycle Usage

Planning:
- identify data ownership, evaluation needs, and privacy boundaries

Architecture:
- define provenance, schema contracts, retrieval authority, and tool boundaries

Implementation:
- validate model outputs, preserve versions, and add quality checks

Testing:
- verify evaluation, regression, and data quality gates

Review:
- challenge leakage, hallucination-sensitive paths, and unvalidated model outputs

Verification:
- compare AI/system claims to evidence and evaluation results
