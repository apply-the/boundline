//! Idempotency contract tests for canonical, revision-bound mutations.

use boundline_core::transaction::idempotency::{
    CanonicalRequestDigest, CanonicalizationVersion, IdempotencyError, IdempotencyExecutionState,
    IdempotencyKey, IdempotencyStore, MutationCompletion, MutationExecutionFailure,
};
use boundline_protocol::{
    ContractLine, EvidenceReference, MutationOutcome, MutationRequestEnvelope,
    MutationResultEnvelope, NextAction, OperationId, ProtocolVersion, ReasonCode, RequestDigest,
    RequestId, Revision, TraceReference,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;

const CONTRACT_LINE: &str = "boundline.protocol";
const OPERATION: &str = "session.advance";
const REQUEST_ID: &str = "request-001";
const INITIAL_REVISION: u64 = 7;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TestPayload {
    value: u64,
    labels: HashMap<String, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct TestResult {
    summary: String,
}

#[derive(Clone, Debug, Serialize)]
struct FloatPayload {
    value: f64,
}

fn check(condition: bool, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

fn check_eq<T: Debug + PartialEq>(
    actual: T,
    expected: T,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual == expected {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("{message}: actual {actual:?}, expected {expected:?}"))
            .into())
    }
}

fn payload(value: u64, reverse_insertion: bool) -> TestPayload {
    let mut labels = HashMap::new();
    if reverse_insertion {
        labels.insert("z".to_owned(), 2);
        labels.insert("a".to_owned(), 1);
    } else {
        labels.insert("a".to_owned(), 1);
        labels.insert("z".to_owned(), 2);
    }
    TestPayload { value, labels }
}

fn request(
    contract_line: &str,
    operation: &str,
    request_id: &str,
    expected_revision: u64,
    payload: TestPayload,
) -> Result<MutationRequestEnvelope<TestPayload>, Box<dyn std::error::Error>> {
    let mut request = MutationRequestEnvelope {
        protocol_version: ProtocolVersion::V1,
        contract_line: ContractLine::new(contract_line),
        operation: OperationId::new(operation),
        request_id: RequestId::new(request_id),
        canonical_request_digest: RequestDigest::new("uncomputed"),
        expected_state_revision: Revision::new(expected_revision),
        payload,
    };
    request.canonical_request_digest =
        CanonicalRequestDigest::from_request(&request)?.cryptographic_digest;
    Ok(request)
}

fn accepted_completion(summary: &str) -> MutationCompletion<TestResult> {
    MutationCompletion {
        outcome: MutationOutcome::Accepted(TestResult { summary: summary.to_owned() }),
        evidence: vec![EvidenceReference::new("evidence-001")],
        traces: vec![TraceReference::new("trace-001")],
        next_actions: vec![NextAction {
            operation: OperationId::new("status"),
            summary: "Inspect the terminal result".to_owned(),
        }],
    }
}

fn rejection_reason(result: &MutationResultEnvelope<TestResult>) -> Option<ReasonCode> {
    match result.outcome {
        MutationOutcome::Accepted(_) => None,
        MutationOutcome::Rejected(reason) => Some(reason),
    }
}

#[test]
fn same_key_and_digest_replays_exact_terminal_result_before_revision_validation()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let request =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let executions = AtomicUsize::new(0);

    let first = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("executed once"))
    })?;
    let replay = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("must not execute"))
    })?;

    check_eq(replay, first, "replayed terminal result")?;
    check_eq(executions.load(Ordering::SeqCst), 1, "mutation execution count")?;
    check_eq(store.current_revision()?, Revision::new(8), "advanced revision")?;
    check_eq(store.record_count()?, 1, "idempotency record count")
}

