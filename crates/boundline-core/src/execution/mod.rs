//! Execution safety module for Boundline.
//!
//! Provides deterministic command intent classification, execution
//! policy enforcement (Intent × Zone matrix), structured evidence
//! capture, secret redaction, dry-run tiering, mutation boundary
//! tracking, and governance hooks.

pub mod classifier;
pub mod dry_run;
pub mod evidence;
pub mod hooks;
pub mod mutation;
pub mod policy;
pub mod redaction;
