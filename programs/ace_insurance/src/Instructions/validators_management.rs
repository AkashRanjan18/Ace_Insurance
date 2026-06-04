use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer as SystemTransfer};
use crate::errors::*;
use crate::events::*;
use crate::states::*;

/// Initialize validator stake pool for a pool
pub fn initialize_validator_stake(ctx: Context<InitializeValidatorStake>) -> Result<()> {
    let validator_stake_pool = &mut ctx.accounts.validator_stake_pool;
    let pool = &ctx.accounts.pool;
    validator_stake_pool.pool = pool.key();
    validator_stake_pool.validators = Vec::new();
    validator_stake_pool.total_validators = 0;
    validator_stake_pool.bump = ctx.bumps.validator_stake_pool;
    msg!("Validator stake initialized for hack pool {}", pool.key());
    Ok(())
}

/// Stake SOL to become a validator
pub fn stake_as_validator(ctx: Context<StakeAsValidator>, stake_amount: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;
    const MIN_STAKE: u64 = 100_000_000; // 0.1 SOL in lamports
    require!(stake_amount >= MIN_STAKE, AceError::InsufficientStake);
    let validator_key = ctx.accounts.validator.key();

    let transfer_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        SystemTransfer {
            from: ctx.accounts.validator.to_account_info(),
            to: ctx.accounts.validator_stake.to_account_info(),
        },
    );
    transfer(transfer_ctx, stake_amount)?;

    let validator_stake = &mut ctx.accounts.validator_stake;
    validator_stake.validator = validator_key;
    validator_stake.stake_amount = stake_amount;
    validator_stake.validations_completed = 0;
    validator_stake.successful_validations = 0;
    validator_stake.reputation_score = ValidatorStake::INITIAL_REPUTATION;
    validator_stake.last_validation = 0;
    validator_stake.bump = ctx.bumps.validator_stake;
    validator_stake.tier = ValidatorTier::Standard;
    validator_stake.vote_weight = 1;

    let validator_stake_pool = &mut ctx.accounts.validator_stake_pool;
    if !validator_stake_pool.validators.contains(&validator_key) {
        // Safe fixed literal rule bounds to prevent unallocated vector panic
        require!(validator_stake_pool.validators.len() < 32, AceError::InsufficientValidators);
        validator_stake_pool.validators.push(validator_key);
        validator_stake_pool.total_validators = validator_stake_pool.total_validators.checked_add(1).ok_or(AceError::MathOverflow)?;
    }

    emit!(ValidatorStakedEvent {
        validator: validator_key,
        pool: pool.key(),
        stake_amount,
        reputation_score: ValidatorStake::INITIAL_REPUTATION,
        tier: ValidatorTier::Standard,
        timestamp: clock.unix_timestamp,
    });
    Ok(())
}

/// Validate a claim (approve or reject) with technical analysis
pub fn validate_claim(ctx: Context<ValidateClaim>, approve: bool, reason: String, technical_analysis: String) -> Result<()> {
    let claim = &mut ctx.accounts.claim_request;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    require!(claim.status == ClaimStatus::UnderValidation || claim.status == ClaimStatus::Pending, AceError::ClaimPeriodExpired);
    let validator_key = ctx.accounts.validator.key();
    require!(claim.validators_assigned.contains(&validator_key), AceError::UnauthorizedValidator);

    let already_validated = claim.validations.iter().any(|v| v.validator == validator_key);
    require!(!already_validated, AceError::DuplicateValidation);
    require!(reason.len() <= 200, AceError::StringTooLong);
    require!(technical_analysis.len() <= 500, AceError::StringTooLong);

    claim.validations.push(Validation {
        validator: validator_key,
        approved: approve,
        reason: reason.clone(),
        timestamp: clock.unix_timestamp,
        technical_analysis: technical_analysis.clone(),
    });
    ctx.accounts.validator_stake.validations_completed = ctx.accounts.validator_stake.validations_completed.checked_add(1).ok_or(AceError::MathOverflow)?;
ctx.accounts.validator_stake.last_validation = clock.unix_timestamp;

    let validator_stake = &ctx.accounts.validator_stake;
    let vote_weight = validator_stake.vote_weight as u8;

    if approve {
        claim.approvals = claim.approvals.checked_add(vote_weight).ok_or(AceError::MathOverflow)?;
        if validator_stake.tier == ValidatorTier::Auditor {
            claim.auditor_approvals = claim.auditor_approvals.checked_add(1).ok_or(AceError::MathOverflow)?;
        }
    } else {
        claim.rejections = claim.rejections.checked_add(vote_weight).ok_or(AceError::MathOverflow)?;
    }

    let total_validations = claim.approvals.checked_add(claim.rejections).ok_or(AceError::MathOverflow)?;
    let required_validations = claim.validators_assigned.len() as u8;
    let is_finalized = total_validations >= required_validations;
    let majority_threshold = (required_validations / 2) + 1;
    let is_approved = claim.approvals >= majority_threshold;

    if is_finalized {
        if is_approved {
            claim.status = ClaimStatus::Approved;
            claim.resolved_at = Some(clock.unix_timestamp);
            claim.payout_amount = Some(claim.amount_requested);
            msg!("Hack claim {} APPROVED", claim.claim_id);
        } else {
            claim.status = ClaimStatus::Rejected;
            claim.resolved_at = Some(clock.unix_timestamp);
            msg!("Hack claim {} REJECTED", claim.claim_id);
        }

        let voted_with_majority = (is_approved && approve) || (!is_approved && !approve);
        update_validator_reputation(&mut ctx.accounts.validator_stake, voted_with_majority, pool)?;

       if !approve && !is_approved {
    // If the consensus rejected it, look if the honest validators flagged it as fraud
    let fraud_flagged = claim.validations.iter().any(|v| !v.approved && (v.reason.contains("self-hack") || v.reason.contains("fraud")));
    
    if fraud_flagged {
        slash_validator_full(&mut ctx.accounts.validator_stake)?;
    }
}
    } else {
        claim.status = ClaimStatus::UnderValidation;
        
    }

    emit!(ClaimValidatedEvent {
        claim_id: claim.claim_id,
        validator: validator_key,
        approved: approve,
        claim_status: claim.status,
        approvals: claim.approvals,
        rejections: claim.rejections,
        auditor_approvals: claim.auditor_approvals,
        timestamp: clock.unix_timestamp,
    });
    Ok(())
}

