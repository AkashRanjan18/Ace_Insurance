use anchor_lang::prelude::*;

use crate::errors::*;
use crate::events::*;
use crate::states::*;

/// Calculate and apply OCCR discount for a user
/// OCCR = On-Chain Coverage Risk Rating
/// Users get discount based on security best practices
pub fn calculate_occr_discount(
    ctx: Context<CalculateOccrDiscount>,
    uses_multisig: bool,
    uses_hardware_wallet: bool,
) -> Result<()> {
    let user_coverage = &mut ctx.accounts.user_coverage;
    let clock = Clock::get()?;

    require!(
        user_coverage.user == ctx.accounts.user.key(),
        AceError::Unauthorized
    );

    // Calculate OCCR discount:
    // - Multisig only: 10%
    // - Hardware wallet only: 10%
    // - Both: 20%
    // - Neither: 0%
    let discount: u8 = if uses_multisig && uses_hardware_wallet {
        20
    } else if uses_multisig || uses_hardware_wallet {
        10
    } else {
        0
    };

    // Apply discount
    user_coverage.occr_discount = discount;
    user_coverage.uses_multisig = uses_multisig;
    user_coverage.uses_hardware_wallet = uses_hardware_wallet;

    emit!(OccrDiscountAppliedEvent {
        user: ctx.accounts.user.key(),
        pool: ctx.accounts.pool.key(),
        discount_percent: discount,
        uses_multisig,
        uses_hardware_wallet,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "OCCR discount applied for user {}: {}% off premium",
        ctx.accounts.user.key(),
        discount
    );

    Ok(())
}

/// Assess hack severity for a claim
/// Called by validators or authority after investigation
pub fn assess_hack_severity(
    ctx: Context<AssessHackSeverity>,
    severity: SeverityTier,
) -> Result<()> {
    let claim = &mut ctx.accounts.claim_request;
    let clock = Clock::get()?;

    // Only authority or assigned validators can assess severity
    let caller = ctx.accounts.authority.key();
    let is_authority = caller == ctx.accounts.pool.authority;
    let is_assigned_validator = claim.validators_assigned.contains(&caller);

    require!(
        is_authority || is_assigned_validator,
        AceError::Unauthorized
    );

    claim.severity = Some(severity);

    msg!(
        "Hack claim {} severity assessed as {:?}",
        claim.claim_id,
        severity
    );

    Ok(())
}

// ---------------------------
// Account Validation Contexts
// ---------------------------

#[derive(Accounts)]
pub struct CalculateOccrDiscount<'info> {
    #[account(
        mut,
        seeds = [b"coverage", user.key().as_ref(), pool.key().as_ref()],
        bump = user_coverage.bump,
        constraint = user_coverage.user == user.key() @ AceError::Unauthorized,
        constraint = user_coverage.pool == pool.key() @ AceError::InactiveCoverage
    )]
    pub user_coverage: Box<Account<'info, UserCoverage>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AssessHackSeverity<'info> {
    #[account(mut)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(mut)]
    pub authority: Signer<'info>,
}