#[test]
fn same_key_replays_terminal_reason_evidence_and_next_actions()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let request =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let completion = MutationCompletion {
        outcome: MutationOutcome::<TestResult>::Rejected(ReasonCode::ExecutorCapabilityDenied),
        evidence: vec![EvidenceReference::new("denial-evidence")],
        traces: vec![TraceReference::new("denial-trace")],
        next_actions: vec![NextAction {
            operation: OperationId::new("inspect"),
            summary: "Inspect denied capability".to_owned(),
        }],
    };

    let first = store.execute(&request, || Ok(completion))?;
    let replay =
        store.execute(&request, || Ok(accepted_completion("must not replace rejection")))?;

    check_eq(replay.clone(), first, "replayed terminal rejection")?;
    check_eq(
        rejection_reason(&replay.envelope),
        Some(ReasonCode::ExecutorCapabilityDenied),
        "terminal reason code",
    )?;
    check_eq(store.current_revision()?, Revision::new(INITIAL_REVISION), "unchanged revision")
}

#[test]
fn same_key_with_different_digest_conflicts_without_mutation_or_record_change()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let first_request =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let conflicting_request = request(CONTRACT_LINE, OPERATION, REQUEST_ID, 8, payload(2, false))?;
    let executions = AtomicUsize::new(0);

    store.execute(&first_request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("original"))
    })?;
    let key = IdempotencyKey::from_request(&first_request);
    let original_record = store.record(&key)?;
    let conflict = store.execute(&conflicting_request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("conflict"))
    })?;

    check_eq(
        rejection_reason(&conflict.envelope),
        Some(ReasonCode::IdempotencyConflict),
        "conflict reason",
    )?;
    check_eq(executions.load(Ordering::SeqCst), 1, "conflicting execution count")?;
    check_eq(store.current_revision()?, Revision::new(8), "revision after conflict")?;
    check_eq(store.record(&key)?, original_record, "original record after conflict")
}

#[test]
fn existing_key_with_changed_payload_and_forged_old_digest_still_conflicts()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let original =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let mut forged = request(CONTRACT_LINE, OPERATION, REQUEST_ID, 8, payload(2, false))?;
    forged.canonical_request_digest = original.canonical_request_digest.clone();

    store.execute(&original, || Ok(accepted_completion("original")))?;
    let result = store.execute(&forged, || Ok(accepted_completion("must not execute")))?;

    check_eq(
        rejection_reason(&result.envelope),
        Some(ReasonCode::IdempotencyConflict),
        "forged old digest conflict reason",
    )?;
    check_eq(store.record_count()?, 1, "forged digest record count")?;
    check_eq(store.current_revision()?, Revision::new(8), "forged digest state revision")
}

#[test]
fn new_key_with_valid_revision_executes_once_and_records_terminal_result()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let request =
        request(CONTRACT_LINE, OPERATION, "new-request", INITIAL_REVISION, payload(1, false))?;

    let result = store.execute(&request, || Ok(accepted_completion("committed")))?;
    let record = store
        .record(&IdempotencyKey::from_request(&request))?
        .ok_or_else(|| std::io::Error::other("terminal idempotency record missing"))?;

    check_eq(result.envelope.resulting_revision, Revision::new(8), "resulting revision")?;
    check_eq(record.execution_state, IdempotencyExecutionState::Terminal, "record state")?;
    check_eq(record.resulting_state_revision, Some(Revision::new(8)), "recorded revision")?;
    check(record.recorded_terminal_result.is_some(), "terminal result was not recorded")
}

#[test]
fn new_key_with_stale_revision_rejects_without_record_or_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let request = request(CONTRACT_LINE, OPERATION, REQUEST_ID, 6, payload(1, false))?;
    let executions = AtomicUsize::new(0);

    let result = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("stale"))
    })?;

    check_eq(
        rejection_reason(&result.envelope),
        Some(ReasonCode::StateRevisionMismatch),
        "stale revision reason",
    )?;
    check_eq(executions.load(Ordering::SeqCst), 0, "stale mutation executions")?;
    check_eq(store.record_count()?, 0, "stale request record count")?;
    check_eq(store.current_revision()?, Revision::new(INITIAL_REVISION), "stale state revision")
}

