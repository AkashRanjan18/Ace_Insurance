use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::*;
use crate::events::*;
use crate::states::*;

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
    let claim = &mut ctx.accounts.claim_request;
    let user_coverage = &ctx.accounts.user_coverage;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify user has active coverage
    require!(user_coverage.coverage_active, AceError::InactiveCoverage);
    require!(
        user_coverage.user == ctx.accounts.claimant.key(),
        AceError::Unauthorized
    );
    require!(
        user_coverage.pool == pool.key(),
        AceError::InactiveCoverage
    );

    // Validate hack claim amount
    require!(amount_requested > 0, AceError::InvalidCoverageAmount);
    require!(
        amount_requested <= user_coverage.coverage_amount,
        AceError::ExcessiveClaimAmount
    );

    // Validate claim period — must be recent incident
    let time_since_incident = clock.unix_timestamp.saturating_sub(incident_timestamp);
    require!(time_since_incident >0, AceError::ClaimPeriodExpired);
    require!(
        time_since_incident <= pool.claim_period,
        AceError::ClaimPeriodExpired
    );

    // Validate user joined before incident (prevent fraud)
    require!(
        user_coverage.joined_at < incident_timestamp,
        AceError::ClaimPeriodExpired
    );

    // Validate evidence requirements
    require!(!evidence_hash.is_empty(), AceError::EmptyEvidenceHash);
    require!(
        affected_protocol != Pubkey::default(),
        AceError::InvalidProtocolAddress
    );
    require!(
        wallet_balance_before > wallet_balance_after,
        AceError::InvalidWalletLoss
    );

    // Initialize claim request
    let claim_key = claim.key();
    let claimant_key = ctx.accounts.claimant.key();
    let pool_key = pool.key();

    claim.claim_id = claim_key;
    claim.claimant = claimant_key;
    claim.pool = pool_key;
    claim.amount_requested = amount_requested;
    claim.hack_type = hack_type;
    claim.incident_timestamp = incident_timestamp;
    claim.evidence_hash = evidence_hash;
    claim.affected_protocol = affected_protocol;
    claim.wallet_balance_before = wallet_balance_before;
    claim.wallet_balance_after = wallet_balance_after;
    claim.validators_assigned = Vec::new();
    claim.validations = Vec::new();
    claim.approvals = 0;
    claim.rejections = 0;
    claim.auditor_approvals = 0;
    claim.status = ClaimStatus::Pending;
    claim.vrf_result = None;
    claim.created_at = clock.unix_timestamp;
    claim.resolved_at = None;
    claim.payout_amount = None;
    claim.bump = ctx.bumps.claim_request;
    claim.severity = None;
    claim.emergency_payout_received = false;

    // Update pool active claims counter
    pool.active_claims = pool
        .active_claims
        .checked_add(1)
        .ok_or(AceError::MathOverflow)?;

    emit!(ClaimSubmittedEvent {
        claim_id: claim_key,
        claimant: claimant_key,
        pool: pool_key,
        amount_requested,
        hack_type,
        affected_protocol,
        incident_timestamp,
        wallet_balance_before,
        wallet_balance_after,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Hack claim {} submitted | Type: {:?} | Protocol: {} | Loss: {} USDC",
        claim_key,
        hack_type,
        affected_protocol,
        wallet_balance_before.saturating_sub(wallet_balance_after)
    );

    Ok(())
}

