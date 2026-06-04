use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::errors::*;

/// Deposit idle pool funds to yield vault
/// Placeholder for Kamino Finance integration
pub fn deposit_to_yield(ctx: Context<DepositToYield>, _amount: u64) -> Result<()> {
    msg!("Deposit to yield vault — not yet implemented, requires Kamino integration");
    // In production: CPI call to Kamino vault deposit
    Ok(())
}

/// Withdraw funds from yield vault back to pool
/// Placeholder for Kamino Finance integration
pub fn withdraw_from_yield(ctx: Context<WithdrawFromYield>, _amount: u64) -> Result<()> {
    msg!("Withdraw from yield vault — not yet implemented, requires Kamino integration");
    // In production: CPI call to Kamino vault withdraw
    Ok(())
}

// ----------------------------
// Account Validation Contexts
// ----------------------------

#[derive(Accounts)]
pub struct DepositToYield<'info> {
    #[account(mut)]
    pub pool: Box<Account<'info, crate::states::InsurancePool>>,

    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump
    )]
    pub pool_vault: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    /// CHECK: Yield vault address (Kamino)
    pub yield_vault: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawFromYield<'info> {
    #[account(mut)]
    pub pool: Box<Account<'info, crate::states::InsurancePool>>,

    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump
    )]
    pub pool_vault: Box<InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>>,

    /// CHECK: Yield vault address (Kamino)
    pub yield_vault: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}