#[test]
fn request_id_scope_isolated_by_operation_and_contract_line()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let requests = [
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?,
        request(CONTRACT_LINE, "session.verify", REQUEST_ID, 8, payload(1, false))?,
        request("framework.adapter", OPERATION, REQUEST_ID, 9, payload(1, false))?,
    ];
    let executions = AtomicUsize::new(0);

    for scoped_request in &requests {
        store.execute(scoped_request, || {
            executions.fetch_add(1, Ordering::SeqCst);
            Ok(accepted_completion("scoped"))
        })?;
    }

    check_eq(executions.load(Ordering::SeqCst), 3, "scoped mutation count")?;
    check_eq(store.record_count()?, 3, "scoped record count")?;
    check_eq(store.current_revision()?, Revision::new(10), "scoped resulting revision")
}

#[test]
fn canonical_digest_ignores_map_insertion_order_and_distinguishes_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let left = request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let reordered =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, true))?;
    let changed =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(2, false))?;

    check_eq(
        left.canonical_request_digest.clone(),
        reordered.canonical_request_digest,
        "map-order-independent digest",
    )?;
    check(
        left.canonical_request_digest != changed.canonical_request_digest,
        "different payloads shared a digest",
    )?;
    check_eq(
        left.canonical_request_digest.clone(),
        RequestDigest::new(
            "sha256:b3949679e3736e425fcaac7366e22bc711283d3179af5f37de97336b39edd273",
        ),
        "versioned SHA-256 digest",
    )?;
    check_eq(
        CanonicalRequestDigest::from_request(&left)?.canonicalization_version,
        CanonicalizationVersion::CanonicalJsonV1,
        "canonicalization version",
    )
}

#[test]
fn unsupported_canonical_value_fails_before_mutation() -> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::<TestResult>::new(INITIAL_REVISION);
    let request = MutationRequestEnvelope {
        protocol_version: ProtocolVersion::V1,
        contract_line: ContractLine::new(CONTRACT_LINE),
        operation: OperationId::new(OPERATION),
        request_id: RequestId::new(REQUEST_ID),
        canonical_request_digest: RequestDigest::new("uncomputed"),
        expected_state_revision: Revision::new(INITIAL_REVISION),
        payload: FloatPayload { value: f64::NAN },
    };
    let executions = AtomicUsize::new(0);

    let result = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("unsupported"))
    });

    check(
        matches!(result, Err(IdempotencyError::Canonicalization(_))),
        "unsupported value did not return canonicalization failure",
    )?;
    check_eq(executions.load(Ordering::SeqCst), 0, "unsupported mutation count")?;
    check_eq(store.record_count()?, 0, "unsupported record count")
}

#[test]
fn concurrent_identical_requests_execute_once_and_share_terminal_result()
-> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(IdempotencyStore::new(INITIAL_REVISION));
    let request =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let executions = Arc::new(AtomicUsize::new(0));
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();

    let first_store = Arc::clone(&store);
    let first_request = request.clone();
    let first_executions = Arc::clone(&executions);
    let first = thread::spawn(move || {
        first_store.execute(&first_request, || {
            first_executions.fetch_add(1, Ordering::SeqCst);
            started_sender
                .send(())
                .map_err(|error| MutationExecutionFailure::new(error.to_string()))?;
            release_receiver
                .recv()
                .map_err(|error| MutationExecutionFailure::new(error.to_string()))?;
            Ok(accepted_completion("concurrent"))
        })
    });

    started_receiver.recv()?;
    let second_store = Arc::clone(&store);
    let second_request = request.clone();
    let second_executions = Arc::clone(&executions);
    let second = thread::spawn(move || {
        second_store.execute(&second_request, || {
            second_executions.fetch_add(1, Ordering::SeqCst);
            Ok(accepted_completion("must not execute"))
        })
    });
    release_sender.send(())?;

    let first_result =
        first.join().map_err(|_| std::io::Error::other("first request thread panicked"))??;
    let second_result =
        second.join().map_err(|_| std::io::Error::other("second request thread panicked"))??;

    check_eq(second_result, first_result, "concurrent replay result")?;
    check_eq(executions.load(Ordering::SeqCst), 1, "concurrent mutation count")?;
    check_eq(store.record_count()?, 1, "concurrent record count")
}

