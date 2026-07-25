//! Opaque public identifiers keep protocol fields distinct without imposing runtime policy.

use serde::{Deserialize, Serialize};

macro_rules! string_identifier {
    ($(#[$metadata:meta])* $name:ident) => {
        $(#[$metadata])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates an identifier from its stable wire value.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }
    };
}

string_identifier!(
    /// Identifies a compatible public contract line.
    ContractLine
);
string_identifier!(
    /// Identifies an operation within a contract line.
    OperationId
);
string_identifier!(
    /// Identifies an idempotent mutation request.
    RequestId
);
string_identifier!(
    /// Carries the canonical digest of a mutation request.
    RequestDigest
);
string_identifier!(
    /// Identifies a governed Boundline session.
    SessionId
);
string_identifier!(
    /// Identifies one clone-local repository instance.
    RepositoryId
);
string_identifier!(
    /// Identifies a workflow or adapter stage.
    StageId
);
string_identifier!(
    /// Refers to immutable evidence without embedding its storage representation.
    EvidenceReference
);
string_identifier!(
    /// Refers to an immutable trace without exposing trace persistence.
    TraceReference
);
string_identifier!(
    /// Identifies a claim that evidence is intended to support.
    Claim
);
string_identifier!(
    /// Carries the digest of the accepted complete diff.
    AcceptedDiffDigest
);
string_identifier!(
    /// Carries a versioned worktree fingerprint digest.
    Fingerprint
);

/// Monotonic public state revision used for optimistic concurrency.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Revision(u64);

impl Revision {
    /// Creates a revision from its monotonic numeric value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}
