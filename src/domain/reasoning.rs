use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::configuration::RouteSlot;
use crate::domain::governance::CanonMode;

pub const REASONING_POSTURE_V1_CONTRACT_LINE: &str = "governed_reasoning_posture_v1";
const CURRENT_BOUNDLINE_VERSION: &str = env!("CARGO_PKG_VERSION");

const MINIMUM_SINGLE_PARTICIPANT: usize = 1;
const MINIMUM_PAIR_PARTICIPANTS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningProfileId {
    BoundedSelfConsistency,
    IndependentPairReview,
    HeterogeneousSecurityReview,
    BoundedReflexion,
}

impl ReasoningProfileId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BoundedSelfConsistency => "bounded_self_consistency",
            Self::IndependentPairReview => "independent_pair_review",
            Self::HeterogeneousSecurityReview => "heterogeneous_security_review",
            Self::BoundedReflexion => "bounded_reflexion",
        }
    }

    pub const fn family(self) -> ReasoningProfileFamily {
        match self {
            Self::BoundedSelfConsistency => ReasoningProfileFamily::SelfConsistency,
            Self::IndependentPairReview => ReasoningProfileFamily::BlindReview,
            Self::HeterogeneousSecurityReview => ReasoningProfileFamily::HeterogeneousReview,
            Self::BoundedReflexion => ReasoningProfileFamily::Reflexion,
        }
    }
}

impl std::fmt::Display for ReasoningProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningProfileFamily {
    SelfConsistency,
    BlindReview,
    HeterogeneousReview,
    Reflexion,
    DebateEnabled,
}

impl ReasoningProfileFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelfConsistency => "self_consistency",
            Self::BlindReview => "blind_review",
            Self::HeterogeneousReview => "heterogeneous_review",
            Self::Reflexion => "reflexion",
            Self::DebateEnabled => "debate_enabled",
        }
    }

    pub const fn allows_debate(self) -> bool {
        matches!(self, Self::DebateEnabled)
    }

    pub const fn allows_reflexion(self) -> bool {
        matches!(self, Self::Reflexion)
    }

    pub const fn minimum_participants(self) -> usize {
        match self {
            Self::SelfConsistency | Self::Reflexion => MINIMUM_SINGLE_PARTICIPANT,
            Self::BlindReview | Self::HeterogeneousReview | Self::DebateEnabled => {
                MINIMUM_PAIR_PARTICIPANTS
            }
        }
    }
}

impl std::fmt::Display for ReasoningProfileFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningActivationStatus {
    Pending,
    Active,
    Completed,
    Degraded,
    Blocked,
    Interrupted,
    Escalated,
    Failed,
}

impl ReasoningActivationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Degraded => "degraded",
            Self::Blocked => "blocked",
            Self::Interrupted => "interrupted",
            Self::Escalated => "escalated",
            Self::Failed => "failed",
        }
    }

    pub const fn halts_outer_workflow(self) -> bool {
        matches!(self, Self::Blocked | Self::Interrupted | Self::Escalated | Self::Failed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningActivationTrigger {
    CanonRequiredChallenge,
    GovernanceEscalation,
    OperatorPolicy,
    LocalFixture,
}

impl ReasoningActivationTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CanonRequiredChallenge => "canon_required_challenge",
            Self::GovernanceEscalation => "governance_escalation",
            Self::OperatorPolicy => "operator_policy",
            Self::LocalFixture => "local_fixture",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningParticipantRoleKind {
    IndependentPath,
    BlindReviewer,
    HeterogeneousReviewer,
    Critic,
    Reviser,
    Arbiter,
}

impl ReasoningParticipantRoleKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IndependentPath => "independent_path",
            Self::BlindReviewer => "blind_reviewer",
            Self::HeterogeneousReviewer => "heterogeneous_reviewer",
            Self::Critic => "critic",
            Self::Reviser => "reviser",
            Self::Arbiter => "arbiter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningRoutePreference {
    Planning,
    Implementation,
    Verification,
    Review,
    Adjudication,
}

impl ReasoningRoutePreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Implementation => "implementation",
            Self::Verification => "verification",
            Self::Review => "review",
            Self::Adjudication => "adjudication",
        }
    }
}