/// Promote a validator to Auditor tier (callable by pool authority)
pub fn promote_to_auditor(ctx: Context<PromoteToAuditor>) -> Result<()> {
    let validator_stake = &mut ctx.accounts.validator_stake;
    let pool = &ctx.accounts.pool;

    require!(ctx.accounts.authority.key() == pool.authority, AceError::Unauthorized);
    require!(validator_stake.reputation_score >= ValidatorStake::AUDITOR_REPUTATION_THRESHOLD, AceError::LowReputation);

    validator_stake.tier = ValidatorTier::Auditor;
    validator_stake.vote_weight = 3;

    emit!(ValidatorPromotedToAuditorEvent {
        validator: ctx.accounts.validator.key(),
        pool: pool.key(),
        reputation_score: validator_stake.reputation_score,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

/// Update validator reputation based on voting outcome

fn update_validator_reputation(validator_stake: &mut ValidatorStake, voted_with_majority: bool, _pool: &InsurancePool) -> Result<()> {
    if voted_with_majority {
        validator_stake.successful_validations = validator_stake.successful_validations.checked_add(1).ok_or(AceError::MathOverflow)?;
        validator_stake.reputation_score = validator_stake.reputation_score.saturating_add(100).min(ValidatorStake::MAX_REPUTATION);
        msg!("Validator {} rewarded: +100 reputation", validator_stake.validator);
    } else {
        let slash_amount = (validator_stake.stake_amount as u128)
            .checked_mul(6u128)
            .ok_or(AceError::MathOverflow)?
            .checked_div(100)
            .ok_or(AceError::MathOverflow)? as u64;

        validator_stake.reputation_score = validator_stake.reputation_score.saturating_sub(200);
        validator_stake.stake_amount = validator_stake.stake_amount.saturating_sub(slash_amount);
        msg!("Validator {} slashed {} lamports and -200 reputation", validator_stake.validator, slash_amount);
    }
    Ok(())
}

/// Full slashing for confirmed fraud (self-hack approval)
fn slash_validator_full(validator_stake: &mut ValidatorStake) -> Result<()> {
    validator_stake.reputation_score = 0;
    validator_stake.stake_amount = 0;
    msg!("Validator {} FULLY SLASHED for fraud | All stake burned", validator_stake.validator);
    Ok(())
}


// ---------------------------
// Account Validation Contexts
// ---------------------------

#[derive(Accounts)]
pub struct InitializeValidatorStake<'info> {
    #[account(init, payer = authority, space = 8 + ValidatorStakePool::INIT_SPACE, seeds = [b"validator_stake", pool.key().as_ref()], bump)]
    pub validator_stake_pool: Box<Account<'info, ValidatorStakePool>>,
    pub pool: Box<Account<'info, InsurancePool>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeAsValidator<'info> {
    #[account(init, payer = validator, space = 8 + ValidatorStake::INIT_SPACE, seeds = [b"validator", validator.key().as_ref(), pool.key().as_ref()], bump)]
    pub validator_stake: Box<Account<'info, ValidatorStake>>,
    #[account(mut, seeds = [b"validator_stake", pool.key().as_ref()], bump = validator_stake_pool.bump)]
    pub validator_stake_pool: Box<Account<'info, ValidatorStakePool>>,
    pub pool: Box<Account<'info, InsurancePool>>,
    #[account(mut)]
    pub validator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ValidateClaim<'info> {
    #[account(mut,
    realloc = 8 + claim_request.to_account_info().data_len() + Validation::INIT_SPACE,
        realloc::payer = validator,
        realloc::zero = false,)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,
    #[account(mut, seeds = [b"validator", validator.key().as_ref(), pool.key().as_ref()], bump = validator_stake.bump, constraint = validator_stake.validator == validator.key() @ AceError::UnauthorizedValidator)]
    pub validator_stake: Box<Account<'info, ValidatorStake>>,
    pub pool: Box<Account<'info, InsurancePool>>,
    #[account(mut)]
    pub validator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PromoteToAuditor<'info> {
    #[account(mut, seeds = [b"validator", validator.key().as_ref(), pool.key().as_ref()], bump = validator_stake.bump, constraint = validator_stake.validator == validator.key() @ AceError::UnauthorizedValidator)]
    pub validator_stake: Box<Account<'info, ValidatorStake>>,
    pub pool: Box<Account<'info, InsurancePool>>,
    #[account(mut, constraint = authority.key() == pool.authority @ AceError::Unauthorized)]
    pub authority: Signer<'info>,
    /// CHECK: Validator to promote
    pub validator: AccountInfo<'info>,
}
