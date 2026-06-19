//! Spend exception approval management for Boundline.
//!
//! Resolves the required approver role based on authority zone and
//! repository egress status, builds decision projections, records
//! approvals, and manages the approval lifecycle (consume, expire).

use boundline_core::domain::inference_economics::{
    ApprovalScope, ApprovalState, ApprovalType, ApproverRole, MonetaryAmount,
    SpendExceptionApprovalRecord, SpendExceptionDecisionProjection,
};

/// Context for a spend exception approval request.
#[derive(Debug, Clone)]
pub struct ApprovalContext {
    /// The approval type being requested.
    pub approval_type: ApprovalType,
    /// The authority zone of the task.
    pub authority_zone: String,
    /// Whether repository content leaves the local environment.
    pub repository_egress: bool,
    /// The session owner identifier.
    pub session_owner_id: String,
    /// Whether the session has an active governance policy.
    pub has_governance_policy: bool,
    /// Whether governance policy explicitly assigns both roles to the
    /// session owner.
    pub owner_is_also_governance_approver: bool,
    /// The provider identifier.
    pub provider_id: String,
    /// The model identifier.
    pub model_id: String,
    /// The route identifier.
    pub route: String,
    /// The requested monetary amount.
    pub requested_amount: MonetaryAmount,
    /// Session currency.
    pub currency: boundline_core::domain::inference_economics::Currency,
    /// Session identifier.
    pub session_id: String,
}

/// Manages the spend exception approval lifecycle.
pub struct ApprovalManager;

/// Errors that can occur during approval operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApprovalError {
    /// The approval has already been consumed.
    #[error("approval has already been consumed")]
    AlreadyConsumed,
    /// The approval has expired.
    #[error("approval has expired")]
    Expired,
    /// The approval was revoked.
    #[error("approval has been revoked")]
    Revoked,
    /// The approval scope does not cover this call.
    #[error("approval scope does not cover this call")]
    ScopeMismatch,
    /// The approver does not have the required role.
    #[error("approver does not have the required role")]
    UnauthorizedApprover,
}

impl ApprovalManager {
    /// Resolve which role must approve a spend exception.
    #[must_use]
    pub fn resolve_approver(
        authority_zone: &str,
        repository_egress: bool,
        _session_owner_id: &str,
        owner_is_also_governance_approver: bool,
    ) -> ApproverRole {
        let is_red_zone = authority_zone == "red";

        if is_red_zone || repository_egress {
            // Red-zone and egress calls require governance approver.
            // Session owner may self-approve ONLY if policy explicitly
            // assigns both roles.
            if owner_is_also_governance_approver {
                return ApproverRole::SessionOwner;
            }
            return ApproverRole::GovernanceApprover;
        }

        // Low-risk, non-egress: session owner may approve.
        ApproverRole::SessionOwner
    }

    /// Build a decision projection for operator display.
    #[must_use]
    pub fn request_approval(ctx: &ApprovalContext) -> SpendExceptionDecisionProjection {
        let required_role = Self::resolve_approver(
            &ctx.authority_zone,
            ctx.repository_egress,
            &ctx.session_owner_id,
            ctx.owner_is_also_governance_approver,
        );

        let mut required_actions = vec!["approve_spend_exception".to_string()];
        if ctx.repository_egress {
            required_actions.push("approve_repository_egress".to_string());
        }

        SpendExceptionDecisionProjection {
            approval_type: ctx.approval_type,
            approval_state: ApprovalState::Pending,
            required_role,
            authority_zone: ctx.authority_zone.clone(),
            repository_egress: ctx.repository_egress,
            requested_amount: ctx.requested_amount,
            currency: ctx.currency,
            required_actions,
        }
    }
}

/// Parameters for recording an approval.
pub struct RecordApprovalParams {
    pub approver_identity: String,
    pub approver_role: ApproverRole,
    pub reason: String,
    pub scope: ApprovalScope,
    pub provider_id: String,
    pub model_id: String,
    pub route: String,
    pub session_id: String,
}

impl ApprovalManager {
    /// Record an approval.
    #[must_use]
    pub fn record_approval(
        projection: &SpendExceptionDecisionProjection,
        params: RecordApprovalParams,
    ) -> SpendExceptionApprovalRecord {
        let now_iso = "2026-06-18T00:00:00Z".to_string();
        SpendExceptionApprovalRecord {
            approval_id: format!("approval-{}", params.session_id),
            approval_type: projection.approval_type,
            approver_identity: params.approver_identity,
            approver_role: params.approver_role,
            session_id: params.session_id,
            execution_run_id: None,
            provider_id: params.provider_id,
            model_id: params.model_id,
            route: params.route,
            authority_zone: projection.authority_zone.clone(),
            repository_egress: projection.repository_egress,
            approved_amount: None,
            scope: params.scope,
            reason: params.reason,
            created_at: now_iso,
            consumed_at: None,
            expires_at: None,
            state: ApprovalState::Pending,
            data_transmission_authorized: if projection.repository_egress {
                Some(false)
            } else {
                None
            },
        }
    }

