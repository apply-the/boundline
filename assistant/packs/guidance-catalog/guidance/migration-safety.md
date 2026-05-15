# Migration Safety

Schema and data migrations in production systems require careful planning to avoid downtime, data loss, and backward compatibility breaks.

## Expand And Contract

Use the expand/contract pattern for breaking schema changes:
1. Expand: add new column/field, deploy code that writes both
2. Migrate: backfill existing data
3. Contract: remove old column/field after all consumers updated

Never remove or rename columns in a single deployment.

## Zero-Downtime Deployments

Ensure migrations are compatible with both old and new application versions running simultaneously during rolling deployments.

## Data Integrity

Validate data before and after migration. Use checksums or row counts. Run migrations in transactions where the database supports it. Have rollback plans.

## Testing Migrations

Test migrations against production-scale data copies. Verify both forward and rollback paths. Check performance impact on large tables.

## Feature Flags

Use feature flags to decouple deployment from release. New code paths can exist alongside old ones until migration is verified complete.

## Anti-Patterns

- Destructive migrations without rollback plan
- Schema changes that break running application instances
- Large data migrations without progress tracking
- Missing validation after migration
- Migrations that hold locks on large tables for extended periods
- Coupling code deployment to data migration completion
- Testing migrations only against small datasets

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: public-contract-stability (data contracts)
- `operations_readiness`: rollback capability
- `resilience`: migration failure handling
