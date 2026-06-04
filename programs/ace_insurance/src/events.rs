use anchor_lang::prelude::*;

use crate::states::*;

// ================================================================
// POOL EVENTS
// ================================================================

#[event]
pub struct PoolCreatedEvent {
    pub pool_id: Pubkey,
    pub authority: Pubkey,
    pub premium_amount: u64,
    pub coverage_amount: u64,
    pub min_validators: u8,
    pub claim_period: i64,
    pub timestamp: i64,
}

#[event]
pub struct UserJoinedEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub coverage_amount: u64,
    pub premium_paid: u64,
    pub occr_discount: u8,
    pub timestamp: i64,
}

#[event]
pub struct PremiumPaidEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub total_paid: u64,
    pub timestamp: i64,
}

// ================================================================
// CLAIM EVENTS
// ================================================================

#[event]
pub struct ClaimSubmittedEvent {
    pub claim_id: Pubkey,
    pub claimant: Pubkey,
    pub pool: Pubkey,
    pub amount_requested: u64,
    pub hack_type: HackType,
    pub affected_protocol: Pubkey,
    pub incident_timestamp: i64,
    pub wallet_balance_before: u64,
    pub wallet_balance_after: u64,
    pub timestamp: i64,
}

#[event]
pub struct ClaimValidatedEvent {
    pub claim_id: Pubkey,
    pub validator: Pubkey,
    pub approved: bool,
    pub claim_status: ClaimStatus,
    pub approvals: u8,
    pub rejections: u8,
    pub auditor_approvals: u8,
    pub timestamp: i64,
}

#[event]
pub struct ClaimPaidOutEvent {
    pub claim_id: Pubkey,
    pub claimant: Pubkey,
    pub pool: Pubkey,
    pub amount: u64,
    pub is_emergency: bool,
    pub timestamp: i64,
}

// ================================================================
// VALIDATOR EVENTS
// ================================================================

#[event]
pub struct ValidatorStakedEvent {
    pub validator: Pubkey,
    pub pool: Pubkey,
    pub stake_amount: u64,
    pub reputation_score: u32,
    pub tier: ValidatorTier,
    pub timestamp: i64,
}

#[event]
pub struct ValidatorPromotedToAuditorEvent {
    pub validator: Pubkey,
    pub pool: Pubkey,
    pub reputation_score: u32,
    pub timestamp: i64,
}

// ================================================================
// VRF EVENTS
// ================================================================

#[event]
pub struct VrfStateInitializedEvent {
    pub pool: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ValidatorsAssignedEvent {
    pub pool: Pubkey,
    pub claim_id: Pubkey,
    pub validators: Vec<Pubkey>,
    pub randomness: [u8; 32],
    pub timestamp: i64,
}

// ================================================================
// DISTRIBUTION EVENTS
// ================================================================

#[event]
pub struct DistributionQueueInitializedEvent {
    pub pool: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ClaimsDistributedEvent {
    pub pool: Pubkey,
    pub round: u64,
    pub total_claims: u32,
    pub selected_claims: u32,
    pub oversubscribed: bool,
    pub available_funds: u64,
    pub timestamp: i64,
}

// ================================================================
// FAST-TRACK EVENTS
// ================================================================

#[event]
pub struct FastTrackProposedEvent {
    pub pool: Pubkey,
    pub affected_protocol: Pubkey,
    pub hack_type: HackType,
    pub proposed_by: Pubkey,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct FastTrackActivatedEvent {
    pub pool: Pubkey,
    pub affected_protocol: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPayoutEvent {
    pub claim_id: Pubkey,
    pub claimant: Pubkey,
    pub pool: Pubkey,
    pub emergency_amount: u64,
    pub remaining_amount: u64,
    pub timestamp: i64,
}

// ================================================================
// RISK SCORE EVENTS
// ================================================================

#[event]
pub struct OccrDiscountAppliedEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub discount_percent: u8,
    pub uses_multisig: bool,
    pub uses_hardware_wallet: bool,
    pub timestamp: i64,
}