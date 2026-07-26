//! Canonical idempotency records serialize mutation admission by revision.

mod digest;

pub use digest::{CanonicalRequestDigest, CanonicalizationVersion};

use boundline_protocol::{
    ContractLine, EvidenceReference, MutationOutcome, MutationRequestEnvelope,
    MutationResultEnvelope, NextAction, OperationId, ReasonCode, RequestId, Revision,
    TraceReference,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Condvar, Mutex, MutexGuard};
use tracing::{debug, warn};

/// Idempotency namespace for one mutation request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdempotencyKey {
    /// Public contract namespace.
    pub contract_line: ContractLine,
    /// Mutation operation within the contract namespace.
    pub operation: OperationId,
    /// Caller-supplied request identity.
    pub request_id: RequestId,
}

impl IdempotencyKey {
    /// Extracts the lookup key from a typed mutation request.
    pub fn from_request<T>(request: &MutationRequestEnvelope<T>) -> Self {
        Self {
            contract_line: request.contract_line.clone(),
            operation: request.operation.clone(),
            request_id: request.request_id.clone(),
        }
    }
}

/// Lifecycle of an idempotency intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdempotencyExecutionState {
    /// One admitted mutation currently owns the revision boundary.
    InFlight,
    /// Exact terminal result and revision are recorded.
    Terminal,
    /// Effects did not reach a terminal commit and require reconciliation.
    Nonterminal,
}

/// Stored mutation intent or terminal replay record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdempotencyRecord<T> {
    /// Scoped idempotency key.
    pub key: IdempotencyKey,
    /// Trusted versioned digest of the request.
    pub canonical_request_digest: CanonicalRequestDigest,
    /// Current execution lifecycle.
    pub execution_state: IdempotencyExecutionState,
    /// Exact terminal result when execution completed.
    pub recorded_terminal_result: Option<IdempotencyResult<T>>,
    /// Resulting state revision when execution completed.
    pub resulting_state_revision: Option<Revision>,
}

/// Exact replay value retained by the internal idempotency boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdempotencyResult<T> {
    /// Frozen public result projection.
    pub envelope: MutationResultEnvelope<T>,
    /// Authoritative next actions retained without widening the frozen V1 DTO.
    pub next_actions: Vec<NextAction>,
}

/// Terminal business result produced by an admitted mutation.
pub struct MutationCompletion<T> {
    /// Accepted value or stable terminal rejection.
    pub outcome: MutationOutcome<T>,
    /// Immutable evidence attached to the terminal result.
    pub evidence: Vec<EvidenceReference>,
    /// Immutable traces attached to the terminal result.
    pub traces: Vec<TraceReference>,
    /// Authoritative actions admitted after the terminal result.
    pub next_actions: Vec<NextAction>,
}

/// Process-local execution failure before terminal commit.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct MutationExecutionFailure {
    message: String,
}

impl MutationExecutionFailure {
    /// Creates an explicit nonterminal execution failure.
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

/// Idempotency processing failure that cannot be represented as a terminal mutation result.
#[derive(Debug, thiserror::Error)]
pub enum IdempotencyError {
    /// Canonicalization rejected or could not encode the request.
    #[error("canonical request failed: {0}")]
    Canonicalization(String),
    /// Caller-provided digest does not match the trusted canonical digest.
    #[error("canonical request digest does not match the request")]
    CanonicalDigestMismatch,
    /// A prior attempt exists but did not reach terminal commit.
    #[error("request has a nonterminal idempotency record requiring reconciliation")]
    NonterminalRequest,
    /// The newly admitted mutation failed before terminal commit.
    #[error("mutation did not reach terminal commit: {0}")]
    MutationIncomplete(String),
    /// Shared transaction state cannot be trusted after synchronization poisoning.
    #[error("idempotency synchronization state is poisoned")]
    SynchronizationPoisoned,
    /// Monotonic state revision cannot advance beyond its numeric range.
    #[error("state revision is exhausted")]
    RevisionExhausted,
}

struct StoreState<T> {
    current_revision: u64,
    records: HashMap<IdempotencyKey, IdempotencyRecord<T>>,
    active_mutation: Option<IdempotencyKey>,
}

/// Process-local reference coordinator for canonical idempotency semantics.
///
/// The coordinator models atomic claim and completion transitions. Durable
/// storage adapters are intentionally deferred to the later persistence
/// milestone.
pub struct IdempotencyStore<T> {
    state: Mutex<StoreState<T>>,
    changed: Condvar,
}

impl<T: Clone> IdempotencyStore<T> {
    /// Creates an empty store at the supplied authoritative revision.
    pub fn new(initial_revision: u64) -> Self {
        Self {
            state: Mutex::new(StoreState {
                current_revision: initial_revision,
                records: HashMap::new(),
                active_mutation: None,
            }),
            changed: Condvar::new(),
        }
    }

