//! Backup-first transactional migration for the frozen Boundline 0.82.0 bridge.
//!
//! The bridge is intentionally internal to Boundline's state layer. It
//! validates an exact historical state identity, preserves the source before
//! staging, and leaves a durable journal at every recovery boundary.

mod engine;
mod model;

pub use engine::{
    apply_migration, apply_migration_with_control, inspect_legacy_state, plan_migration,
    recover_migration,
};
pub use model::{
    CanonicalVerificationResult, ConversionReport, LegacyStateItem, MigrationAction,
    MigrationBoundary, MigrationControl, MigrationError, MigrationInspection, MigrationOutcome,
    MigrationPlan, MigrationReasonCode, MigrationSource, MigrationStatus, SemanticLossRecord,
    UnsupportedStateRecord,
};
