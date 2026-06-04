use anchor_lang::prelude::*;


// ENUMS

/// Specific hack types that ACE covers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug,InitSpace)]
pub enum HackType {
    SmartContractExploit,
    WalletHack,
    PhishingAttack,
    SocialEngineering,
    FlashLoanAttack,
    OracleManipulation,
    ReentrancyAttack,
    AccessControlBypass,
    BridgeExploit,
    RugPull,
    Other,
}



/// Severity tiers for hack assessment
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum SeverityTier {
    Low,     // Small-scale wallet drain (individual)
    Medium,  // Smart contract bug (pool-wide)
    Critical, // Protocol-wide rugpull or bridge exploit
}



/// Claim status tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ClaimStatus {
    Pending,
    UnderValidation,
    Approved,
    Rejected,
    Distributed,
    Queued,
    FastTrackPending,
    EmergencyLiquidated,
}



/// Validator tier levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ValidatorTier {
    Standard,    // Regular staker
    Auditor,     // Verified security researcher — votes carry more weight
}



/// Individual validation record
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug,InitSpace)]
pub struct Validation {
    pub validator: Pubkey,
    pub approved: bool,
    #[max_len(200)]
    pub reason: String,
    pub timestamp: i64,
    #[max_len(500)]
    pub technical_analysis: String, // Specific to hack: evidence analysis notes
}



/// Risk score for a user (OCCR - On-Chain Coverage Risk Rating)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct UserRiskScore {
    /// The user address being evaluated
    pub user: Pubkey,
    /// How long the user has been live (days)
    pub age_days: u32,
    /// Flag to check if their insurance policy is currently active and paid up
    pub is_insured: bool,
    /// The exact premium amount they must pay/have paid to stay covered
    pub premium_owed: u64,
}




// MAIN ACCOUNTS

/// Main insurance pool account — holds hack insurance pool config
#[account]
#[derive(InitSpace)]
pub struct InsurancePool {
    pub pool_id: Pubkey,
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub premium_amount: u64,
    pub coverage_amount: u64,
    pub total_pooled: u64,
    pub total_members: u32,
    pub active_claims: u32,
    pub claim_period: i64,
    pub min_validators: u8,
    pub bump: u8,
    /// Governance key that can trigger fast-track
    pub governance_authority: Pubkey,
    /// Whether fast-track is currently active for this pool
    pub fast_track_active: bool,
    /// Timestamp when pool was created
    pub created_at: i64,
}



/// User coverage account — tracks individual member's hack insurance
#[account]
#[derive(InitSpace)]
pub struct UserCoverage {
    pub user: Pubkey,
    pub occr_score: u8,
    pub pool: Pubkey,
    pub premiums_paid: u64,
    pub last_payment: i64,
    pub coverage_active: bool,
    pub coverage_amount: u64,
    pub claims_made: u8,
    pub joined_at: i64,
    pub bump: u8,
    /// User's OCCR discount factor (0-20% off premium)
    pub occr_discount: u8,
    /// Whether user uses a multisig
    pub uses_multisig: bool,
    /// Whether user uses a hardware wallet
    pub uses_hardware_wallet: bool,
}

/// Pool-level validator stake tracker — lists all staked validators for a pool
#[account]
#[derive(InitSpace)]
pub struct ValidatorStakePool {
    pub pool: Pubkey,
    #[max_len(100)]
    pub validators: Vec<Pubkey>,
    pub total_validators: u32,
    pub bump: u8,
}

/// Validator stake account — for hack security experts
#[account]
#[derive(InitSpace)]
pub struct ValidatorStake {
    pub validator: Pubkey,
    pub stake_amount: u64,
    pub validations_completed: u32,
    pub successful_validations: u32,
    pub reputation_score: u32,
    pub last_validation: i64,
    pub bump: u8,
    pub tier: ValidatorTier,
    /// Vote weight multiplier: Standard=1, Auditor=3
    pub vote_weight: u8,
}

impl ValidatorStake {
   
    pub const INITIAL_REPUTATION: u32 = 5000;
    pub const MAX_REPUTATION: u32 = 10000;
    pub const AUDITOR_REPUTATION_THRESHOLD: u32 = 8000;
}


/// VRF state for random validator selection
#[account]
#[derive(InitSpace)]
pub struct VrfState {
    pub pool: Pubkey,
    pub switchboard_vrf: Pubkey,
    pub authority: Pubkey,
    pub last_randomness: Option<[u8; 32]>,
    pub last_timestamp: i64,
    #[max_len(50)]
    pub pending_claims: Vec<Pubkey>,
    pub requests_completed: u64,
    pub bump: u8,
}

/// Claim request — hack-specific with evidence_hash
#[account]
#[derive(InitSpace)]
pub struct ClaimRequest {
    pub claim_id: Pubkey,
    pub claimant: Pubkey,
    pub pool: Pubkey,
    pub amount_requested: u64,
    pub hack_type: HackType,
    pub incident_timestamp: i64,
    /// IPFS/Arweave hash containing: tx sigs, affected protocol, post-mortem
    #[max_len(150)]
    pub evidence_hash: String,
    /// Affected protocol address
    pub affected_protocol: Pubkey,
    /// Wallet balance check: snapshot of holdings before incident
    pub wallet_balance_before: u64,
    /// Wallet balance check: snapshot after incident (to verify loss)
    pub wallet_balance_after: u64,
    #[max_len(10)]
    pub validators_assigned: Vec<Pubkey>,
    #[max_len(10)]
    pub validations: Vec<Validation>,
    pub approvals: u8,
    pub rejections: u8,
    pub auditor_approvals: u8,
    pub status: ClaimStatus,
    pub vrf_result: Option<[u8; 32]>,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
    pub payout_amount: Option<u64>,
    pub bump: u8,
    /// Severity assessment
    pub severity: Option<SeverityTier>,
    /// Fast-track emergency payout (20% immediate)
    pub emergency_payout_received: bool,
}

/// Distribution queue for managing oversubscribed claims
#[account]
#[derive(InitSpace)]
pub struct DistributionQueue {
    pub pool: Pubkey,
    pub total_approved_claims: u32,
    pub total_requested_amount: u64,
    pub available_funds: u64,
    #[max_len(100)]
    pub pending_claims: Vec<Pubkey>,
    #[max_len(50)]
    pub selected_claims: Vec<Pubkey>,
    pub vrf_result: Option<[u8; 32]>,
    pub is_oversubscribed: bool,
    pub distribution_round: u64,
    pub last_distribution: i64,
    pub bump: u8,
}

/// Fast-track governance proposal for emergency payouts
#[account]
#[derive(InitSpace)]
pub struct FastTrackProposal {
    pub pool: Pubkey,
    /// Known hack protocol address being fast-tracked
    pub affected_protocol: Pubkey,
    pub hack_type: HackType,
    pub proposed_by: Pubkey,
    pub approval_count: u8,
    pub rejection_count: u8,
    pub is_active: bool,
    pub expires_at: i64,
    pub bump: u8,
}

