use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::*;
use crate::events::*;
use crate::states::*;

/// Initialize distribution queue for a pool
pub fn initialize_distribution_queue(
    ctx: Context<InitializeDistributionQueue>,
) -> Result<()> {
    let queue = &mut ctx.accounts.distribution_queue;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    queue.pool = pool.key();
    queue.total_approved_claims = 0;
    queue.total_requested_amount = 0;
    queue.available_funds = pool.total_pooled;
    queue.pending_claims = Vec::new();
    queue.selected_claims = Vec::new();
    queue.vrf_result = None;
    queue.is_oversubscribed = false;
    queue.distribution_round = 0;  
    queue.last_distribution = clock.unix_timestamp;
    queue.bump = ctx.bumps.distribution_queue;

    emit!(DistributionQueueInitializedEvent {
        pool: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Distribution queue initialized for hack pool {}", pool.key());
    Ok(())
}

/// Add approved claim to distribution queue
pub fn add_to_distribution_queue(
    ctx: Context<AddToDistributionQueue>,
) -> Result<()> {
    let queue = &mut ctx.accounts.distribution_queue;
    let claim = &ctx.accounts.claim_request;

    require!(
        claim.status == ClaimStatus::Approved,
        AceError::InactiveCoverage
    );
    require!(
        !queue.pending_claims.contains(&claim.key()),
        AceError::DuplicateValidation
    );

    queue.pending_claims.push(claim.key());
    queue.total_approved_claims = queue
        .total_approved_claims
        .checked_add(1)
        .ok_or(AceError::MathOverflow)?;
    queue.total_requested_amount = queue
        .total_requested_amount
        .checked_add(claim.amount_requested)
        .ok_or(AceError::MathOverflow)?;

    msg!(
        "Hack claim {} added to distribution queue | Total: {} claims, {} USDC",
        claim.key(),
        queue.total_approved_claims,
        queue.total_requested_amount
    );

    Ok(())
}

/// Distribute claims — normal or oversubscribed
pub fn distribute_claims(
    ctx: Context<DistributeClaims>,
    randomness: Option<[u8; 32]>,
) -> Result<()> {
    let queue = &mut ctx.accounts.distribution_queue;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    queue.available_funds = pool.total_pooled;

    let is_oversubscribed = queue.total_requested_amount > queue.available_funds;
    queue.is_oversubscribed = is_oversubscribed;

    if !is_oversubscribed {
        // Normal case: pay all approved claims
        queue.selected_claims = queue.pending_claims.clone();
        msg!(
            "Normal distribution: {} claims, {} USDC available, {} USDC requested",
            queue.pending_claims.len(),
            queue.available_funds,
            queue.total_requested_amount
        );
    } else {
        // Oversubscribed: use VRF for fair random selection
        require!(randomness.is_some(), AceError::InvalidTimestamp);
        let random_bytes = randomness.unwrap();
        queue.vrf_result = Some(random_bytes);

        queue.selected_claims.clear();
        let mut remaining_funds = queue.available_funds;
        let mut selected_indices = Vec::new();
        let total_claims = queue.pending_claims.len();

        for i in 0..total_claims {
            if remaining_funds == 0 {
                break;
            }

            let random_offset = i % 8;
            let random_bytes_subset = [
                random_bytes[random_offset * 4],
                random_bytes[random_offset * 4 + 1],
                random_bytes[random_offset * 4 + 2],
                random_bytes[random_offset * 4 + 3],
            ];
            let random_value = u32::from_le_bytes(random_bytes_subset);

            let mut attempts = 0;
            loop {
                let index = ((random_value as usize + attempts) % total_claims) as usize;
                if !selected_indices.contains(&index) {
                    selected_indices.push(index);
                    let claim_key = queue.pending_claims[index];
                    let avg_claim_size = queue.total_requested_amount / total_claims as u64;

                    if remaining_funds >= avg_claim_size {
                        queue.selected_claims.push(claim_key);
                        remaining_funds = remaining_funds.saturating_sub(avg_claim_size);
                    }
                    break;
                }
                attempts += 1;
                if attempts >= total_claims {
                    break;
                }
            }
        }

        msg!(
            "Oversubscribed: Selected {} out of {} claims for payment",
            queue.selected_claims.len(),
            queue.pending_claims.len()
        );
    }

    queue.distribution_round = queue
        .distribution_round
        .checked_add(1)
        .ok_or(AceError::MathOverflow)?;
    queue.last_distribution = clock.unix_timestamp;

    emit!(ClaimsDistributedEvent {
        pool: pool.key(),
        round: queue.distribution_round,
        total_claims: queue.pending_claims.len() as u32,
        selected_claims: queue.selected_claims.len() as u32,
        oversubscribed: is_oversubscribed,
        available_funds: queue.available_funds,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Payout individual claim
pub fn payout_claim(ctx: Context<PayoutClaim>) -> Result<()> {
    let claim = &mut ctx.accounts.claim_request;
    let pool = &mut ctx.accounts.pool;
    let queue = &mut ctx.accounts.distribution_queue;
    let clock = Clock::get()?;

    require!(
        claim.status == ClaimStatus::Approved,
        AceError::InactiveCoverage
    );
    require!(
        queue.selected_claims.contains(&claim.key()),
        AceError::UnauthorizedValidator
    );

    let payout_amount = claim.amount_requested.min(claim.payout_amount.unwrap_or(claim.amount_requested));
    require!(
        pool.total_pooled >= payout_amount,
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
    token::transfer(transfer_ctx, payout_amount)?;

    // Update state
    pool.total_pooled = pool.total_pooled.saturating_sub(payout_amount);
    pool.active_claims = pool.active_claims.saturating_sub(1);

    claim.status = ClaimStatus::Distributed;
    claim.resolved_at = Some(clock.unix_timestamp);
    claim.payout_amount = Some(payout_amount);

    // Remove from distribution queue
    if let Some(pos) = queue.pending_claims.iter().position(|&c| c == claim.key()) {
        queue.pending_claims.remove(pos);
    }
    if let Some(pos) = queue.selected_claims.iter().position(|&c| c == claim.key()) {
        queue.selected_claims.remove(pos);
    }

    queue.total_approved_claims = queue.total_approved_claims.saturating_sub(1);
    queue.total_requested_amount = queue.total_requested_amount.saturating_sub(payout_amount);

    emit!(ClaimPaidOutEvent {
        claim_id: claim.key(),
        claimant: claim.claimant,
        pool: pool.key(),
        amount: payout_amount,
        is_emergency: false,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Hack claim {} paid out {} USDC to {}",
        claim.key(),
        payout_amount,
        claim.claimant
    );

    Ok(())
}
//-----------------------------
// Account Validation Contexts
//-----------------------------

#[derive(Accounts)]
pub struct InitializeDistributionQueue<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + DistributionQueue::INIT_SPACE,
        seeds = [b"distribution", pool.key().as_ref()],
        bump
    )]
    pub distribution_queue: Box<Account<'info, DistributionQueue>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddToDistributionQueue<'info> {
    #[account(
        mut,
        seeds = [b"distribution", pool.key().as_ref()],
        bump = distribution_queue.bump
    )]
    pub distribution_queue: Box<Account<'info, DistributionQueue>>,

    pub claim_request: Box<Account<'info, ClaimRequest>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeClaims<'info> {
    #[account(
        mut,
        seeds = [b"distribution", pool.key().as_ref()],
        bump = distribution_queue.bump
    )]
    pub distribution_queue: Box<Account<'info, DistributionQueue>>,

    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PayoutClaim<'info> {
    #[account(mut)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,

    #[account(mut)]
    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        mut,
        seeds = [b"distribution", pool.key().as_ref()],
        bump = distribution_queue.bump
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
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}