#[test]
fn concurrent_conflicting_digest_cannot_create_a_second_record()
-> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(IdempotencyStore::new(INITIAL_REVISION));
    let original =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let conflict = request(CONTRACT_LINE, OPERATION, REQUEST_ID, 8, payload(2, false))?;
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();

    let original_store = Arc::clone(&store);
    let original_thread = thread::spawn(move || {
        original_store.execute(&original, || {
            started_sender
                .send(())
                .map_err(|error| MutationExecutionFailure::new(error.to_string()))?;
            release_receiver
                .recv()
                .map_err(|error| MutationExecutionFailure::new(error.to_string()))?;
            Ok(accepted_completion("original"))
        })
    });

    started_receiver.recv()?;
    let conflict_result =
        store.execute(&conflict, || Ok(accepted_completion("must not execute")))?;
    release_sender.send(())?;
    let original_result = original_thread
        .join()
        .map_err(|_| std::io::Error::other("original request thread panicked"))??;

    check_eq(
        rejection_reason(&conflict_result.envelope),
        Some(ReasonCode::IdempotencyConflict),
        "concurrent conflict reason",
    )?;
    check(
        matches!(original_result.envelope.outcome, MutationOutcome::Accepted(_)),
        "original request did not complete",
    )?;
    check_eq(store.record_count()?, 1, "concurrent conflict record count")
}

#[test]
fn failed_mutation_remains_explicitly_nonterminal_and_is_not_silently_retried()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::new(INITIAL_REVISION);
    let request =
        request(CONTRACT_LINE, OPERATION, REQUEST_ID, INITIAL_REVISION, payload(1, false))?;
    let executions = AtomicUsize::new(0);

    let first = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Err(MutationExecutionFailure::new("executor terminated before commit"))
    });
    let retry = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("must not silently retry"))
    });
    let record = store
        .record(&IdempotencyKey::from_request(&request))?
        .ok_or_else(|| std::io::Error::other("nonterminal record missing"))?;

    check(matches!(first, Err(IdempotencyError::MutationIncomplete(_))), "failed mutation result")?;
    check(matches!(retry, Err(IdempotencyError::NonterminalRequest)), "nonterminal retry result")?;
    check_eq(executions.load(Ordering::SeqCst), 1, "failed mutation execution count")?;
    check_eq(record.execution_state, IdempotencyExecutionState::Nonterminal, "failed state")?;
    check(record.recorded_terminal_result.is_none(), "failed mutation recorded terminal result")?;
    check_eq(store.current_revision()?, Revision::new(INITIAL_REVISION), "failed revision")
}

#[test]
fn exhausted_revision_rejects_before_mutation_and_intent_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let store = IdempotencyStore::<TestResult>::new(u64::MAX);
    let request = request(CONTRACT_LINE, OPERATION, REQUEST_ID, u64::MAX, payload(1, false))?;
    let executions = AtomicUsize::new(0);

    let result = store.execute(&request, || {
        executions.fetch_add(1, Ordering::SeqCst);
        Ok(accepted_completion("must not execute"))
    });

    check(matches!(result, Err(IdempotencyError::RevisionExhausted)), "revision exhaustion")?;
    check_eq(executions.load(Ordering::SeqCst), 0, "exhausted mutation count")?;
    check_eq(store.record_count()?, 0, "exhausted record count")
}
