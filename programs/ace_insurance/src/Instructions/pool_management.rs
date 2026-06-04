use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::*;
use crate::events::*;
use crate::states::*;

/// Initialize a new hack insurance pool
pub fn initialize_pool(
    ctx: Context<InitializePool>,
    premium_amount: u64,
    coverage_amount: u64,
    min_validators: u8,
    claim_period: i64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    require!(premium_amount > 0, AceError::InvalidPremiumAmount);
    require!(
        coverage_amount > premium_amount,
        AceError::InvalidCoverageAmount
    );
    require!(min_validators >= 3, AceError::InsufficientValidators);
    require!(claim_period > 0, AceError::InvalidClaimPeriod);

    let pool_key = pool.key();
    let authority_key = ctx.accounts.authority.key();
    let vault_key = ctx.accounts.pool_vault.key();

    pool.pool_id = pool_key;
    pool.authority = authority_key;
    pool.vault = vault_key;
    pool.premium_amount = premium_amount;
    pool.coverage_amount = coverage_amount;
    pool.total_pooled = 0;
    pool.total_members = 0;
    pool.active_claims = 0;
    pool.claim_period = claim_period;
    pool.min_validators = min_validators;
    pool.bump = ctx.bumps.pool;
    pool.governance_authority = authority_key;
    pool.fast_track_active = false;
    pool.created_at = clock.unix_timestamp;

    emit!(PoolCreatedEvent {
        pool_id: pool_key,
        authority: authority_key,
        premium_amount,
        coverage_amount,
        min_validators,
        claim_period,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "ACE Hack Insurance Pool created: {} | Premium: {} USDC | Coverage: {} USDC",
        pool_key,
        premium_amount,
        coverage_amount
    );

    Ok(())
}

/// Join an existing hack insurance pool
pub fn join_pool(
    ctx: Context<JoinPool>,
    coverage_amount: u64,
    uses_multisig: bool,
    uses_hardware_wallet: bool,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let user_coverage = &mut ctx.accounts.user_coverage;
    let clock = Clock::get()?;

    require!(
        coverage_amount <= pool.coverage_amount,
        AceError::ExcessiveCoverageAmount
    );
    require!(coverage_amount > 0, AceError::InvalidCoverageAmount);

    // Transfer premium (USDC) from user to pool vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, pool.premium_amount)?;

    // Calculate OCCR discount (0% if no best practices, up to 20%)
    let occr_discount: u8 = if uses_multisig && uses_hardware_wallet {
        20
    } else if uses_multisig || uses_hardware_wallet {
        10
    } else {
        0
    };

    // Initialize user coverage
    user_coverage.user = ctx.accounts.user.key();
    user_coverage.pool = pool.key();
    user_coverage.premiums_paid = pool.premium_amount;
    user_coverage.last_payment = clock.unix_timestamp;
    user_coverage.coverage_active = true;
    user_coverage.coverage_amount = coverage_amount;
    user_coverage.claims_made = 0;
    user_coverage.joined_at = clock.unix_timestamp;
    user_coverage.bump = ctx.bumps.user_coverage;
    user_coverage.occr_discount = occr_discount;
    user_coverage.uses_multisig = uses_multisig;
    user_coverage.uses_hardware_wallet = uses_hardware_wallet;

    // Update pool stats
    pool.total_pooled = pool
        .total_pooled
        .checked_add(pool.premium_amount)
        .ok_or(AceError::MathOverflow)?;
    pool.total_members = pool
        .total_members
        .checked_add(1)
        .ok_or(AceError::MathOverflow)?;

    emit!(UserJoinedEvent {
        user: ctx.accounts.user.key(),
        pool: pool.key(),
        coverage_amount,
        premium_paid: pool.premium_amount,
        occr_discount,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "User {} joined hack pool {} | Coverage: {} USDC | OCCR Discount: {}%",
        ctx.accounts.user.key(),
        pool.key(),
        coverage_amount,
        occr_discount
    );

    Ok(())
}

/// Pay monthly premium to maintain coverage
pub fn pay_premium(ctx: Context<PayPremium>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let user_coverage = &mut ctx.accounts.user_coverage;
    let clock = Clock::get()?;

    require!(
        user_coverage.user == ctx.accounts.user.key(),
        AceError::Unauthorized
    );

    // Apply OCCR discount if eligible
    let effective_premium = if user_coverage.occr_discount > 0 {
        let discount_amount = (pool.premium_amount as u128)
            .checked_mul(user_coverage.occr_discount as u128)
            .ok_or(AceError::MathOverflow)?
            .checked_div(100)
            .ok_or(AceError::MathOverflow)? as u64;
        pool.premium_amount.saturating_sub(discount_amount)
    } else {
        pool.premium_amount
    };

    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, effective_premium)?;

    user_coverage.premiums_paid = user_coverage
        .premiums_paid
        .checked_add(effective_premium)
        .ok_or(AceError::MathOverflow)?;
    user_coverage.last_payment = clock.unix_timestamp;
    user_coverage.coverage_active = true;

    emit!(PremiumPaidEvent {
        user: ctx.accounts.user.key(),
        pool: pool.key(),
        amount: effective_premium,
        total_paid: user_coverage.premiums_paid,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Premium paid: {} USDC by {} for hack pool {}",
        effective_premium,
        ctx.accounts.user.key(),
        pool.key()
    );

    Ok(())
}

// ----------------------------
// Account Validation Contexts
// ----------------------------

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + InsurancePool::INIT_SPACE,
        seeds = [b"pool", authority.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        init,
        payer = authority,
        token::mint = usdc_mint,
        token::authority = pool,
        seeds = [b"vault", pool.key().as_ref()],
        bump
    )]
    pub pool_vault: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    /// CHECK: USDC mint address, validated by token program
    pub usdc_mint: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct JoinPool<'info> {
    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        init,
        payer = user,
        space = 8 + UserCoverage::INIT_SPACE,
        seeds = [b"coverage", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_coverage: Box<Account<'info, UserCoverage>>,

    #[account(
        mut,
        constraint = pool_vault.key() == pool.vault @ AceError::Unauthorized
    )]
    pub pool_vault: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ AceError::Unauthorized,
        constraint = user_token_account.mint == pool_vault.mint @ AceError::InvalidPremiumAmount
    )]
    pub user_token_account: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PayPremium<'info> {
    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        mut,
        seeds = [b"coverage", user.key().as_ref(), pool.key().as_ref()],
        bump = user_coverage.bump,
        constraint = user_coverage.pool == pool.key() @ AceError::InactiveCoverage
    )]
    pub user_coverage: Box<Account<'info, UserCoverage>>,

    #[account(
        mut,
        constraint = pool_vault.key() == pool.vault @ AceError::Unauthorized
    )]
    pub pool_vault: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ AceError::Unauthorized,
        constraint = user_token_account.mint == pool_vault.mint @ AceError::InvalidPremiumAmount
    )]
    pub user_token_account: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}