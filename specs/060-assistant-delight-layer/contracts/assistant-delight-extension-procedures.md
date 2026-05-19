# S7 Contract Amendment Procedures

**Version**: 0.5.0  
**Effective Date**: 2026-05-17  
**Purpose**: Formal procedures for adding, modifying, or retiring S7 contract elements without causing divergence between Boundline and Canon

---

## Overview

The S7 contract is a living document that evolves as new delight capabilities are added. Amendments must follow a bilateral procedure to ensure both Boundline and Canon teams maintain alignment and accountability.

---

## Amendment Types

### Type A: Adding New Canon Input Class

**What**: S7 needs to consume a new category of Canon-governed artifact (e.g., performance benchmarks, compliance certificates, deployment readiness signals).

**Example**: "S7 should cite performance benchmarks when available to explain why a stage is ready/not ready to proceed."

**Process**:

1. **Proposal Phase** (Boundline initiates)
   - Open GitHub issue in Boundline repo: `S7 Contract Amendment: Add [InputClass]`
   - Title: Descriptive and specific
   - Description must include:
     - What the new input class represents
     - When/how often it becomes available
     - What metadata must accompany it (promoted_at, promoted_by, version, etc.)
     - Which S7 vocabulary terms would use it (Risk, Assumption, Blocker, etc.)
     - Max age for freshness validation (or null if evergreen)
     - Example use case in an S7 explanation
     - Estimated implementation effort in Boundline and Canon
     - Compatibility with existing contracts or version bump required

2. **Cross-Repo Proposal** (Simultaneous, Boundline + Canon)
   - Create matching amendment proposal in Canon repo (spec/057 or equivalent)
   - Canon documents what it commits to providing:
     - Data schema for this artifact class
     - Promotion criteria and validation rules
     - How Canon ensures quality/accuracy
     - How Canon signals compatibility version
   - Boundline documents what it commits to consuming:
     - Source attribution rules
     - Degradation handling if artifact is missing/stale/incompatible
     - Where in explanations this artifact will be cited

3. **Bilateral Review** (Staggered review, NOT sequential)
   - Boundline team: Review proposal in both Boundline and Canon specs for feasibility and alignment
   - Canon team: Review proposal in both Boundline and Canon specs for data quality and commitment
   - Cross-team review meeting (sync or async): Confirm both teams understand the amendment
   - Points of approval needed:
     - Boundline tech lead: "We can implement S7 consuming this" ✓
     - Canon tech lead: "We can reliably provide this" ✓
     - No approval: amendment is blocked; proposer notified with rationale

4. **Specification Update** (Simultaneous in both repos)
   - Update Boundline `specs/060-assistant-delight-layer/contracts/assistant-delight-contract.md` Section III
   - Update Canon `specs/057-s7-delight-provider/contracts/...` (provider side)
   - Both update: `assistant-delight-input-classes.schema.json`
   - Both update: Data model documentation
   - Both bump contract version: 0.5.0 → 0.6.0 (or appropriate semver)
   - Update CHANGELOG.md in both repos with amendment details

5. **Implementation Sequencing**
   - If amendment is simple (non-breaking): Can ship in same release as amendment approval
   - If amendment is complex (schema change, validation addition): Staged release plan created
   - Compatibility window defined if older contract versions must remain supported

6. **Deployment** (Synchronized or with compatibility layer)
   - Both Boundline and Canon deploy updated contract version
   - S7 runtime validates consumption against current contract_version field
   - If versions mismatch, degradation signal informs operator

**Success Criteria**:
- ✓ Amendment merged to both repos in same batch (within 2 weeks of each other)
- ✓ Both teams have approved and documented their side
- ✓ CHANGELOG.md entries are clear and linked
- ✓ No silent contract expansion

---

### Type B: Modifying Existing Canon Input Class

**What**: Changing metadata requirements, max age, validation rules, or vocabulary alignment for an already-contracted input.

**Example**: "Increase max age for security findings from 14 days to 30 days due to operational constraints."

**Process**:

1. **Proposal Phase**
   - Open issue: `S7 Contract Amendment: Modify [InputClass]`
   - Clearly state what is changing and why
   - Impact analysis:
     - Will this make existing S7 explanations more or less confident?
     - Are there any compatibility concerns (e.g., older artifacts that suddenly become usable)?
     - Do validation rules need updating?

2. **Bilateral Discussion**
   - Why is this change needed?
   - Can the requesting team solve the problem differently (e.g., instead of increasing max age, more frequent updates)?
   - What are the implications for both teams?

