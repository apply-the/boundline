//! Stable public protocol contracts for Boundline.
//!
//! This crate contains transport-neutral DTOs only. Runtime authority,
//! persistence records, mutation execution, and publication remain owned by
//! Boundline's private implementation crates.

mod adapter;
mod canonical;
mod envelope;
mod evidence;
mod identifiers;
mod projection;

pub use adapter::{
    AdapterDescriptor, AdapterIdentity, AdapterProposal, AdapterTransport, CapabilityDescriptor,
    CapabilityDisposition, FrameworkAdapterAuthority, MutationProposal, OperationDescriptor,
};
pub use canonical::canonical_json;
pub use envelope::{
    MutationOutcome, MutationRequestEnvelope, MutationResultEnvelope, ProtocolVersion, ReasonCode,
};
pub use evidence::{EvidenceBinding, EvidenceFreshness, LineageDescriptor, RouteDescriptor};
pub use identifiers::{
    AcceptedDiffDigest, Claim, ContractLine, EvidenceReference, Fingerprint, OperationId,
    RepositoryId, RequestDigest, RequestId, Revision, SessionId, StageId, TraceReference,
};
pub use projection::{
    CanonOutcomeSyncStatus, ExecutorCapabilityStatus, NextAction, OperatingStage, ProofFreshness,
    PublicSessionProjection, PublicationStatus, QuarantineStatus, RecoveryStatus, SessionLifecycle,
};
