# Domain Language Guidance

## Purpose

This guidance defines how Boundline should use and protect a product area's ubiquitous language during AI-assisted delivery.

It applies to discovery, requirements, domain-language, domain-model, architecture, backlog, implementation, review, refactor, and verification work.

## Authority Classification

Default strength: recommended  
Canon-governed domain-language packets may make specific terms mandatory or deprecated for a project area.

## Core Thesis

A system's language is part of its architecture.

When AI introduces plausible but inconsistent terminology, the codebase and product model drift.

Domain language guidance exists to protect:

- shared meaning
- product concepts
- user-visible vocabulary
- domain invariants
- capability boundaries
- architecture decisions

## Ubiquitous Language

Ubiquitous language is the shared vocabulary used by product, domain experts, engineers, tests, APIs, and documentation.

A term should define:

- name
- definition
- source
- owner
- accepted synonyms
- forbidden synonyms
- related concepts
- examples
- non-examples
- lifecycle status

## Term Status

Recommended statuses:

```text
proposed
accepted
deprecated
forbidden
ambiguous
external-only
implementation-only
```

## Term Drift

Term drift happens when the same concept receives multiple names or when one term is used for different meanings.

Examples:

- `customer`, `account`, `user`, `member` used interchangeably
- `order` used for cart, purchase, shipment, and invoice
- `workspace` and `tenant` confused
- API field names diverge from domain language
- tests use obsolete vocabulary

Guardians should flag drift when it affects meaning or discoverability.

## Accepted Synonyms

Some synonyms are legitimate.

Example:

```text
Customer = public product term
AccountHolder = legal/domain term
User = authentication actor
```

The domain language must explain the distinction.

## Implementation Terms vs Domain Terms

Implementation terms should not replace domain language.

Examples of implementation terms:

- DTO
- Entity
- Handler
- Processor
- Manager
- Record
- Row

These may exist in code, but they should not become product/domain concepts unless justified.

## External Terms

External provider vocabulary may differ from internal domain language.

Use anti-corruption mapping when needed.

Example:

```text
Provider "subscriber" maps to internal "account member"
```

Do not allow external integration terms to silently rewrite the internal model.

## Naming In Code

Domain terms should appear in:

- domain types
- service names
- API contracts
- tests
- events
- documentation
- error messages where user-facing

Avoid:

- generic names
- implementation-only names
- inconsistent abbreviations
- new terms introduced by a generated diff without definition

## Domain Language And Tests

Tests are a strong signal for language drift.

Test names should express behavior using domain language.

Bad:

```text
test_process_valid_data
```

Better:

```text
rotates_refresh_token_when_session_is_active
```

## AI-Assisted Delivery Risks

AI often invents:

- plausible synonyms
- generic entity names
- framework-shaped vocabulary
- provider terminology
- inconsistent event names
- shallow test names

Guardians should challenge new terms and unclear naming.

## Anti-Patterns

- new term without definition
- two names for same concept
- one name for multiple concepts
- external provider term leaking into domain core
- implementation term used as domain term
- generic manager/handler/processor names in domain layer
- deprecated term reintroduced
- tests using stale vocabulary
- API contract field diverges from governed domain term

## Guardian Hooks

Recommended guardians:

- domain-language-drift-guardian
- new-term-without-definition-guardian
- deprecated-term-guardian
- external-term-leakage-guardian
- implementation-term-guardian
- test-language-guardian
- api-vocabulary-guardian

## Structured Finding Example

```json
{
  "guardian": "domain-language-drift",
  "rule": "new-term-without-definition",
  "disposition": "concern",
  "summary": "The change introduces `memberAccount` while the Canon domain language defines `accountHolder` for this concept.",
  "evidence_refs": ["src/accounts/member_account.ts", "docs/project/domain-language.md"],
  "recommended_action": "Use the governed term or update the domain-language packet before introducing a new concept."
}
```

## Lifecycle Usage

Discovery:
- collect candidate terms and ambiguity

Requirements:
- align terms before scoping behavior

Domain-language:
- stabilize vocabulary explicitly

Domain-model:
- connect terms to concepts and invariants

Architecture:
- protect language across boundaries

Implementation:
- guide names in code, tests, APIs, and events

Review:
- flag drift, deprecated terms, and generic naming

Refactor:
- rename toward governed language without expanding behavior