3. **Specification Update**
   - Modify the relevant entry in `assistant-delight-input-classes.schema.json`
   - Update contract narrative explaining the change
   - Bump contract version (minor version if compatible, major if breaking)
   - Document in CHANGELOG.md with date and rationale

4. **Compatibility Handling**
   - If change is backward-compatible: Can deploy without versioning concerns
   - If change is breaking: Create deprecation window (2 releases) where old rules are still honored but flagged

5. **Deployment**
   - Both teams update their validation logic
   - S7 runtime compares contract_version on incoming artifacts
   - If mismatch → degradation signal (but continue if possible)

**Success Criteria**:
- ✓ Change is documented with clear rationale
- ✓ Both teams agree on compatibility implications
- ✓ Validation logic updated in both repos
- ✓ No silent semantic changes

---

### Type C: Adding New S7 Vocabulary Term

**What**: Introducing new explanation vocabulary beyond the current Risk, Assumption, Blocker, Confidence, Next Action, Missing Evidence.

**Example**: "Add 'Blocker Status' term to report whether blockers are critical, manageable, or resolved."

**Process**:

1. **Proposal Phase**
   - Provide: Definition, use cases, examples, how it relates to existing terms
   - Question: Why can't existing terms cover this? Why is new term necessary?
   - Provide: Rendering examples (CLI, chat, IDE)

2. **Review & Debate**
   - Vocab review board (maintainers from both teams) evaluates:
     - Does it reduce operator confusion or increase it?
     - Does it align with S7's core purpose (source-attributed explanations)?
     - Will it require schema changes to data model?

3. **Specification Update**
   - Update `assistant-delight-explanation-vocabulary.md` with new term definition
   - Update data model to include new vocabulary choice
   - Update example answers to show new term in context
   - Document how new term interacts with existing terms

4. **Deprecation of Old Term** (if applicable)
   - If new term replaces old term, start deprecation window
   - Old term still rendered for 2 releases with "deprecated" flag
   - Operator migration guidance provided

**Success Criteria**:
- ✓ New term does not create ambiguity with existing terms
- ✓ Rendering examples are clear across all surfaces
- ✓ Data model is updated consistently

---

### Type D: Retiring a Canon Input Class

**What**: Removing a previously contracted input class because S7 no longer needs it, or Canon can no longer provide it reliably.

**Example**: "Retire 'audit_findings' input class; S7 can convey governance intent through packets and readiness signals."

**Process**:

1. **Proposal Phase**
   - Clear rationale: Why is this no longer needed?
   - Impact analysis: Which S7 explanations currently use this input? What's the fallback?
   - Deprecation timeline: When will it be removed?

2. **Deprecation Window** (2 releases)
   - Release N: Mark input class as deprecated; S7 still accepts it but flags in degradation signals
   - Release N+1: Final release with deprecation flag; operator guidance about fallback
   - Release N+2: Input class removed from contract

3. **Specification Update**
   - Move retired class to "retired classes" section in contract
   - Document what replaced it (if anything)
   - Provide fallback guidance for operators
   - Update data model to exclude retired class

4. **Migration Guidance**
   - Document how S7 explanations that used retired class will behave post-retirement
   - Explain fallback behavior and confidence impact
   - Example: "Audit findings no longer available; risk assessments now rely on packet data and runtime checks"

**Success Criteria**:
- ✓ 2-release deprecation window observed
- ✓ Operator migration path clear
- ✓ Fallback behavior documented
- ✓ No data loss during transition

---

### Type E: Emergency Contract Override

**What**: Urgent correction needed due to data quality issue, security concern, or critical bug.

**Scenario**: "Discovery that security findings are unreliable due to scanner misconfiguration; must temporarily halt S7 consumption."

**Process**:

1. **Immediate Alert**
   - Notify both teams immediately (Slack, email, escalation)
   - Describe the problem and proposed solution
   - Propose emergency override (usually: temporarily disable input class or lower confidence)

2. **Quick Review** (same day)
   - Both teams confirm the issue and proposed override
   - Override deployed with temporary flag
   - CHANGELOG.md entry: "EMERGENCY: [class] disabled due to [reason]"

3. **Formal Amendment** (within 1 week)
   - Follow normal amendment procedures to address root cause
   - Decide: Is this a temporary fix or permanent change?
   - Document findings and resolution in amendment proposal