/// Request emergency payout (fast-track) — 20% immediate liquidity
pub fn request_emergency_payout(
    ctx: Context<RequestEmergencyPayout>,
) -> Result<()> {
    let claim = &mut ctx.accounts.claim_request;
    let pool = &mut ctx.accounts.pool;
    let queue = &ctx.accounts.distribution_queue;
    let clock = Clock::get()?;

    // Verify fast-track is active for this pool
    require!(pool.fast_track_active, AceError::FastTrackNotActive);

    // Verify claim belongs to this pool and is still pending/under validation
    require!(
        claim.pool == pool.key(),
        AceError::InactiveCoverage
    );
    require!(
        claim.status == ClaimStatus::Pending || claim.status == ClaimStatus::UnderValidation,
        AceError::ClaimPeriodExpired
    );

    // Verify emergency payout not already received
    require!(
        !claim.emergency_payout_received,
        AceError::EmergencyPayoutAlreadyReceived
    );

    // Calculate 20% emergency liquidity payout
    let emergency_amount = claim
        .amount_requested
        .checked_mul(20)
        .ok_or(AceError::MathOverflow)?
        .checked_div(100)
        .ok_or(AceError::MathOverflow)?;

    let remaining_amount = claim.amount_requested.saturating_sub(emergency_amount);

    // Verify pool has sufficient funds
    require!(
        pool.total_pooled >= emergency_amount,
        AceError::InsufficientPoolFunds
    );

    // Transfer USDC from pool vault to claimant
    let pool_key = pool.key();
    let pool_bump = pool.bump;
    let seeds = &[b"vault", pool_key.as_ref(), &[pool_bump]];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.claimant_token_account.to_account_info(),
            authority: ctx.accounts.pool_vault.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, emergency_amount)?;

    // Update state
    claim.emergency_payout_received = true;
    claim.status = ClaimStatus::EmergencyLiquidated;
    pool.total_pooled = pool.total_pooled.saturating_sub(emergency_amount);

    // Resubmit for remaining amount — claim stays active
    // The remaining amount will be processed through normal validation
    claim.amount_requested = remaining_amount;

    emit!(EmergencyPayoutEvent {
        claim_id: claim.key(),
        claimant: claim.claimant,
        pool: pool.key(),
        emergency_amount,
        remaining_amount,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Emergency payout: {} USDC to {} for claim {} | Remaining: {} USDC",
        emergency_amount,
        claim.claimant,
        claim.key(),
        remaining_amount
    );

    Ok(())
}

/// Propose a fast-track for a known hack (governance)
pub fn propose_fast_track(
    ctx: Context<ProposeFastTrack>,
    hack_type: HackType,
    affected_protocol: Pubkey,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    // Only pool authority can propose
    require!(
        ctx.accounts.authority.key() == pool.governance_authority,
        AceError::Unauthorized
    );

    require!(
        affected_protocol != Pubkey::default(),
        AceError::InvalidProtocolAddress
    );

    proposal.pool = pool.key();
    proposal.affected_protocol = affected_protocol;
    proposal.hack_type = hack_type;
    proposal.proposed_by = ctx.accounts.authority.key();
    proposal.approval_count = 1; // Authority auto-approves
    proposal.rejection_count = 0;
    proposal.is_active = true;
    proposal.expires_at = clock.unix_timestamp.saturating_add(86400); // 24 hours expiry
    proposal.bump = ctx.bumps.proposal;

    emit!(FastTrackProposedEvent {
        pool: pool.key(),
        affected_protocol,
        hack_type,
        proposed_by: ctx.accounts.authority.key(),
        expires_at: proposal.expires_at,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Fast-track proposed for protocol {} | Type: {:?} | Expires: {}",
        affected_protocol,
        hack_type,
        proposal.expires_at
    );

    Ok(())
}

/// Approve fast-track proposal (must be auditor-tier validator)
pub fn approve_fast_track(
    ctx: Context<ApproveFastTrack>,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let validator_stake = &ctx.accounts.validator_stake;
    let clock = Clock::get()?;

    // Verify proposal is still active and not expired
    require!(proposal.is_active, AceError::FastTrackNotActive);
    require!(
        clock.unix_timestamp <= proposal.expires_at,
        AceError::FastTrackProposalExpired
    );

    // Only auditor-tier validators can approve fast-track
    require!(
        validator_stake.tier == ValidatorTier::Auditor,
        AceError::AuditorRequiredForFastTrack
    );

    proposal.approval_count = proposal
        .approval_count
        .checked_add(1)
        .ok_or(AceError::MathOverflow)?;

    msg!(
        "Auditor {} approved fast-track proposal | Approvals: {}/5",
        ctx.accounts.validator.key(),
        proposal.approval_count
    );

    Ok(())
}

/// Activate fast-track once enough auditor approvals (5 required)
pub fn activate_fast_track(
    ctx: Context<ActivateFastTrack>,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify proposal is still active
    require!(proposal.is_active, AceError::FastTrackNotActive);
    require!(
        clock.unix_timestamp <= proposal.expires_at,
        AceError::FastTrackProposalExpired
    );

    // Need 5 auditor approvals to activate
    let required_approvals = 5u8;
    require!(
        proposal.approval_count >= required_approvals,
        AceError::InsufficientAuditorApprovals
    );

    // Activate fast-track on the pool
    pool.fast_track_active = true;
    proposal.is_active = false;

    emit!(FastTrackActivatedEvent {
        pool: pool.key(),
        affected_protocol: proposal.affected_protocol,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Fast-track ACTIVATED for pool {} | Protocol: {}",
        pool.key(),
        proposal.affected_protocol
    );

    Ok(())
}

//------------------------------
// Account Validation Contexts
//------------------------------
#[derive(Accounts)]
#[instruction(
    hack_type: HackType,
    claim_amount: u64,
    incident_timestamp: i64
)]
pub struct SubmitClaim<'info> {

