//! Durable state transitions, fencing ownership, and atomic record storage.

use super::*;

pub(super) struct Ownership {
    pub(super) file: File,
}

impl Drop for Ownership {
    fn drop(&mut self) {
        let _ = File::unlock(&self.file);
    }
}

pub(super) fn terminal_delivery_result(
    record: &CanonOutcomeOutboxRecord,
) -> Option<DeliveryAttemptResult> {
    match record.state {
        OutboxState::Synchronized => Some(DeliveryAttemptResult::AlreadySynchronized),
        OutboxState::PermanentRejected => record
            .last_response
            .as_ref()
            .and_then(|response| response.reason_code)
            .map(DeliveryAttemptResult::PermanentRejected)
            .or(Some(DeliveryAttemptResult::RetryExhausted)),
        OutboxState::Conflict => Some(DeliveryAttemptResult::Conflict),
        OutboxState::Archived => Some(DeliveryAttemptResult::Archived),
        OutboxState::Pending | OutboxState::InFlight | OutboxState::RetryScheduled => None,
    }
}

pub(super) fn admit_attempt(
    path: &Path,
    record: &mut CanonOutcomeOutboxRecord,
) -> Result<(), OutboxError> {
    record.attempt_count = record.attempt_count.saturating_add(1);
    record.fencing_token = record.fencing_token.saturating_add(1);
    record.state = OutboxState::InFlight;
    record.next_retry_at_ms = None;
    write_record(path, record)
}

pub(super) fn record_transport_failure(
    path: &Path,
    mut record: CanonOutcomeOutboxRecord,
    failure: TransportFailure,
    now: u64,
) -> Result<DeliveryAttemptResult, OutboxError> {
    let reason = failure.kind.as_str().to_string();
    record.attempt_history.push(DeliveryAttempt {
        attempt_number: record.attempt_count,
        fencing_token: record.fencing_token,
        attempted_at_ms: now,
        reason_code: Some(reason.clone()),
        response: None,
    });
    record.last_reason_code = Some(reason);
    if record.attempt_count >= MAX_DELIVERY_ATTEMPTS {
        record.state = OutboxState::PermanentRejected;
        record.next_retry_at_ms = None;
        write_record(path, &record)?;
        return Ok(DeliveryAttemptResult::RetryExhausted);
    }
    record.state = OutboxState::RetryScheduled;
    record.next_retry_at_ms = Some(now.saturating_add(retry_delay(record.attempt_count)));
    write_record(path, &record)?;
    Ok(DeliveryAttemptResult::RetryScheduled)
}

pub(super) fn record_response(
    path: &Path,
    mut record: CanonOutcomeOutboxRecord,
    response: RecordOutcomeResponse,
    now: u64,
) -> Result<DeliveryAttemptResult, OutboxError> {
    let reason = response.reason_code.map(RecordOutcomeRejectionReason::as_str).map(str::to_string);
    record.attempt_history.push(DeliveryAttempt {
        attempt_number: record.attempt_count,
        fencing_token: record.fencing_token,
        attempted_at_ms: now,
        reason_code: reason.clone(),
        response: Some(response.clone()),
    });
    record.last_reason_code = reason;
    record.last_response = Some(response.clone());
    match response.disposition {
        RecordOutcomeDisposition::Recorded | RecordOutcomeDisposition::Replayed => {
            record.state = OutboxState::Synchronized;
            record.canon_revision = response.decision_memory_revision;
            record.canon_digest = response.decision_memory_digest;
            write_record(path, &record)?;
            Ok(DeliveryAttemptResult::Synchronized)
        }
        RecordOutcomeDisposition::Rejected => classify_rejection(path, record, response),
    }
}

fn classify_rejection(
    path: &Path,
    mut record: CanonOutcomeOutboxRecord,
    response: RecordOutcomeResponse,
) -> Result<DeliveryAttemptResult, OutboxError> {
    let reason = response.reason_code.ok_or(OutboxError::ResponseMismatch)?;
    if reason == RecordOutcomeRejectionReason::PersistenceFailed {
        if record.attempt_count >= MAX_DELIVERY_ATTEMPTS {
            record.state = OutboxState::PermanentRejected;
            record.next_retry_at_ms = None;
            write_record(path, &record)?;
            return Ok(DeliveryAttemptResult::RetryExhausted);
        }
        record.state = OutboxState::RetryScheduled;
        let attempted_at =
            record.attempt_history.last().map_or(0, |attempt| attempt.attempted_at_ms);
        record.next_retry_at_ms =
            Some(attempted_at.saturating_add(retry_delay(record.attempt_count)));
        write_record(path, &record)?;
        return Ok(DeliveryAttemptResult::RetryScheduled);
    }
    if reason == RecordOutcomeRejectionReason::IdentityDigestConflict {
        record.state = OutboxState::Conflict;
        record.next_retry_at_ms = None;
        write_record(path, &record)?;
        return Ok(DeliveryAttemptResult::Conflict);
    }
    record.state = OutboxState::PermanentRejected;
    record.next_retry_at_ms = None;
    write_record(path, &record)?;
    Ok(DeliveryAttemptResult::PermanentRejected(reason))
}

pub(super) fn validate_response(
    record: &CanonOutcomeOutboxRecord,
    response: &RecordOutcomeResponse,
) -> Result<(), OutboxError> {
    response.validate().map_err(|_| OutboxError::ResponseMismatch)?;
    if response.event_id != record.request.event_id
        || response.event_digest != record.request.event_digest
    {
        Err(OutboxError::ResponseMismatch)
    } else {
        Ok(())
    }
}

fn retry_delay(attempt: u32) -> u64 {
    let exponent = attempt.saturating_sub(1).min(16);
    INITIAL_RETRY_DELAY_MS.saturating_mul(1_u64 << exponent).min(MAX_RETRY_DELAY_MS)
}

pub(super) fn read_record(path: &Path) -> Result<CanonOutcomeOutboxRecord, OutboxError> {
    if !path.is_file() {
        return Err(OutboxError::NotFound);
    }
    let record: CanonOutcomeOutboxRecord = serde_json::from_slice(&fs::read(path)?)?;
    if record.schema_version == OUTBOX_SCHEMA_VERSION {
        Ok(record)
    } else {
        Err(OutboxError::IdentityDigestConflict)
    }
}

pub(super) fn write_record(
    path: &Path,
    record: &CanonOutcomeOutboxRecord,
) -> Result<(), OutboxError> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("missing outbox directory"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(record)?;
    let mut file = File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    sync_directory(parent)?;
    Ok(())
}

pub(super) fn sync_directory(path: &Path) -> Result<(), OutboxError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
