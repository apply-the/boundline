//! Proposal-only adapter descriptors expose capability needs without a runtime host.

use serde::{Deserialize, Serialize};

use crate::{
    ContractLine, EvidenceReference, NextAction, OperationId, ProtocolVersion, StageId,
    TraceReference,
};

/// Immutable identity used to qualify an adapter executable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterIdentity {
    /// Adapter package identity.
    pub adapter_id: String,
    /// Adapter package version.
    pub version: String,
    /// Digest of the qualified executable.
    pub executable_digest: String,
}

/// Stable transport family declared by FrameworkAdapterV1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterTransport {
    /// One request per local subprocess with no persistent daemon.
    OneShotLocalSubprocess,
}

/// Whether a requested capability still requires Boundline admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDisposition {
    /// Boundline must explicitly admit the requested capability.
    RequiresAdmission,
}

/// Public capability request descriptor without an internal grant record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    /// Stable capability identifier.
    pub capability: String,
    /// Admission disposition for the capability.
    pub disposition: CapabilityDisposition,
}

/// Public operation descriptor used during adapter qualification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationDescriptor {
    /// Stable operation identifier.
    pub operation: OperationId,
    /// Whether the operation may propose workspace mutation.
    pub mutating: bool,
}

/// Static adapter capabilities used before any invocation is admitted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterDescriptor {
    /// Schema version used to encode the descriptor.
    pub protocol_version: ProtocolVersion,
    /// Qualified adapter executable identity.
    pub identity: AdapterIdentity,
    /// Supported framework-adapter contract line.
    pub protocol_line: ContractLine,
    /// Supported bounded transport.
    pub transport: AdapterTransport,
    /// Operations the executable declares.
    pub operations: Vec<OperationDescriptor>,
    /// Stages the executable declares.
    pub stages: Vec<StageId>,
    /// Capabilities the executable asks Boundline to admit.
    pub requested_capabilities: Vec<CapabilityDescriptor>,
}

/// Frozen authority marker for every FrameworkAdapterV1 result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterAuthority {
    /// Result is non-authoritative and requires Boundline validation.
    ProposalOnly,
}

/// Proposed file mutation identified without directly applying it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationProposal {
    /// Workspace-relative proposed path.
    pub path: String,
    /// Digest of the proposed content.
    pub content_digest: String,
}

/// Non-authoritative result descriptor returned by an adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterProposal {
    /// Invocation that produced the proposal.
    pub invocation_id: String,
    /// Explicit proposal-only authority marker.
    pub authority: FrameworkAdapterAuthority,
    /// Human-readable proposal summary.
    pub summary: String,
    /// Proposed mutations requiring Boundline admission.
    pub mutations: Vec<MutationProposal>,
    /// Proposed artifact references.
    pub artifacts: Vec<EvidenceReference>,
    /// Evidence supplied with the proposal.
    pub evidence: Vec<EvidenceReference>,
    /// Diagnostic trace references.
    pub diagnostics: Vec<TraceReference>,
    /// Requested, non-authoritative next actions.
    pub next_actions: Vec<NextAction>,
}