#[account(
init,
payer = claimant,
space = 8 + ClaimRequest::INIT_SPACE,
seeds = [
    b"claim",
    claimant.key().as_ref(),
    pool.key().as_ref(),
    &incident_timestamp.to_le_bytes(),
],
bump
)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,

    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        seeds = [b"coverage", claimant.key().as_ref(), pool.key().as_ref()],
        bump = user_coverage.bump,
        constraint = user_coverage.pool == pool.key() @ AceError::InactiveCoverage,
        constraint = user_coverage.user == claimant.key() @ AceError::Unauthorized
    )]
    pub user_coverage: Box<Account<'info, UserCoverage>>,

    #[account(mut)]
    pub claimant: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct RequestEmergencyPayout<'info> {
    #[account(mut)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,

    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        mut,
        seeds = [b"distribution", pool.key().as_ref()],
        bump = distribution_queue.bump,
        constraint = distribution_queue.pool == pool.key() @ AceError::InactiveCoverage
    )]
    pub distribution_queue: Box<Account<'info, DistributionQueue>>,

    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump
    )]
    pub pool_vault: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    #[account(mut)]
    pub claimant_token_account: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    #[account(mut)]
    pub claimant: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ProposeFastTrack<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + FastTrackProposal::INIT_SPACE,
        seeds = [b"fast_track", pool.key().as_ref(), affected_protocol.key().as_ref()],
        bump
    )]
    pub proposal: Box<Account<'info, FastTrackProposal>>,

    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    /// CHECK: The affected protocol address
    pub affected_protocol: AccountInfo<'info>,

    #[account(
        mut,
        constraint = authority.key() == pool.governance_authority @ AceError::Unauthorized
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApproveFastTrack<'info> {
    #[account(mut)]
    pub proposal: Box<Account<'info, FastTrackProposal>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        seeds = [b"validator", validator.key().as_ref(), pool.key().as_ref()],
        bump = validator_stake.bump,
        constraint = validator_stake.validator == validator.key() @ AceError::UnauthorizedValidator,
        constraint = validator_stake.tier == ValidatorTier::Auditor @ AceError::AuditorRequiredForFastTrack
    )]
    pub validator_stake: Box<Account<'info, ValidatorStake>>,

    pub validator: Signer<'info>,
}

#[derive(Accounts)]
pub struct ActivateFastTrack<'info> {
    #[account(mut)]
    pub proposal: Box<Account<'info, FastTrackProposal>>,

    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(mut)]
    pub authority: Signer<'info>,
}