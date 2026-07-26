//! Trusted request digests bind canonical bytes to their named representation.

use boundline_protocol::{
    ContractLine, MutationRequestEnvelope, OperationId, ProtocolVersion, RequestDigest, RequestId,
    Revision, canonical_json,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::IdempotencyError;

const SHA256_PREFIX: &str = "sha256:";
const DIGEST_SEPARATOR: u8 = 0;
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Named canonical request representation used by internal digest records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalizationVersion {
    /// Recursively key-sorted compact JSON with floats and duplicate keys rejected.
    CanonicalJsonV1,
}

impl CanonicalizationVersion {
    const fn digest_domain(self) -> &'static str {
        match self {
            Self::CanonicalJsonV1 => "boundline:canonical-json-v1",
        }
    }
}

/// Versioned cryptographic identity of a canonical mutation request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalRequestDigest {
    /// Canonicalization rules used for the preimage.
    pub canonicalization_version: CanonicalizationVersion,
    /// Domain-separated SHA-256 digest.
    pub cryptographic_digest: RequestDigest,
}

impl CanonicalRequestDigest {
    /// Computes the trusted digest for a typed mutation request.
    pub fn from_request<T: Serialize>(
        request: &MutationRequestEnvelope<T>,
    ) -> Result<Self, IdempotencyError> {
        let version = CanonicalizationVersion::CanonicalJsonV1;
        let canonical = canonical_json(&CanonicalMutationRequest {
            protocol_version: request.protocol_version,
            contract_line: &request.contract_line,
            operation: &request.operation,
            request_id: &request.request_id,
            expected_state_revision: request.expected_state_revision,
            payload: &request.payload,
        })
        .map_err(|error| IdempotencyError::Canonicalization(error.to_string()))?;
        Ok(Self {
            canonicalization_version: version,
            cryptographic_digest: RequestDigest::new(hash_representation(
                version,
                canonical.as_bytes(),
            )),
        })
    }
}

#[derive(Serialize)]
struct CanonicalMutationRequest<'a, T> {
    protocol_version: ProtocolVersion,
    contract_line: &'a ContractLine,
    operation: &'a OperationId,
    request_id: &'a RequestId,
    expected_state_revision: Revision,
    payload: &'a T,
}

fn hash_representation(version: CanonicalizationVersion, canonical_json: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(version.digest_domain().as_bytes());
    hasher.update([DIGEST_SEPARATOR]);
    hasher.update(canonical_json);
    let digest = hasher.finalize();
    encode_sha256(&digest)
}

fn encode_sha256(digest: &[u8]) -> String {
    let mut encoded = String::with_capacity(SHA256_PREFIX.len() + digest.len() * 2);
    encoded.push_str(SHA256_PREFIX);
    for byte in digest {
        encoded.push(HEX_DIGITS[usize::from(byte >> 4)] as char);
        encoded.push(HEX_DIGITS[usize::from(byte & 0x0f)] as char);
    }
    encoded
}