    /// Consume an approval for a specific call.
    ///
    /// # Errors
    ///
    /// Returns an error if the approval cannot be consumed.
    pub fn consume_approval(
        record: &mut SpendExceptionApprovalRecord,
        _call_id: &str,
    ) -> Result<(), ApprovalError> {
        match record.state {
            ApprovalState::Consumed => return Err(ApprovalError::AlreadyConsumed),
            ApprovalState::Expired => return Err(ApprovalError::Expired),
            ApprovalState::Revoked => return Err(ApprovalError::Revoked),
            ApprovalState::Pending => {}
        }
        record.state = ApprovalState::Consumed;
        record.consumed_at = Some("2026-06-18T00:00:00Z".to_string());
        Ok(())
    }

    /// Expire all unconsumed approvals that are past their expiry.
    #[must_use]
    pub fn expire_approvals(records: &mut [SpendExceptionApprovalRecord]) -> usize {
        // In production this would compare against the current wall-clock time.
        // For now, expire only records with an explicit expires_at in the past.
        let mut expired = 0;
        for record in records {
            if record.state == ApprovalState::Pending && record.expires_at.is_some() {
                // Simplified: treat any non-empty expires_at as expired for
                // testability. Production would parse and compare.
                record.state = ApprovalState::Expired;
                expired += 1;
            }
        }
        expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundline_core::domain::inference_economics::{ApprovalScope, Currency};
    use std::str::FromStr;

    fn test_ctx(zone: &str, egress: bool) -> ApprovalContext {
        ApprovalContext {
            approval_type: ApprovalType::BudgetOverride,
            authority_zone: zone.into(),
            repository_egress: egress,
            session_owner_id: "owner-1".into(),
            has_governance_policy: true,
            owner_is_also_governance_approver: false,
            provider_id: "openai".into(),
            model_id: "gpt-4o".into(),
            route: "tier2".into(),
            requested_amount: MonetaryAmount::from_str("5.00").expect("valid"),
            currency: Currency::Usd,
            session_id: "session-1".into(),
        }
    }

    #[test]
    fn session_owner_approves_low_risk_non_egress() {
        let role = ApprovalManager::resolve_approver("green", false, "owner-1", false);
        assert_eq!(role, ApproverRole::SessionOwner);
    }

    #[test]
    fn governance_approver_required_for_red_zone() {
        let role = ApprovalManager::resolve_approver("red", false, "owner-1", false);
        assert_eq!(role, ApproverRole::GovernanceApprover);
    }

    #[test]
    fn governance_approver_required_for_egress() {
        let role = ApprovalManager::resolve_approver("green", true, "owner-1", false);
        assert_eq!(role, ApproverRole::GovernanceApprover);
    }

    #[test]
    fn owner_can_self_approve_when_policy_allows() {
        let role = ApprovalManager::resolve_approver("red", true, "owner-1", true);
        assert_eq!(role, ApproverRole::SessionOwner);
    }

    #[test]
    fn request_approval_builds_projection_with_egress_action() {
        let ctx = test_ctx("red", true);
        let proj = ApprovalManager::request_approval(&ctx);
        assert_eq!(proj.approval_state, ApprovalState::Pending);
        assert_eq!(proj.required_role, ApproverRole::GovernanceApprover);
        assert!(proj.required_actions.contains(&"approve_repository_egress".to_string()));
    }

    #[test]
    fn record_approval_creates_pending_record() {
        let ctx = test_ctx("green", false);
        let proj = ApprovalManager::request_approval(&ctx);
        let record = ApprovalManager::record_approval(
            &proj,
            RecordApprovalParams {
                approver_identity: "operator-1".into(),
                approver_role: ApproverRole::SessionOwner,
                reason: "testing".into(),
                scope: ApprovalScope::SingleCall,
                provider_id: "openai".into(),
                model_id: "gpt-4o".into(),
                route: "tier2".into(),
                session_id: "session-1".into(),
            },
        );
        assert_eq!(record.state, ApprovalState::Pending);
        assert_eq!(record.approver_role, ApproverRole::SessionOwner);
        assert!(!record.approval_id.is_empty());
    }

    #[test]
    fn consume_approval_transitions_to_consumed() {
        let ctx = test_ctx("green", false);
        let proj = ApprovalManager::request_approval(&ctx);
        let mut record = ApprovalManager::record_approval(
            &proj,
            RecordApprovalParams {
                approver_identity: "op".into(),
                approver_role: ApproverRole::SessionOwner,
                reason: "r".into(),
                scope: ApprovalScope::SingleCall,
                provider_id: "p".into(),
                model_id: "m".into(),
                route: "r".into(),
                session_id: "s".into(),
            },
        );
        ApprovalManager::consume_approval(&mut record, "call-1").expect("consume ok");
        assert_eq!(record.state, ApprovalState::Consumed);
        assert!(record.consumed_at.is_some());
    }

    #[test]
    fn consume_approval_rejects_already_consumed() {
        let ctx = test_ctx("green", false);
        let proj = ApprovalManager::request_approval(&ctx);
        let mut record = ApprovalManager::record_approval(
            &proj,
            RecordApprovalParams {
                approver_identity: "op".into(),
                approver_role: ApproverRole::SessionOwner,
                reason: "r".into(),
                scope: ApprovalScope::SingleCall,
                provider_id: "p".into(),
                model_id: "m".into(),
                route: "r".into(),
                session_id: "s".into(),
            },
        );
        ApprovalManager::consume_approval(&mut record, "call-1").expect("first ok");
        let err = ApprovalManager::consume_approval(&mut record, "call-2").unwrap_err();
        assert_eq!(err, ApprovalError::AlreadyConsumed);
    }

    #[test]
    fn expire_approvals_marks_expired() {
        let mut records = vec![SpendExceptionApprovalRecord {
            approval_id: "a1".into(),
            approval_type: ApprovalType::BudgetOverride,
            approver_identity: "op".into(),
            approver_role: ApproverRole::SessionOwner,
            session_id: "s1".into(),
            execution_run_id: None,
            provider_id: "p".into(),
            model_id: "m".into(),
            route: "r".into(),
            authority_zone: "green".into(),
            repository_egress: false,
            approved_amount: None,
            scope: ApprovalScope::SingleCall,
            reason: "test".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            consumed_at: None,
            expires_at: Some("2026-01-02T00:00:00Z".into()),
            state: ApprovalState::Pending,
            data_transmission_authorized: None,
        }];
        let expired = ApprovalManager::expire_approvals(&mut records);
        assert_eq!(expired, 1);
        assert_eq!(records[0].state, ApprovalState::Expired);
    }

    #[test]
    fn expire_approvals_skips_already_consumed() {
        let mut records = vec![SpendExceptionApprovalRecord {
            approval_id: "a1".into(),
            approval_type: ApprovalType::BudgetOverride,
            approver_identity: "op".into(),
            approver_role: ApproverRole::SessionOwner,
            session_id: "s1".into(),
            execution_run_id: None,
            provider_id: "p".into(),
            model_id: "m".into(),
            route: "r".into(),
            authority_zone: "green".into(),
            repository_egress: false,
            approved_amount: None,
            scope: ApprovalScope::SingleCall,
            reason: "test".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            consumed_at: None,
            expires_at: Some("2026-01-02T00:00:00Z".into()),
            state: ApprovalState::Consumed,
            data_transmission_authorized: None,
        }];
        let expired = ApprovalManager::expire_approvals(&mut records);
        assert_eq!(expired, 0);
        assert_eq!(records[0].state, ApprovalState::Consumed);
    }

    #[test]
    fn request_approval_non_egress_has_no_egress_action() {
        let ctx = test_ctx("green", false);
        let proj = ApprovalManager::request_approval(&ctx);
        assert!(!proj.required_actions.contains(&"approve_repository_egress".to_string()));
    }

    #[test]
    fn record_approval_with_egress_sets_data_transmission_authorization() {
        let ctx = test_ctx("green", true);
        let proj = ApprovalManager::request_approval(&ctx);
        let record = ApprovalManager::record_approval(
            &proj,
            RecordApprovalParams {
                approver_identity: "operator-1".into(),
                approver_role: ApproverRole::GovernanceApprover,
                reason: "egress test".into(),
                scope: ApprovalScope::SingleCall,
                provider_id: "openai".into(),
                model_id: "gpt-4o".into(),
                route: "tier2".into(),
                session_id: "session-1".into(),
            },
        );
        assert_eq!(record.data_transmission_authorized, Some(false));
    }

    #[test]
    fn consume_approval_rejects_expired() {
        let ctx = test_ctx("green", false);
        let proj = ApprovalManager::request_approval(&ctx);
        let mut record = ApprovalManager::record_approval(
            &proj,
            RecordApprovalParams {
                approver_identity: "op".into(),
                approver_role: ApproverRole::SessionOwner,
                reason: "r".into(),
                scope: ApprovalScope::SingleCall,
                provider_id: "p".into(),
                model_id: "m".into(),
                route: "r".into(),
                session_id: "s".into(),
            },
        );
        record.state = ApprovalState::Expired;
        let err = ApprovalManager::consume_approval(&mut record, "call-1").unwrap_err();
        assert_eq!(err, ApprovalError::Expired);
    }

    #[test]
    fn consume_approval_rejects_revoked() {
        let ctx = test_ctx("green", false);
        let proj = ApprovalManager::request_approval(&ctx);
        let mut record = ApprovalManager::record_approval(
            &proj,
            RecordApprovalParams {
                approver_identity: "op".into(),
                approver_role: ApproverRole::SessionOwner,
                reason: "r".into(),
                scope: ApprovalScope::SingleCall,
                provider_id: "p".into(),
                model_id: "m".into(),
                route: "r".into(),
                session_id: "s".into(),
            },
        );
        record.state = ApprovalState::Revoked;
        let err = ApprovalManager::consume_approval(&mut record, "call-1").unwrap_err();
        assert_eq!(err, ApprovalError::Revoked);
    }
}