4. **Resolution**
   - Either restore class with fixes, or retire it via Type D procedure
   - Post-mortem discussion: How to prevent recurrence?

**Success Criteria**:
- ✓ Issue addressed within hours (not days)
- ✓ Formal amendment follows within week
- ✓ Root cause documented
- ✓ Prevention strategy for future

---

## Amendment Workflow Diagram

```
Proposal
   ↓
[Proposer writes spec]
   ↓
Boundline Review ← → Canon Review  (parallel)
   ↓                    ↓
Boundline Approval + Canon Approval (both required)
   ↓
Specification Update (both repos)
   ↓
Contract Version Bump
   ↓
CHANGELOG.md entries (both repos)
   ↓
Implementation (per feature tasks)
   ↓
Deployment Coordination
   ↓
Post-Deploy Validation
   ↓
Complete
```

---

## Amendment Checklist

Before merging any amendment:

- [ ] **Proposal clearly describes what is changing and why**
- [ ] **Impact analysis includes implications for both teams**
- [ ] **Boundline approver has signed off: "We can implement/consume this"**
- [ ] **Canon approver has signed off: "We can reliably provide this"**
- [ ] **Both specs are updated with identical contract versions**
- [ ] **Data model and schemas updated consistently**
- [ ] **assistant-delight-explanation-vocabulary.md and assistant-delight-input-classes.schema.json updated**
- [ ] **CHANGELOG.md entries in both repos with amendment date and summary**
- [ ] **Degradation handling rules updated if new input classes added**
- [ ] **Examples and documentation updated to reflect change**
- [ ] **If breaking change: Compatibility window / migration guidance provided**
- [ ] **Cross-repo reference verified (can find related amendment in other repo)**

---

## Amendment Tracking

Every approved amendment is recorded in a `amendment-history.md` file:

```markdown
# S7 Contract Amendment History

## Amendment 001: Add "readiness_signals" input class (v0.5.0)
- **Date**: 2026-05-17
- **Proposer**: [name/team]
- **Type**: Type A (Add input class)
- **Status**: Approved and merged
- **Release**: v0.5.0
- **Related Issues**: Boundline#123, Canon#456
- **Boundline Spec Commit**: abc123
- **Canon Spec Commit**: def456

## Amendment 002: [Future amendment]
- ...
```

---

## Amendment Template

```markdown
# S7 Contract Amendment: [Title]

## Proposal
[What is changing and why?]

## Current Behavior
[How does the contract work today?]

## Proposed Change
[Exact specification of what changes]

## Impact Analysis
- Boundline impact: [How does this affect S7 runtime?]
- Canon impact: [How does this affect what Canon provides?]
- Operator impact: [How do explanations change?]
- Breaking change: [Yes/No + compatibility plan]

## Examples
[Show how S7 explanations would change]

## Validation Rules
[Any new validation logic needed?]

## Questions for Review
[Specific points reviewers should address]

## Timeline
- Proposal date: [DATE]
- Target approval: [DATE]
- Target merge: [DATE]
- Target deployment: [DATE]
```

---

## Anti-Patterns to Avoid

❌ **Silent extension**: Adding new S7 functionality that secretly consumes uncontracted Canon inputs  
→ Always formally amend the contract first

❌ **Unilateral change**: Boundline modifying contract without Canon input (or vice versa)  
→ Bilateral review required before any merge

❌ **Undocumented version bump**: Changing contract version without CHANGELOG entry  
→ Every version bump must be traceable

❌ **Deprecation surprise**: Removing features without deprecation window  
→ 2-release minimum for any retirement

❌ **Compatibility breakage**: New amendment breaks existing S7 explanations without notification  
→ Degradation signals must be explicit if compatibility changes

---

## Success Metrics for Amendments

✓ **Bilateral agreement**: Both teams approve before merge  
✓ **Specification clarity**: Amendment is unambiguous and reviewable  
✓ **Traceability**: Amendment is linked to tracking issues and commits in both repos  
✓ **No divergence**: Both teams' implementations follow the same contract  
✓ **Operator transparency**: Changes are visible in CHANGELOG and documented  

---

## Related Documents

- `/specs/060-assistant-delight-layer/contracts/assistant-delight-contract.md` — Main contract
- `/specs/060-assistant-delight-layer/contracts/assistant-delight-explanation-vocabulary.md` — Vocabulary
- `/specs/060-assistant-delight-layer/contracts/assistant-delight-input-classes.schema.json` — Input schema
- `amendment-history.md` — Record of all amendments
