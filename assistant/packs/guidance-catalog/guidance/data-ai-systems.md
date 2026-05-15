# Data And AI Systems

Conventions for data pipelines, machine learning systems, and AI-integrated applications. These systems have unique challenges around reproducibility, data quality, and model governance.

## Data Quality

Validate data at ingestion boundaries. Define and enforce schemas. Monitor for drift, missing values, and anomalies. Treat data quality as a first-class concern.

## Pipeline Design

Make pipelines idempotent and rerunnable. Use explicit dependencies between stages. Separate extraction, transformation, and loading concerns. Version pipeline definitions.

## Reproducibility

Pin library versions. Track data lineage. Version datasets alongside code. Record experiment parameters and random seeds. Make results reproducible from inputs.

## Model Governance

Track model versions, training data, and evaluation metrics. Validate models before deployment. Monitor model performance in production. Have rollback procedures for model updates.

## AI Integration

When integrating AI/LLM outputs into applications:
- Treat AI output as untrusted external input
- Validate format, content, and safety before use
- Handle failures gracefully (rate limits, timeouts, unexpected responses)
- Log prompts and responses for debugging and audit
- Version prompt templates alongside application code

## Testing

Test pipelines with representative data subsets. Test model inference with known inputs. Use contract tests for data schemas between pipeline stages.

## Anti-Patterns

- Missing schema validation at data boundaries
- Non-idempotent pipelines that cannot be safely rerun
- Unreproducible experiments (missing version pins or seeds)
- AI output used without validation or safety checks
- Missing monitoring for data drift or model degradation
- Pipeline dependencies expressed only through scheduling order
- Training data mixed with test data

## Guardian Hooks

Guardians that apply to this guidance:
- `security_boundary`: untrusted AI output, data injection risks
- `observability`: pipeline monitoring, model performance tracking
- `supply_chain`: data lineage, model versioning
