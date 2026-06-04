use anchor_lang::prelude::*;

declare_id!("4LP2JnLzuPhTCLR6MzPEr2Cr2zU3om7Sk33G5QaeKXRU");

pub mod errors;
pub mod events;
pub mod states;
pub mod Instructions;

#[allow(unused_imports)]
use errors::*;
#[allow(unused_imports)]
use events::*;
#[allow(unused_imports)]
use states::*;
use Instructions::*;

#[program]
pub mod ace_insurance {
    use super::*;

    // ----------------------------------------------------------------
    // POOL MANAGEMENT
    // ----------------------------------------------------------------

    /// Create a new hack insurance pool
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        premium_amount: u64,
        coverage_amount: u64,
        min_validators: u8,
        claim_period: i64,
    ) -> Result<()> {
        Instructions::pool_management::initialize_pool(
            ctx, premium_amount, coverage_amount, min_validators, claim_period,
        )
    }

    /// Join an existing hack insurance pool
    pub fn join_pool(
        ctx: Context<JoinPool>,
        coverage_amount: u64,
        uses_multisig: bool,
        uses_hardware_wallet: bool,
    ) -> Result<()> {
        Instructions::pool_management::join_pool(ctx, coverage_amount, uses_multisig, uses_hardware_wallet)
    }

    /// Pay monthly premium to maintain coverage
    pub fn pay_premium(ctx: Context<PayPremium>) -> Result<()> {
        Instructions::pool_management::pay_premium(ctx)
    }

    // ----------------------------------------------------------------
    // CLAIMS MANAGEMENT
    // ----------------------------------------------------------------

    /// Submit a hack claim with on-chain evidence
    pub fn submit_claim(
        ctx: Context<SubmitClaim>,
        hack_type: HackType,
        amount_requested: u64,
        incident_timestamp: i64,
        evidence_hash: String,
        affected_protocol: Pubkey,
        wallet_balance_before: u64,
        wallet_balance_after: u64,
    ) -> Result<()> {
        Instructions::claims_management::submit_claim(
            ctx, hack_type, amount_requested, incident_timestamp,
            evidence_hash, affected_protocol,
            wallet_balance_before, wallet_balance_after,
        )
    }

    /// Request emergency payout (fast-track) for a claim during an active fast-track event
    pub fn request_emergency_payout(
        ctx: Context<RequestEmergencyPayout>,
    ) -> Result<()> {
        Instructions::claims_management::request_emergency_payout(ctx)
    }

    // ----------------------------------------------------------------
    // VALIDATORS MANAGEMENT
    // ----------------------------------------------------------------

    /// Initialize validator stake for a pool
    pub fn initialize_validator_stake(
        ctx: Context<InitializeValidatorStake>,
    ) -> Result<()> {
        Instructions::validators_management::initialize_validator_stake(ctx)
    }

    /// Stake SOL to become a validator
    pub fn stake_as_validator(
        ctx: Context<StakeAsValidator>,
        stake_amount: u64,
    ) -> Result<()> {
        Instructions::validators_management::stake_as_validator(ctx, stake_amount)
    }

    /// Validate a claim (approve or reject) with technical analysis
    pub fn validate_claim(
        ctx: Context<ValidateClaim>,
        approve: bool,
        reason: String,
        technical_analysis: String,
    ) -> Result<()> {
        Instructions::validators_management::validate_claim(ctx, approve, reason, technical_analysis)
    }

    /// Promote a validator to Auditor tier (callable by pool authority)
    pub fn promote_to_auditor(
        ctx: Context<PromoteToAuditor>,
    ) -> Result<()> {
        Instructions::validators_management::promote_to_auditor(ctx)
    }

    // ----------------------------------------------------------------
    // VRF INTEGRATION
    // ----------------------------------------------------------------

    /// Initialize VRF state for a pool
    pub fn initialize_vrf_state(ctx: Context<InitializeVrfState>) -> Result<()> {
        Instructions::vrf_integration::initialize_vrf_state(ctx)
    }

    /// Request validator selection using VRF for a claim
    pub fn request_validator_selection(
        ctx: Context<RequestValidatorSelection>,
        claim_id: Pubkey,
    ) -> Result<()> {
        Instructions::vrf_integration::request_validator_selection(ctx, claim_id)
    }

    // ----------------------------------------------------------------
    // DISTRIBUTION
    // ----------------------------------------------------------------

    /// Initialize distribution queue for a pool
    pub fn initialize_distribution_queue(
        ctx: Context<InitializeDistributionQueue>,
    ) -> Result<()> {
        Instructions::distribution::initialize_distribution_queue(ctx)
    }

    /// Add approved claim to distribution queue
    pub fn add_to_distribution_queue(
        ctx: Context<AddToDistributionQueue>,
    ) -> Result<()> {
        Instructions::distribution::add_to_distribution_queue(ctx)
    }

    /// Distribute claims (normal or oversubscribed)
    pub fn distribute_claims(
        ctx: Context<DistributeClaims>,
        randomness: Option<[u8; 32]>,
    ) -> Result<()> {
        Instructions::distribution::distribute_claims(ctx, randomness)
    }

    /// Payout individual claim
    pub fn payout_claim(ctx: Context<PayoutClaim>) -> Result<()> {
        Instructions::distribution::payout_claim(ctx)
    }

    // ----------------------------------------------------------------
    // FAST-TRACK (GOVERNANCE)
    // ----------------------------------------------------------------

    /// Propose a fast-track for a known hack (governance)
    pub fn propose_fast_track(
        ctx: Context<ProposeFastTrack>,
        hack_type: HackType,
        affected_protocol: Pubkey,
    ) -> Result<()> {
        Instructions::claims_management::propose_fast_track(ctx, hack_type, affected_protocol)
    }

    /// Approve fast-track proposal (must be auditor)
    pub fn approve_fast_track(
        ctx: Context<ApproveFastTrack>,
    ) -> Result<()> {
        Instructions::claims_management::approve_fast_track(ctx)
    }

    /// Activate fast-track once enough approvals
    pub fn activate_fast_track(
        ctx: Context<ActivateFastTrack>,
    ) -> Result<()> {
        Instructions::claims_management::activate_fast_track(ctx)
    }

    // ----------------------------------------------------------------
    // RISK SCORE (OCCR)
    // ----------------------------------------------------------------

    /// Calculate and apply OCCR discount for a user
    pub fn calculate_occr_discount(
        ctx: Context<CalculateOccrDiscount>,
        uses_multisig: bool,
        uses_hardware_wallet: bool,
    ) -> Result<()> {
        Instructions::risk_score::calculate_occr_discount(ctx, uses_multisig, uses_hardware_wallet)
    }

    /// Assess hack severity for a claim
    pub fn assess_hack_severity(
        ctx: Context<AssessHackSeverity>,
        severity: SeverityTier,
    ) -> Result<()> {
        Instructions::risk_score::assess_hack_severity(ctx, severity)
    }

    // ----------------------------------------------------------------
    // YIELD GENERATION
    // ----------------------------------------------------------------

    /// Deposit idle pool funds to yield vault
    pub fn deposit_to_yield(ctx: Context<DepositToYield>, amount: u64) -> Result<()> {
        Instructions::yield_generation::deposit_to_yield(ctx, amount)
    }

    /// Withdraw funds from yield vault back to pool
    pub fn withdraw_from_yield(ctx: Context<WithdrawFromYield>, amount: u64) -> Result<()> {
        Instructions::yield_generation::withdraw_from_yield(ctx, amount)
    }
}