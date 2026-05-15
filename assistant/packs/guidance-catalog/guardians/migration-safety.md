# Migration Safety Guardian

Enforce safe database and schema migration practices that prevent data loss and downtime.

## Rules

### destructive-migration-without-rollback
Dropping columns, tables, or changing column types without a tested rollback path risks irreversible data loss. Use expand/contract pattern for breaking schema changes.

Triggers: `DROP COLUMN`, `DROP TABLE`, `ALTER COLUMN` type changes in a single migration without corresponding reverse migration, destructive migrations without feature-flag gating.

### schema-lock-duration
Migrations that acquire exclusive locks on large tables can block all reads and writes for extended periods. Use online DDL techniques or batched operations.

Triggers: `ALTER TABLE` on tables with millions of rows without `CONCURRENTLY` (PostgreSQL) or equivalent, index creation without online flag, column additions with non-null defaults on large tables (pre-PG11).

### missing-backfill-validation
Data migrations must validate results after execution. Missing validation means corrupted or incomplete backfills go undetected until a consumer fails.

Triggers: backfill scripts without row-count verification, data transformations without before/after checksums, migrations that modify data without logging affected row counts.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages and databases. Cross-cutting; relevant when changes include schema migrations, data transformations, or storage contract changes.