impl From<RouteSlot> for ReasoningRoutePreference {
    fn from(value: RouteSlot) -> Self {
        match value {
            RouteSlot::Planning => Self::Planning,
            RouteSlot::Implementation => Self::Implementation,
            RouteSlot::Verification => Self::Verification,
            RouteSlot::Review => Self::Review,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningAdjudicationMode {
    None,
    Arbiter,
    GovernanceReview,
    HumanOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndependenceAssessmentResult {
    Passed,
    Degraded,
    Failed,
}

impl IndependenceAssessmentResult {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningParticipantStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Omitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningIterationKind {
    Branch,
    DebateRound,
    ReflexionRevision,
    AdjudicationStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningIterationCondition {
    Active,
    Stagnated,
    Completed,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningOutcomeKind {
    Converged,
    Disagreed,
    Adjudicated,
    Degraded,
    Blocked,
    Interrupted,
    Escalated,
    Failed,
}

impl ReasoningOutcomeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Converged => "converged",
            Self::Disagreed => "disagreed",
            Self::Adjudicated => "adjudicated",
            Self::Degraded => "degraded",
            Self::Blocked => "blocked",
            Self::Interrupted => "interrupted",
            Self::Escalated => "escalated",
            Self::Failed => "failed",
        }
    }

    pub const fn requires_explicit_reason(self) -> bool {
        matches!(
            self,
            Self::Degraded | Self::Blocked | Self::Interrupted | Self::Escalated | Self::Failed
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningConfidenceLevel {
    Low,
    Medium,
    High,
}

impl ReasoningConfidenceLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningAdmissionEffect {
    None,
    Warn,
    Gate,
    Escalate,
}

impl ReasoningAdmissionEffect {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Warn => "warn",
            Self::Gate => "gate",
            Self::Escalate => "escalate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonAdmissionPriority {
    Advisory,
    RequiredBeforeContinue,
    RequiredBeforeAcceptance,
}

impl CanonAdmissionPriority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Advisory => "advisory",
            Self::RequiredBeforeContinue => "required_before_continue",
            Self::RequiredBeforeAcceptance => "required_before_acceptance",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningBudget {
    pub max_participants: usize,
    pub max_branches: usize,
    pub max_debate_rounds: usize,
    pub max_reflexion_revisions: usize,
    pub max_calls: usize,
    pub max_tokens: usize,
    pub max_adjudication_steps: usize,
}

impl ReasoningBudget {
    pub fn validate_for_family(
        &self,
        family: ReasoningProfileFamily,
    ) -> Result<(), ReasoningProfileError> {
        for (label, value) in [
            ("max_participants", self.max_participants),
            ("max_branches", self.max_branches),
            ("max_calls", self.max_calls),
            ("max_tokens", self.max_tokens),
            ("max_adjudication_steps", self.max_adjudication_steps),
        ] {
            if value == 0 {
                return Err(ReasoningProfileError::NonPositiveBudgetValue(label));
            }
        }

        if self.max_participants < family.minimum_participants() {
            return Err(ReasoningProfileError::InsufficientParticipants {
                required: family.minimum_participants(),
                actual: self.max_participants,
            });
        }

        if self.max_debate_rounds > 0 && !family.allows_debate() {
            return Err(ReasoningProfileError::DebateDisabledForFamily(family));
        }

        if self.max_reflexion_revisions > 0 && !family.allows_reflexion() {
            return Err(ReasoningProfileError::ReflexionDisabledForFamily(family));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependenceFloor {
    #[serde(default)]
    pub route_distinct: bool,
    #[serde(default)]
    pub provider_distinct: bool,
    #[serde(default)]
    pub context_distinct: bool,
    #[serde(default)]
    pub prompt_pattern_distinct: bool,
    pub minimum_participants: usize,
}

impl IndependenceFloor {
    pub fn validate_for_family(
        &self,
        family: ReasoningProfileFamily,
    ) -> Result<(), ReasoningProfileError> {
        if self.minimum_participants == 0 {
            return Err(ReasoningProfileError::InvalidMinimumParticipants);
        }

        if self.minimum_participants < family.minimum_participants() {
            return Err(ReasoningProfileError::InsufficientParticipants {
                required: family.minimum_participants(),
                actual: self.minimum_participants,
            });
        }

        if family == ReasoningProfileFamily::BlindReview
            && self.minimum_participants < MINIMUM_PAIR_PARTICIPANTS
        {
            return Err(ReasoningProfileError::BlindReviewRequiresPair);
        }

        if family == ReasoningProfileFamily::HeterogeneousReview
            && !self.provider_distinct
            && !self.route_distinct
        {
            return Err(ReasoningProfileError::HeterogeneousReviewNeedsDistinctProviderOrRoute);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantRoleDefinition {
    pub role_id: String,
    pub role_kind: ReasoningParticipantRoleKind,
    pub preferred_slot: ReasoningRoutePreference,
    pub independence_requirements: IndependenceFloor,
    #[serde(default)]
    pub required: bool,
}

impl ParticipantRoleDefinition {
    fn validate(&self, family: ReasoningProfileFamily) -> Result<(), ReasoningProfileError> {
        if self.role_id.trim().is_empty() {
            return Err(ReasoningProfileError::MissingRoleId);
        }

        self.independence_requirements.validate_for_family(family)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningDegradationPolicy {
    #[serde(default)]
    pub allow_degraded_independence: bool,
    #[serde(default)]
    pub allow_reduced_participants: bool,
    #[serde(default)]
    pub interruptible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_next_action: Option<String>,
}

impl ReasoningDegradationPolicy {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.blocked_next_action.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(ReasoningProfileError::EmptyBlockedNextAction);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningProfileDefinition {
    pub profile_id: ReasoningProfileId,
    pub family: ReasoningProfileFamily,
    pub allowed_stages: Vec<CanonMode>,
    pub limits: ReasoningBudget,
    pub participant_roles: Vec<ParticipantRoleDefinition>,
    pub adjudication_mode: ReasoningAdjudicationMode,
    pub degradation_policy: ReasoningDegradationPolicy,
}

impl ReasoningProfileDefinition {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.profile_id.family() != self.family {
            return Err(ReasoningProfileError::ProfileFamilyMismatch {
                profile_id: self.profile_id,
                family: self.family,
            });
        }

        if self.allowed_stages.is_empty() {
            return Err(ReasoningProfileError::MissingAllowedStages(self.profile_id));
        }

        if self.participant_roles.is_empty() {
            return Err(ReasoningProfileError::MissingParticipantRoles(self.profile_id));
        }

        self.limits.validate_for_family(self.family)?;
        self.degradation_policy.validate()?;

        let mut role_ids = BTreeSet::new();
        for role in &self.participant_roles {
            role.validate(self.family)?;
            if !role_ids.insert(role.role_id.clone()) {
                return Err(ReasoningProfileError::DuplicateRoleId(role.role_id.clone()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningCompatibilityWindow {
    pub boundline_min: String,
    pub boundline_max_exclusive: String,
    pub canon_min: String,
    pub canon_max_exclusive: String,
    pub contract_line: String,
}

impl ReasoningCompatibilityWindow {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.contract_line.trim().is_empty() {
            return Err(ReasoningProfileError::MissingContractLine);
        }

        if !self.is_supported_contract_line() {
            return Err(ReasoningProfileError::UnsupportedContractLine(self.contract_line.clone()));
        }

        for (label, value) in [
            ("boundline_min", self.boundline_min.as_str()),
            ("boundline_max_exclusive", self.boundline_max_exclusive.as_str()),
            ("canon_min", self.canon_min.as_str()),
            ("canon_max_exclusive", self.canon_max_exclusive.as_str()),
        ] {
            if parse_version_triplet(value).is_none() {
                return Err(ReasoningProfileError::InvalidCompatibilityVersion(label));
            }
        }

        Ok(())
    }

    pub fn is_supported_contract_line(&self) -> bool {
        self.contract_line == REASONING_POSTURE_V1_CONTRACT_LINE
    }

    pub fn admits_versions(&self, boundline: &str, canon: &str) -> bool {
        let Some(boundline_version) = parse_version_triplet(boundline) else {
            return false;
        };
        let Some(canon_version) = parse_version_triplet(canon) else {
            return false;
        };
        let Some(boundline_min) = parse_version_triplet(self.boundline_min.as_str()) else {
            return false;
        };
        let Some(boundline_max) = parse_version_triplet(self.boundline_max_exclusive.as_str())
        else {
            return false;
        };
        let Some(canon_min) = parse_version_triplet(self.canon_min.as_str()) else {
            return false;
        };
        let Some(canon_max) = parse_version_triplet(self.canon_max_exclusive.as_str()) else {
            return false;
        };

        boundline_version >= boundline_min
            && boundline_version < boundline_max
            && canon_version >= canon_min
            && canon_version < canon_max
    }

    pub fn admits_current_release_pair(&self) -> bool {
        self.admits_versions(
            CURRENT_BOUNDLINE_VERSION,
            crate::domain::distribution::SUPPORTED_CANON_VERSION,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonChallengePostureInput {
    pub contract_line: String,
    pub compatibility_window: ReasoningCompatibilityWindow,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_profile_family: Option<ReasoningProfileFamily>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_profile_id: Option<ReasoningProfileId>,
    pub minimum_independence: IndependenceFloor,
    pub admission_priority: CanonAdmissionPriority,
    #[serde(default)]
    pub confidence_handoff_required: bool,
    pub provenance_ref: String,
}

impl CanonChallengePostureInput {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.contract_line.trim().is_empty() {
            return Err(ReasoningProfileError::MissingContractLine);
        }

        if self.contract_line != REASONING_POSTURE_V1_CONTRACT_LINE {
            return Err(ReasoningProfileError::UnsupportedContractLine(self.contract_line.clone()));
        }

        self.compatibility_window.validate()?;

        if self.contract_line != self.compatibility_window.contract_line {
            return Err(ReasoningProfileError::CompatibilityContractLineMismatch {
                posture: self.contract_line.clone(),
                window: self.compatibility_window.contract_line.clone(),
            });
        }

        if !self.compatibility_window.admits_current_release_pair() {
            return Err(ReasoningProfileError::IncompatibleCompatibilityWindow {
                boundline: CURRENT_BOUNDLINE_VERSION.to_string(),
                canon: crate::domain::distribution::SUPPORTED_CANON_VERSION.to_string(),
            });
        }

        if self.required_profile_family.is_none() && self.required_profile_id.is_none() {
            return Err(ReasoningProfileError::MissingRequiredProfileSelector);
        }

        let family = self
            .required_profile_family
            .or_else(|| self.required_profile_id.map(ReasoningProfileId::family))
            .ok_or(ReasoningProfileError::MissingRequiredProfileSelector)?;

        self.minimum_independence.validate_for_family(family)?;

        if self.provenance_ref.trim().is_empty() {
            return Err(ReasoningProfileError::MissingProvenanceRef);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningObservedDistinctness {
    pub distinct_routes: usize,
    pub distinct_providers: usize,
    pub distinct_contexts: usize,
    pub distinct_prompt_patterns: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependenceAssessment {
    pub requested_floor: IndependenceFloor,
    pub observed_distinctions: ReasoningObservedDistinctness,
    pub result: IndependenceAssessmentResult,
    pub reason: String,
}

impl IndependenceAssessment {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        self.requested_floor.validate_for_family(ReasoningProfileFamily::SelfConsistency)?;

        if self.reason.trim().is_empty() {
            return Err(ReasoningProfileError::MissingIndependenceReason);
        }

        if self.result == IndependenceAssessmentResult::Degraded && self.reason.trim().is_empty() {
            return Err(ReasoningProfileError::MissingIndependenceReason);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantAssignment {
    pub role_id: String,
    pub participant_id: String,
    pub effective_route: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_family: Option<String>,
    pub context_basis: String,
    pub prompting_pattern: String,
    pub status: ReasoningParticipantStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_summary: Option<String>,
}

impl ParticipantAssignment {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.role_id.trim().is_empty() {
            return Err(ReasoningProfileError::MissingRoleId);
        }
        if self.participant_id.trim().is_empty() {
            return Err(ReasoningProfileError::MissingParticipantId);
        }
        if self.effective_route.trim().is_empty() {
            return Err(ReasoningProfileError::MissingEffectiveRoute(self.participant_id.clone()));
        }
        if self.context_basis.trim().is_empty() {
            return Err(ReasoningProfileError::MissingContextBasis(self.participant_id.clone()));
        }
        if self.prompting_pattern.trim().is_empty() {
            return Err(ReasoningProfileError::MissingPromptingPattern(
                self.participant_id.clone(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningIterationRecord {
    pub iteration_kind: ReasoningIterationKind,
    pub iteration_index: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participants: Vec<String>,
    pub summary: String,
    #[serde(default)]
    pub novelty: bool,
    pub condition: ReasoningIterationCondition,
}

impl ReasoningIterationRecord {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.summary.trim().is_empty() {
            return Err(ReasoningProfileError::MissingIterationSummary);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningOutcome {
    pub outcome_kind: ReasoningOutcomeKind,
    pub headline: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disagreement_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub iterations: Vec<ReasoningIterationRecord>,
}

impl ReasoningOutcome {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.headline.trim().is_empty() {
            return Err(ReasoningProfileError::MissingOutcomeHeadline);
        }

        if self.outcome_kind.requires_explicit_reason()
            && self.next_action.is_none()
            && self.disagreement_summary.is_none()
        {
            return Err(ReasoningProfileError::MissingOutcomeReason);
        }

        for iteration in &self.iterations {
            iteration.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningConfidenceContribution {
    pub confidence_level: ReasoningConfidenceLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub basis: Vec<String>,
    pub admission_effect: ReasoningAdmissionEffect,
    pub summary: String,
}

impl ReasoningConfidenceContribution {
    pub fn validate(&self) -> Result<(), ReasoningProfileError> {
        if self.summary.trim().is_empty() {
            return Err(ReasoningProfileError::MissingConfidenceSummary);
        }

        if self.basis.iter().any(|value| value.trim().is_empty()) {
            return Err(ReasoningProfileError::EmptyConfidenceBasis);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileActivationRecord {
    pub activation_id: String,
    pub stage_key: String,
    pub profile_id: ReasoningProfileId,
    pub trigger: ReasoningActivationTrigger,
    pub activation_reason: String,
    pub status: ReasoningActivationStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participants: Vec<ParticipantAssignment>,
    pub budget: ReasoningBudget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posture: Option<CanonChallengePostureInput>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub independence: Option<IndependenceAssessment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<ReasoningOutcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<ReasoningConfidenceContribution>,
}

impl ProfileActivationRecord {
    pub fn validate_against(
        &self,
        definition: &ReasoningProfileDefinition,
    ) -> Result<(), ReasoningProfileError> {
        if self.activation_id.trim().is_empty() {
            return Err(ReasoningProfileError::MissingActivationId);
        }
        if self.stage_key.trim().is_empty() {
            return Err(ReasoningProfileError::MissingStageKey);
        }
        if self.activation_reason.trim().is_empty() {
            return Err(ReasoningProfileError::MissingActivationReason);
        }
        if self.profile_id != definition.profile_id {
            return Err(ReasoningProfileError::ActivationProfileMismatch {
                expected: definition.profile_id,
                actual: self.profile_id,
            });
        }

        self.budget.validate_for_family(definition.family)?;

        let mut participant_ids = BTreeSet::new();
        let mut resolved_roles = BTreeSet::new();
        for participant in &self.participants {
            participant.validate()?;
            if !participant_ids.insert(participant.participant_id.clone()) {
                return Err(ReasoningProfileError::DuplicateParticipantId(
                    participant.participant_id.clone(),
                ));
            }
            resolved_roles.insert(participant.role_id.clone());
        }

        for role in definition.participant_roles.iter().filter(|role| role.required) {
            if !resolved_roles.contains(&role.role_id)
                && self.status == ReasoningActivationStatus::Active
            {
                return Err(ReasoningProfileError::MissingRequiredParticipantRole(
                    role.role_id.clone(),
                ));
            }
        }

        if let Some(posture) = &self.posture {
            posture.validate()?;
        }
        if let Some(independence) = &self.independence {
            independence.validate()?;
        }
        if let Some(outcome) = &self.outcome {
            outcome.validate()?;
        }
        if let Some(confidence) = &self.confidence {
            confidence.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ReasoningProfileError {
    #[error("reasoning profile '{0}' must declare at least one allowed stage")]
    MissingAllowedStages(ReasoningProfileId),
    #[error("reasoning profile '{0}' must declare at least one participant role")]
    MissingParticipantRoles(ReasoningProfileId),
    #[error("reasoning profile '{profile_id}' must use family '{family}'")]
    ProfileFamilyMismatch { profile_id: ReasoningProfileId, family: ReasoningProfileFamily },
    #[error("reasoning budget field '{0}' must be greater than zero")]
    NonPositiveBudgetValue(&'static str),
    #[error("reasoning profile requires at least {required} participants, found {actual}")]
    InsufficientParticipants { required: usize, actual: usize },
    #[error("reasoning family '{0}' does not permit debate rounds")]
    DebateDisabledForFamily(ReasoningProfileFamily),
    #[error("reasoning family '{0}' does not permit reflexion revisions")]
    ReflexionDisabledForFamily(ReasoningProfileFamily),
    #[error("independence floor minimum_participants must be at least one")]
    InvalidMinimumParticipants,
    #[error("blind review requires at least two participants")]
    BlindReviewRequiresPair,
    #[error("heterogeneous review requires a distinct provider or route")]
    HeterogeneousReviewNeedsDistinctProviderOrRoute,
    #[error("participant role id must not be empty")]
    MissingRoleId,
    #[error("participant role '{0}' is duplicated")]
    DuplicateRoleId(String),
    #[error("blocked_next_action must not be empty when present")]
    EmptyBlockedNextAction,
    #[error("reasoning contract line must not be empty")]
    MissingContractLine,
    #[error("reasoning contract line '{0}' is unsupported")]
    UnsupportedContractLine(String),
    #[error("compatibility window field '{0}' must be a semantic version")]
    InvalidCompatibilityVersion(&'static str),
    #[error(
        "Canon posture contract line '{posture}' does not match compatibility window contract line '{window}'"
    )]
    CompatibilityContractLineMismatch { posture: String, window: String },
    #[error(
        "Canon posture compatibility window does not admit Boundline {boundline} and Canon {canon}"
    )]
    IncompatibleCompatibilityWindow { boundline: String, canon: String },
    #[error("Canon posture must declare required_profile_family or required_profile_id")]
    MissingRequiredProfileSelector,
    #[error("Canon posture provenance_ref must not be empty")]
    MissingProvenanceRef,
    #[error("independence assessment reason must not be empty")]
    MissingIndependenceReason,
    #[error("participant assignment id must not be empty")]
    MissingParticipantId,
    #[error("participant '{0}' must declare an effective route")]
    MissingEffectiveRoute(String),
    #[error("participant '{0}' must declare a context basis")]
    MissingContextBasis(String),
    #[error("participant '{0}' must declare a prompting pattern")]
    MissingPromptingPattern(String),
    #[error("reasoning iteration summary must not be empty")]
    MissingIterationSummary,
    #[error("reasoning outcome headline must not be empty")]
    MissingOutcomeHeadline,
    #[error("non-success reasoning outcomes must declare a reason or next action")]
    MissingOutcomeReason,
    #[error("confidence summary must not be empty")]
    MissingConfidenceSummary,
    #[error("confidence basis entries must not be empty")]
    EmptyConfidenceBasis,
    #[error("activation id must not be empty")]
    MissingActivationId,
    #[error("activation stage key must not be empty")]
    MissingStageKey,
    #[error("activation reason must not be empty")]
    MissingActivationReason,
    #[error("activation expected profile '{expected}' but found '{actual}'")]
    ActivationProfileMismatch { expected: ReasoningProfileId, actual: ReasoningProfileId },
    #[error("participant '{0}' is duplicated inside the activation")]
    DuplicateParticipantId(String),
    #[error("required participant role '{0}' is missing from the activation")]
    MissingRequiredParticipantRole(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

fn parse_version_triplet(raw: &str) -> Option<ParsedVersion> {
    let mut parts = raw.trim().split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(ParsedVersion { major, minor, patch })
}

#[cfg(test)]
mod tests {
    use super::{
        CanonAdmissionPriority, CanonChallengePostureInput, IndependenceAssessment,
        IndependenceAssessmentResult, IndependenceFloor, ParticipantAssignment,
        ProfileActivationRecord, REASONING_POSTURE_V1_CONTRACT_LINE, ReasoningActivationStatus,
        ReasoningActivationTrigger, ReasoningAdjudicationMode, ReasoningAdmissionEffect,
        ReasoningBudget, ReasoningCompatibilityWindow, ReasoningConfidenceContribution,
        ReasoningConfidenceLevel, ReasoningDegradationPolicy, ReasoningIterationCondition,
        ReasoningIterationKind, ReasoningIterationRecord, ReasoningOutcome, ReasoningOutcomeKind,
        ReasoningParticipantRoleKind, ReasoningParticipantStatus, ReasoningProfileDefinition,
        ReasoningProfileError, ReasoningProfileFamily, ReasoningProfileId,
        ReasoningRoutePreference,
    };
    use crate::domain::configuration::RouteSlot;
    use crate::domain::governance::CanonMode;

    fn fixture_definition() -> ReasoningProfileDefinition {
        ReasoningProfileDefinition {
            profile_id: ReasoningProfileId::IndependentPairReview,
            family: ReasoningProfileFamily::BlindReview,
            allowed_stages: vec![CanonMode::Verification, CanonMode::PrReview],
            limits: ReasoningBudget {
                max_participants: 2,
                max_branches: 1,
                max_debate_rounds: 0,
                max_reflexion_revisions: 0,
                max_calls: 2,
                max_tokens: 2048,
                max_adjudication_steps: 1,
            },
            participant_roles: vec![super::ParticipantRoleDefinition {
                role_id: "reviewer-a".to_string(),
                role_kind: super::ReasoningParticipantRoleKind::BlindReviewer,
                preferred_slot: super::ReasoningRoutePreference::Verification,
                independence_requirements: IndependenceFloor {
                    route_distinct: true,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: true,
                    minimum_participants: 2,
                },
                required: true,
            }],
            adjudication_mode: super::ReasoningAdjudicationMode::Arbiter,
            degradation_policy: super::ReasoningDegradationPolicy {
                allow_degraded_independence: false,
                allow_reduced_participants: false,
                interruptible: true,
                blocked_next_action: Some("boundline inspect --json".to_string()),
            },
        }
    }

    #[test]
    fn reasoning_profile_definition_accepts_supported_blind_review_shape() {
        assert!(fixture_definition().validate().is_ok());
    }

    #[test]
    fn reasoning_budget_rejects_debate_rounds_for_non_debate_family() {
        let mut definition = fixture_definition();
        definition.limits.max_debate_rounds = 1;

        assert_eq!(
            definition.validate(),
            Err(super::ReasoningProfileError::DebateDisabledForFamily(
                ReasoningProfileFamily::BlindReview,
            )),
        );
    }

    #[test]
    fn reasoning_budget_accepts_single_participant_self_consistency() {
        let budget = ReasoningBudget {
            max_participants: 1,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 1,
            max_tokens: 1024,
            max_adjudication_steps: 1,
        };

        assert!(budget.validate_for_family(ReasoningProfileFamily::SelfConsistency).is_ok());
    }

    #[test]
    fn reasoning_budget_accepts_reflexion_revisions_for_reflexion_family() {
        let budget = ReasoningBudget {
            max_participants: 1,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 1,
            max_calls: 2,
            max_tokens: 2048,
            max_adjudication_steps: 1,
        };

        assert!(budget.validate_for_family(ReasoningProfileFamily::Reflexion).is_ok());
    }

    #[test]
    fn reasoning_budget_rejects_insufficient_participants_for_debate_family() {
        let budget = ReasoningBudget {
            max_participants: 1,
            max_branches: 1,
            max_debate_rounds: 1,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 2048,
            max_adjudication_steps: 1,
        };

        assert_eq!(
            budget.validate_for_family(ReasoningProfileFamily::DebateEnabled),
            Err(super::ReasoningProfileError::InsufficientParticipants { required: 2, actual: 1 }),
        );
    }

    #[test]
    fn independence_floor_rejects_heterogeneous_review_without_distinct_route_or_provider() {
        let floor = IndependenceFloor {
            route_distinct: false,
            provider_distinct: false,
            context_distinct: false,
            prompt_pattern_distinct: false,
            minimum_participants: 2,
        };

        assert_eq!(
            floor.validate_for_family(ReasoningProfileFamily::HeterogeneousReview),
            Err(super::ReasoningProfileError::HeterogeneousReviewNeedsDistinctProviderOrRoute),
        );
    }

    #[test]
    fn compatibility_window_admits_supported_pair() {
        let window = ReasoningCompatibilityWindow {
            boundline_min: "0.61.0".to_string(),
            boundline_max_exclusive: "0.62.0".to_string(),
            canon_min: "0.57.0".to_string(),
            canon_max_exclusive: "0.58.0".to_string(),
            contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
        };

        assert!(window.validate().is_ok());
        assert!(window.admits_versions("0.61.0", "0.57.0"));
        assert!(!window.admits_versions("0.62.0", "0.57.0"));
    }

    #[test]
    fn compatibility_window_rejects_unsupported_contract_line() {
        let window = ReasoningCompatibilityWindow {
            boundline_min: "0.61.0".to_string(),
            boundline_max_exclusive: "0.62.0".to_string(),
            canon_min: "0.57.0".to_string(),
            canon_max_exclusive: "0.58.0".to_string(),
            contract_line: "governed_reasoning_posture_v2".to_string(),
        };

        assert_eq!(
            window.validate(),
            Err(super::ReasoningProfileError::UnsupportedContractLine(
                "governed_reasoning_posture_v2".to_string(),
            )),
        );
    }

    #[test]
    fn canon_posture_requires_profile_selector() {
        let posture = CanonChallengePostureInput {
            contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
            compatibility_window: ReasoningCompatibilityWindow {
                boundline_min: "0.61.0".to_string(),
                boundline_max_exclusive: "0.62.0".to_string(),
                canon_min: "0.57.0".to_string(),
                canon_max_exclusive: "0.58.0".to_string(),
                contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
            },
            required_profile_family: None,
            required_profile_id: None,
            minimum_independence: IndependenceFloor {
                route_distinct: true,
                provider_distinct: true,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 2,
            },
            admission_priority: CanonAdmissionPriority::RequiredBeforeAcceptance,
            confidence_handoff_required: true,
            provenance_ref: "packet:reasoning-posture-123".to_string(),
        };

        assert_eq!(
            posture.validate(),
            Err(super::ReasoningProfileError::MissingRequiredProfileSelector),
        );
    }

    #[test]
    fn canon_posture_rejects_unsupported_contract_line() {
        let posture = CanonChallengePostureInput {
            contract_line: "governed_reasoning_posture_v2".to_string(),
            compatibility_window: ReasoningCompatibilityWindow {
                boundline_min: "0.61.0".to_string(),
                boundline_max_exclusive: "0.62.0".to_string(),
                canon_min: "0.57.0".to_string(),
                canon_max_exclusive: "0.58.0".to_string(),
                contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
            },
            required_profile_family: Some(ReasoningProfileFamily::BlindReview),
            required_profile_id: None,
            minimum_independence: IndependenceFloor {
                route_distinct: true,
                provider_distinct: true,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 2,
            },
            admission_priority: CanonAdmissionPriority::RequiredBeforeAcceptance,
            confidence_handoff_required: true,
            provenance_ref: "packet:reasoning-posture-123".to_string(),
        };

        assert_eq!(
            posture.validate(),
            Err(super::ReasoningProfileError::UnsupportedContractLine(
                "governed_reasoning_posture_v2".to_string(),
            )),
        );
    }

    #[test]
    fn canon_posture_rejects_mismatched_contract_line_between_posture_and_window() {
        let posture = CanonChallengePostureInput {
            contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
            compatibility_window: ReasoningCompatibilityWindow {
                boundline_min: "0.61.0".to_string(),
                boundline_max_exclusive: "0.62.0".to_string(),
                canon_min: "0.57.0".to_string(),
                canon_max_exclusive: "0.58.0".to_string(),
                contract_line: "governed_reasoning_posture_v2".to_string(),
            },
            required_profile_family: Some(ReasoningProfileFamily::BlindReview),
            required_profile_id: None,
            minimum_independence: IndependenceFloor {
                route_distinct: true,
                provider_distinct: true,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 2,
            },
            admission_priority: CanonAdmissionPriority::RequiredBeforeAcceptance,
            confidence_handoff_required: true,
            provenance_ref: "packet:reasoning-posture-123".to_string(),
        };

        assert_eq!(
            posture.validate(),
            Err(super::ReasoningProfileError::UnsupportedContractLine(
                "governed_reasoning_posture_v2".to_string(),
            )),
        );
    }

    #[test]
    fn canon_posture_rejects_incompatible_active_release_pair() {
        let posture = CanonChallengePostureInput {
            contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
            compatibility_window: ReasoningCompatibilityWindow {
                boundline_min: "0.62.0".to_string(),
                boundline_max_exclusive: "0.63.0".to_string(),
                canon_min: "0.57.0".to_string(),
                canon_max_exclusive: "0.58.0".to_string(),
                contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
            },
            required_profile_family: Some(ReasoningProfileFamily::BlindReview),
            required_profile_id: None,
            minimum_independence: IndependenceFloor {
                route_distinct: true,
                provider_distinct: true,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 2,
            },
            admission_priority: CanonAdmissionPriority::RequiredBeforeAcceptance,
            confidence_handoff_required: true,
            provenance_ref: "packet:reasoning-posture-123".to_string(),
        };

        assert_eq!(
            posture.validate(),
            Err(super::ReasoningProfileError::IncompatibleCompatibilityWindow {
                boundline: super::CURRENT_BOUNDLINE_VERSION.to_string(),
                canon: crate::domain::distribution::SUPPORTED_CANON_VERSION.to_string(),
            }),
        );
    }

    #[test]
    fn activation_status_helpers_cover_blocking_states() {
        assert!(!ReasoningActivationStatus::Pending.halts_outer_workflow());
        assert!(!ReasoningActivationStatus::Active.halts_outer_workflow());
        assert!(!ReasoningActivationStatus::Completed.halts_outer_workflow());
        assert!(!ReasoningActivationStatus::Degraded.halts_outer_workflow());
        assert!(ReasoningActivationStatus::Blocked.halts_outer_workflow());
        assert!(ReasoningActivationStatus::Interrupted.halts_outer_workflow());
        assert!(ReasoningActivationStatus::Escalated.halts_outer_workflow());
        assert!(ReasoningActivationStatus::Failed.halts_outer_workflow());
    }

    #[test]
    fn outcome_kind_helpers_cover_reason_required_states() {
        assert!(!ReasoningOutcomeKind::Converged.requires_explicit_reason());
        assert!(!ReasoningOutcomeKind::Disagreed.requires_explicit_reason());
        assert!(!ReasoningOutcomeKind::Adjudicated.requires_explicit_reason());
        assert!(ReasoningOutcomeKind::Degraded.requires_explicit_reason());
        assert!(ReasoningOutcomeKind::Blocked.requires_explicit_reason());
        assert!(ReasoningOutcomeKind::Interrupted.requires_explicit_reason());
        assert!(ReasoningOutcomeKind::Escalated.requires_explicit_reason());
        assert!(ReasoningOutcomeKind::Failed.requires_explicit_reason());
    }

    #[test]
    fn reasoning_runtime_vocabulary_helpers_cover_all_first_release_variants() {
        let profiles = [
            (
                ReasoningProfileId::BoundedSelfConsistency,
                "bounded_self_consistency",
                ReasoningProfileFamily::SelfConsistency,
            ),
            (
                ReasoningProfileId::IndependentPairReview,
                "independent_pair_review",
                ReasoningProfileFamily::BlindReview,
            ),
            (
                ReasoningProfileId::HeterogeneousSecurityReview,
                "heterogeneous_security_review",
                ReasoningProfileFamily::HeterogeneousReview,
            ),
            (
                ReasoningProfileId::BoundedReflexion,
                "bounded_reflexion",
                ReasoningProfileFamily::Reflexion,
            ),
        ];
        for (profile_id, expected_text, expected_family) in profiles {
            assert_eq!(profile_id.as_str(), expected_text);
            assert_eq!(profile_id.to_string(), expected_text);
            assert_eq!(profile_id.family(), expected_family);
        }

        let families = [
            (ReasoningProfileFamily::SelfConsistency, "self_consistency", false, false, 1),
            (ReasoningProfileFamily::BlindReview, "blind_review", false, false, 2),
            (ReasoningProfileFamily::HeterogeneousReview, "heterogeneous_review", false, false, 2),
            (ReasoningProfileFamily::Reflexion, "reflexion", false, true, 1),
            (ReasoningProfileFamily::DebateEnabled, "debate_enabled", true, false, 2),
        ];
        for (family, expected_text, allows_debate, allows_reflexion, minimum_participants) in
            families
        {
            assert_eq!(family.as_str(), expected_text);
            assert_eq!(family.to_string(), expected_text);
            assert_eq!(family.allows_debate(), allows_debate);
            assert_eq!(family.allows_reflexion(), allows_reflexion);
            assert_eq!(family.minimum_participants(), minimum_participants);
        }

        let statuses = [
            (ReasoningActivationStatus::Pending, "pending"),
            (ReasoningActivationStatus::Active, "active"),
            (ReasoningActivationStatus::Completed, "completed"),
            (ReasoningActivationStatus::Degraded, "degraded"),
            (ReasoningActivationStatus::Blocked, "blocked"),
            (ReasoningActivationStatus::Interrupted, "interrupted"),
            (ReasoningActivationStatus::Escalated, "escalated"),
            (ReasoningActivationStatus::Failed, "failed"),
        ];
        for (status, expected_text) in statuses {
            assert_eq!(status.as_str(), expected_text);
        }

        let triggers = [
            (ReasoningActivationTrigger::CanonRequiredChallenge, "canon_required_challenge"),
            (ReasoningActivationTrigger::GovernanceEscalation, "governance_escalation"),
            (ReasoningActivationTrigger::OperatorPolicy, "operator_policy"),
            (ReasoningActivationTrigger::LocalFixture, "local_fixture"),
        ];
        for (trigger, expected_text) in triggers {
            assert_eq!(trigger.as_str(), expected_text);
        }

        let role_kinds = [
            (ReasoningParticipantRoleKind::IndependentPath, "independent_path"),
            (ReasoningParticipantRoleKind::BlindReviewer, "blind_reviewer"),
            (ReasoningParticipantRoleKind::HeterogeneousReviewer, "heterogeneous_reviewer"),
            (ReasoningParticipantRoleKind::Critic, "critic"),
            (ReasoningParticipantRoleKind::Reviser, "reviser"),
            (ReasoningParticipantRoleKind::Arbiter, "arbiter"),
        ];
        for (role_kind, expected_text) in role_kinds {
            assert_eq!(role_kind.as_str(), expected_text);
        }

        let route_preferences = [
            (ReasoningRoutePreference::Planning, "planning", RouteSlot::Planning),
            (ReasoningRoutePreference::Implementation, "implementation", RouteSlot::Implementation),
            (ReasoningRoutePreference::Verification, "verification", RouteSlot::Verification),
            (ReasoningRoutePreference::Review, "review", RouteSlot::Review),
            (ReasoningRoutePreference::Adjudication, "adjudication", RouteSlot::Review),
        ];
        for (route_preference, expected_text, route_slot) in route_preferences {
            assert_eq!(route_preference.as_str(), expected_text);
            if route_preference != ReasoningRoutePreference::Adjudication {
                assert_eq!(ReasoningRoutePreference::from(route_slot), route_preference);
            }
        }

        let independence_results = [
            (IndependenceAssessmentResult::Passed, "passed"),
            (IndependenceAssessmentResult::Degraded, "degraded"),
            (IndependenceAssessmentResult::Failed, "failed"),
        ];
        for (result, expected_text) in independence_results {
            assert_eq!(result.as_str(), expected_text);
        }

        let outcome_kinds = [
            (ReasoningOutcomeKind::Converged, "converged"),
            (ReasoningOutcomeKind::Disagreed, "disagreed"),
            (ReasoningOutcomeKind::Adjudicated, "adjudicated"),
            (ReasoningOutcomeKind::Degraded, "degraded"),
            (ReasoningOutcomeKind::Blocked, "blocked"),
            (ReasoningOutcomeKind::Interrupted, "interrupted"),
            (ReasoningOutcomeKind::Escalated, "escalated"),
            (ReasoningOutcomeKind::Failed, "failed"),
        ];
        for (outcome_kind, expected_text) in outcome_kinds {
            assert_eq!(outcome_kind.as_str(), expected_text);
        }

        let confidence_levels = [
            (ReasoningConfidenceLevel::Low, "low"),
            (ReasoningConfidenceLevel::Medium, "medium"),
            (ReasoningConfidenceLevel::High, "high"),
        ];
        for (confidence_level, expected_text) in confidence_levels {
            assert_eq!(confidence_level.as_str(), expected_text);
        }

        let admission_effects = [
            (ReasoningAdmissionEffect::None, "none"),
            (ReasoningAdmissionEffect::Warn, "warn"),
            (ReasoningAdmissionEffect::Gate, "gate"),
            (ReasoningAdmissionEffect::Escalate, "escalate"),
        ];
        for (admission_effect, expected_text) in admission_effects {
            assert_eq!(admission_effect.as_str(), expected_text);
        }

        let priorities = [
            (CanonAdmissionPriority::Advisory, "advisory"),
            (CanonAdmissionPriority::RequiredBeforeContinue, "required_before_continue"),
            (CanonAdmissionPriority::RequiredBeforeAcceptance, "required_before_acceptance"),
        ];
        for (priority, expected_text) in priorities {
            assert_eq!(priority.as_str(), expected_text);
        }

        let _ = ReasoningAdjudicationMode::None;
        let _ = ReasoningAdjudicationMode::Arbiter;
        let _ = ReasoningAdjudicationMode::GovernanceReview;
        let _ = ReasoningAdjudicationMode::HumanOverride;
        let _ = ReasoningParticipantStatus::Pending;
        let _ = ReasoningParticipantStatus::Running;
        let _ = ReasoningParticipantStatus::Completed;
        let _ = ReasoningParticipantStatus::Failed;
        let _ = ReasoningParticipantStatus::Omitted;
        let _ = ReasoningIterationKind::Branch;
        let _ = ReasoningIterationKind::DebateRound;
        let _ = ReasoningIterationKind::ReflexionRevision;
        let _ = ReasoningIterationKind::AdjudicationStep;
        let _ = ReasoningIterationCondition::Active;
        let _ = ReasoningIterationCondition::Stagnated;
        let _ = ReasoningIterationCondition::Completed;
        let _ = ReasoningIterationCondition::Exhausted;
    }

    #[test]
    fn reasoning_validation_helpers_cover_remaining_error_paths() {
        let zero_budget = ReasoningBudget {
            max_participants: 0,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 1,
            max_tokens: 1,
            max_adjudication_steps: 1,
        };
        assert_eq!(
            zero_budget.validate_for_family(ReasoningProfileFamily::SelfConsistency),
            Err(ReasoningProfileError::NonPositiveBudgetValue("max_participants")),
        );

        let reflexion_disabled = ReasoningBudget {
            max_participants: 2,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 1,
            max_calls: 1,
            max_tokens: 1,
            max_adjudication_steps: 1,
        };
        assert_eq!(
            reflexion_disabled.validate_for_family(ReasoningProfileFamily::BlindReview),
            Err(ReasoningProfileError::ReflexionDisabledForFamily(
                ReasoningProfileFamily::BlindReview,
            )),
        );

        let invalid_floor = IndependenceFloor {
            route_distinct: false,
            provider_distinct: false,
            context_distinct: false,
            prompt_pattern_distinct: false,
            minimum_participants: 0,
        };
        assert_eq!(
            invalid_floor.validate_for_family(ReasoningProfileFamily::SelfConsistency),
            Err(ReasoningProfileError::InvalidMinimumParticipants),
        );

        let mut invalid_definition = fixture_definition();
        invalid_definition.allowed_stages.clear();
        assert_eq!(
            invalid_definition.validate(),
            Err(ReasoningProfileError::MissingAllowedStages(
                ReasoningProfileId::IndependentPairReview,
            )),
        );

        let mut invalid_definition = fixture_definition();
        invalid_definition.participant_roles.clear();
        assert_eq!(
            invalid_definition.validate(),
            Err(ReasoningProfileError::MissingParticipantRoles(
                ReasoningProfileId::IndependentPairReview,
            )),
        );

        let mut invalid_definition = fixture_definition();
        invalid_definition.family = ReasoningProfileFamily::SelfConsistency;
        assert_eq!(
            invalid_definition.validate(),
            Err(ReasoningProfileError::ProfileFamilyMismatch {
                profile_id: ReasoningProfileId::IndependentPairReview,
                family: ReasoningProfileFamily::SelfConsistency,
            }),
        );

        let mut invalid_definition = fixture_definition();
        invalid_definition.participant_roles.push(invalid_definition.participant_roles[0].clone());
        assert_eq!(
            invalid_definition.validate(),
            Err(ReasoningProfileError::DuplicateRoleId("reviewer-a".to_string())),
        );

        let invalid_policy = ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some("   ".to_string()),
        };
        assert_eq!(invalid_policy.validate(), Err(ReasoningProfileError::EmptyBlockedNextAction),);

        let invalid_window = ReasoningCompatibilityWindow {
            boundline_min: "not-a-version".to_string(),
            boundline_max_exclusive: "0.62.0".to_string(),
            canon_min: "0.57.0".to_string(),
            canon_max_exclusive: "0.58.0".to_string(),
            contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
        };
        assert_eq!(
            invalid_window.validate(),
            Err(ReasoningProfileError::InvalidCompatibilityVersion("boundline_min")),
        );
    }

    #[test]
    fn reasoning_record_validators_cover_supporting_models_and_activation_errors() {
        let invalid_assessment = IndependenceAssessment {
            requested_floor: IndependenceFloor {
                route_distinct: false,
                provider_distinct: false,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 1,
            },
            observed_distinctions: super::ReasoningObservedDistinctness {
                distinct_routes: 1,
                distinct_providers: 1,
                distinct_contexts: 1,
                distinct_prompt_patterns: 1,
            },
            result: IndependenceAssessmentResult::Passed,
            reason: "  ".to_string(),
        };
        assert_eq!(
            invalid_assessment.validate(),
            Err(ReasoningProfileError::MissingIndependenceReason),
        );

        let invalid_participant = ParticipantAssignment {
            role_id: "reviewer-a".to_string(),
            participant_id: "participant-a".to_string(),
            effective_route: "".to_string(),
            provider_family: None,
            context_basis: "governance_stage:verify".to_string(),
            prompting_pattern: "blind_reviewer".to_string(),
            status: ReasoningParticipantStatus::Pending,
            result_summary: None,
        };
        assert_eq!(
            invalid_participant.validate(),
            Err(ReasoningProfileError::MissingEffectiveRoute("participant-a".to_string())),
        );

        let invalid_iteration = ReasoningIterationRecord {
            iteration_kind: ReasoningIterationKind::DebateRound,
            iteration_index: 1,
            participants: vec!["reviewer-a".to_string()],
            summary: " ".to_string(),
            novelty: false,
            condition: ReasoningIterationCondition::Stagnated,
        };
        assert_eq!(
            invalid_iteration.validate(),
            Err(ReasoningProfileError::MissingIterationSummary),
        );

        let invalid_outcome = ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Blocked,
            headline: "blocked".to_string(),
            disagreement_summary: None,
            next_action: None,
            iterations: Vec::new(),
        };
        assert_eq!(invalid_outcome.validate(), Err(ReasoningProfileError::MissingOutcomeReason),);

        let invalid_confidence = ReasoningConfidenceContribution {
            confidence_level: ReasoningConfidenceLevel::Low,
            basis: vec![" ".to_string()],
            admission_effect: ReasoningAdmissionEffect::Gate,
            summary: "bounded confidence".to_string(),
        };
        assert_eq!(invalid_confidence.validate(), Err(ReasoningProfileError::EmptyConfidenceBasis),);

        let definition = fixture_definition();
        let invalid_activation = ProfileActivationRecord {
            activation_id: "activation-1".to_string(),
            stage_key: "bug-fix:verify".to_string(),
            profile_id: definition.profile_id,
            trigger: ReasoningActivationTrigger::OperatorPolicy,
            activation_reason: "need stronger challenge".to_string(),
            status: ReasoningActivationStatus::Active,
            participants: Vec::new(),
            budget: definition.limits.clone(),
            posture: None,
            independence: None,
            outcome: None,
            confidence: None,
        };
        assert_eq!(
            invalid_activation.validate_against(&definition),
            Err(ReasoningProfileError::MissingRequiredParticipantRole("reviewer-a".to_string(),)),
        );

        let duplicate_participant_activation = ProfileActivationRecord {
            activation_id: "activation-2".to_string(),
            stage_key: "bug-fix:verify".to_string(),
            profile_id: definition.profile_id,
            trigger: ReasoningActivationTrigger::OperatorPolicy,
            activation_reason: "need stronger challenge".to_string(),
            status: ReasoningActivationStatus::Blocked,
            participants: vec![
                ParticipantAssignment {
                    role_id: "reviewer-a".to_string(),
                    participant_id: "participant-a".to_string(),
                    effective_route: "review:claude:sonnet-4.6".to_string(),
                    provider_family: Some("claude".to_string()),
                    context_basis: "governance_stage:verify".to_string(),
                    prompting_pattern: "blind_reviewer".to_string(),
                    status: ReasoningParticipantStatus::Completed,
                    result_summary: Some("first".to_string()),
                },
                ParticipantAssignment {
                    role_id: "reviewer-a".to_string(),
                    participant_id: "participant-a".to_string(),
                    effective_route: "review:gemini:gemini-2.5-pro".to_string(),
                    provider_family: Some("gemini".to_string()),
                    context_basis: "governance_stage:verify".to_string(),
                    prompting_pattern: "blind_reviewer".to_string(),
                    status: ReasoningParticipantStatus::Completed,
                    result_summary: Some("second".to_string()),
                },
            ],
            budget: definition.limits.clone(),
            posture: None,
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Blocked,
                headline: "blocked".to_string(),
                disagreement_summary: Some("duplicate participant".to_string()),
                next_action: None,
                iterations: Vec::new(),
            }),
            confidence: None,
        };
        assert_eq!(
            duplicate_participant_activation.validate_against(&definition),
            Err(ReasoningProfileError::DuplicateParticipantId("participant-a".to_string(),)),
        );
    }
}