    /// Returns the current authoritative revision.
    pub fn current_revision(&self) -> Result<Revision, IdempotencyError> {
        self.lock_state().map(|state| Revision::new(state.current_revision))
    }

    /// Returns the number of intent and terminal records.
    pub fn record_count(&self) -> Result<usize, IdempotencyError> {
        self.lock_state().map(|state| state.records.len())
    }

    /// Returns a read-only snapshot of one idempotency record.
    pub fn record(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<IdempotencyRecord<T>>, IdempotencyError> {
        self.lock_state().map(|state| state.records.get(key).cloned())
    }

    /// Executes a new canonical mutation or replays its exact terminal result.
    pub fn execute<P, F>(
        &self,
        request: &MutationRequestEnvelope<P>,
        mutation: F,
    ) -> Result<IdempotencyResult<T>, IdempotencyError>
    where
        P: Serialize,
        F: FnOnce() -> Result<MutationCompletion<T>, MutationExecutionFailure>,
    {
        let canonical_digest = CanonicalRequestDigest::from_request(request)?;
        let key = IdempotencyKey::from_request(request);
        let previous_revision = self.admit_or_replay(request, &key, &canonical_digest)?;
        let Some(previous_revision) = previous_revision else {
            return self.replay_or_reject(request, &key, &canonical_digest);
        };

        match mutation() {
            Ok(completion) => {
                self.complete(request, key, canonical_digest, previous_revision, completion)
            }
            Err(failure) => {
                self.mark_nonterminal(&key)?;
                Err(IdempotencyError::MutationIncomplete(failure.message))
            }
        }
    }

    fn admit_or_replay<P>(
        &self,
        request: &MutationRequestEnvelope<P>,
        key: &IdempotencyKey,
        canonical_digest: &CanonicalRequestDigest,
    ) -> Result<Option<u64>, IdempotencyError> {
        let mut state = self.lock_state()?;
        loop {
            if let Some(record) = state.records.get(key) {
                if record.canonical_request_digest != *canonical_digest
                    || record.canonical_request_digest.cryptographic_digest
                        != request.canonical_request_digest
                {
                    return Ok(None);
                }
                match record.execution_state {
                    IdempotencyExecutionState::Terminal
                    | IdempotencyExecutionState::Nonterminal => return Ok(None),
                    IdempotencyExecutionState::InFlight => {
                        state = self.wait_for_change(state)?;
                        continue;
                    }
                }
            }

            if canonical_digest.cryptographic_digest != request.canonical_request_digest {
                return Err(IdempotencyError::CanonicalDigestMismatch);
            }

            if state.active_mutation.is_some() {
                state = self.wait_for_change(state)?;
                continue;
            }

            if request.expected_state_revision != Revision::new(state.current_revision) {
                debug!(
                    ?key,
                    current_revision = state.current_revision,
                    "mutation revision rejected"
                );
                return Ok(None);
            }
            if state.current_revision == u64::MAX {
                return Err(IdempotencyError::RevisionExhausted);
            }

            let previous_revision = state.current_revision;
            state.records.insert(
                key.clone(),
                IdempotencyRecord {
                    key: key.clone(),
                    canonical_request_digest: canonical_digest.clone(),
                    execution_state: IdempotencyExecutionState::InFlight,
                    recorded_terminal_result: None,
                    resulting_state_revision: None,
                },
            );
            state.active_mutation = Some(key.clone());
            debug!(?key, revision = previous_revision, "idempotent mutation admitted");
            return Ok(Some(previous_revision));
        }
    }

    fn replay_or_reject<P>(
        &self,
        request: &MutationRequestEnvelope<P>,
        key: &IdempotencyKey,
        canonical_digest: &CanonicalRequestDigest,
    ) -> Result<IdempotencyResult<T>, IdempotencyError> {
        let state = self.lock_state()?;
        if let Some(record) = state.records.get(key) {
            if record.canonical_request_digest != *canonical_digest {
                warn!(?key, "idempotency key reused with a conflicting canonical request");
                return Ok(rejection_result(
                    request,
                    Revision::new(state.current_revision),
                    ReasonCode::IdempotencyConflict,
                ));
            }
            return match record.execution_state {
                IdempotencyExecutionState::Terminal => record
                    .recorded_terminal_result
                    .clone()
                    .inspect(|_| debug!(?key, "replaying terminal idempotency result"))
                    .ok_or(IdempotencyError::SynchronizationPoisoned),
                IdempotencyExecutionState::Nonterminal => Err(IdempotencyError::NonterminalRequest),
                IdempotencyExecutionState::InFlight => {
                    Err(IdempotencyError::SynchronizationPoisoned)
                }
            };
        }

        Ok(rejection_result(
            request,
            Revision::new(state.current_revision),
            ReasonCode::StateRevisionMismatch,
        ))
    }

    fn complete<P>(
        &self,
        request: &MutationRequestEnvelope<P>,
        key: IdempotencyKey,
        canonical_digest: CanonicalRequestDigest,
        previous_revision: u64,
        completion: MutationCompletion<T>,
    ) -> Result<IdempotencyResult<T>, IdempotencyError> {
        let resulting_revision = match &completion.outcome {
            MutationOutcome::Accepted(_) => {
                let Some(next_revision) = previous_revision.checked_add(1) else {
                    self.mark_nonterminal(&key)?;
                    return Err(IdempotencyError::RevisionExhausted);
                };
                next_revision
            }
            MutationOutcome::Rejected(_) => previous_revision,
        };
        let result = IdempotencyResult {
            envelope: MutationResultEnvelope {
                protocol_version: request.protocol_version,
                request_id: request.request_id.clone(),
                canonical_request_digest: request.canonical_request_digest.clone(),
                previous_revision: Revision::new(previous_revision),
                resulting_revision: Revision::new(resulting_revision),
                outcome: completion.outcome,
                evidence: completion.evidence,
                traces: completion.traces,
            },
            next_actions: completion.next_actions,
        };

        let mut state = self.lock_state()?;
        state.current_revision = resulting_revision;
        state.records.insert(
            key.clone(),
            IdempotencyRecord {
                key: key.clone(),
                canonical_request_digest: canonical_digest,
                execution_state: IdempotencyExecutionState::Terminal,
                recorded_terminal_result: Some(result.clone()),
                resulting_state_revision: Some(Revision::new(resulting_revision)),
            },
        );
        state.active_mutation = None;
        debug!(?key, resulting_revision, "idempotent mutation reached terminal state");
        self.changed.notify_all();
        Ok(result)
    }

    fn mark_nonterminal(&self, key: &IdempotencyKey) -> Result<(), IdempotencyError> {
        let mut state = self.lock_state()?;
        if let Some(record) = state.records.get_mut(key) {
            record.execution_state = IdempotencyExecutionState::Nonterminal;
        }
        state.active_mutation = None;
        warn!(?key, "idempotent mutation retained as nonterminal");
        self.changed.notify_all();
        Ok(())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, StoreState<T>>, IdempotencyError> {
        self.state.lock().map_err(|_| IdempotencyError::SynchronizationPoisoned)
    }

    fn wait_for_change<'a>(
        &self,
        state: MutexGuard<'a, StoreState<T>>,
    ) -> Result<MutexGuard<'a, StoreState<T>>, IdempotencyError> {
        self.changed.wait(state).map_err(|_| IdempotencyError::SynchronizationPoisoned)
    }
}

fn rejection_result<P, T>(
    request: &MutationRequestEnvelope<P>,
    current_revision: Revision,
    reason: ReasonCode,
) -> IdempotencyResult<T> {
    IdempotencyResult {
        envelope: MutationResultEnvelope {
            protocol_version: request.protocol_version,
            request_id: request.request_id.clone(),
            canonical_request_digest: request.canonical_request_digest.clone(),
            previous_revision: current_revision,
            resulting_revision: current_revision,
            outcome: MutationOutcome::Rejected(reason),
            evidence: Vec::new(),
            traces: Vec::new(),
        },
        next_actions: Vec::new(),
    }
}
