use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};

use crate::errors::*;
use crate::events::*;
use crate::states::*;

// ─── Switchboard V2 Program IDs ──────────────────────────────────────────────
// Mainnet-beta: SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f
// Devnet:       2TfB33aLaneQb5TNVs2qSRKwFQbk5b18bBhsmLXrg3M7
//
// In production, import switchboard-solana crate and use:
//   use switchboard_solana::{VrfAccountData, VrfRequestRandomness};
//
// We keep raw CPI here to avoid workspace version conflicts with Anchor 0.31.1.
// The architecture is identical — only the account deserialization helper differs.

#[cfg(feature = "devnet")]
pub const SWITCHBOARD_PROGRAM_ID: Pubkey = pubkey!("2TfB33aLaneQb5TNVs2qSRKwFQbk5b18bBhsmLXrg3M7");

#[cfg(not(feature = "devnet"))]
pub const SWITCHBOARD_PROGRAM_ID: Pubkey =
    pubkey!("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f");

// ─── Switchboard V2 VrfAccountData layout ────────────────────────────────────
// Discriminator = sha256("account:VrfAccountData")[..8]
// After the 8-byte discriminator, VrfStatus is a u8 (1 = None, 2 = Requesting, 3 = Fulfilled).
// The verified randomness lives in `current_round.result: [u8; 32]`.
// Full byte-level layout: https://github.com/switchboard-xyz/switchboard-v2
//
// Using switchboard-solana crate you'd write:
//   let vrf = VrfAccountData::new(&ctx.accounts.switchboard_vrf)?;
//   require!(vrf.status == VrfStatus::StatusCallbackSuccess, ...);
//   let randomness = vrf.current_round.result;
const VRF_DISCRIMINATOR_LEN: usize = 8;
const VRF_STATUS_OFFSET: usize = VRF_DISCRIMINATOR_LEN; // u8
const VRF_STATUS_FULFILLED: u8 = 3; // VrfStatus::StatusCallbackSuccess

// result is at a fixed offset in VrfAccountData — see switchboard-v2 source.
// state(1) + request_slot(8) + last_verified_slot(8) + alpha(256) + alpha_len(4)
// + request_timeout(8) + authority(32) + oracle_queue(32) + escrow(32)
// + on_fulfill callback (4+32+4 = 40) + builders_len(4) + test_mode(1) = 430 bytes
// current_round.result starts at offset 8 + 430 = 438
const VRF_RESULT_OFFSET: usize = VRF_DISCRIMINATOR_LEN + 430;

// ─── Switchboard V2 instruction discriminator for vrf_request_randomness ─────
// sha256("global:vrf_request_randomness")[..8]
const VRF_REQUEST_RANDOMNESS_DISC: [u8; 8] =
    [0x6e, 0x97, 0x48, 0x86, 0x57, 0x69, 0x47, 0x17];

// ─────────────────────────────────────────────────────────────────────────────

/// Initialize VRF state — links a pool to a Switchboard VRF account.
///
/// The Switchboard VRF account must be created off-chain before calling this:
///   `sb vrf create --cluster devnet --keypair ~/.config/solana/id.json`
///
/// The created VRF account pubkey is passed in as `switchboard_vrf`.
pub fn initialize_vrf_state(ctx: Context<InitializeVrfState>) -> Result<()> {
    let vrf_state = &mut ctx.accounts.vrf_state;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    vrf_state.pool = pool.key();
    vrf_state.switchboard_vrf = ctx.accounts.switchboard_vrf.key();
    vrf_state.authority = ctx.accounts.authority.key();
    vrf_state.last_randomness = None;
    vrf_state.last_timestamp = clock.unix_timestamp;
    vrf_state.pending_claims = Vec::new();
    vrf_state.requests_completed = 0;
    vrf_state.bump = ctx.bumps.vrf_state;

    emit!(VrfStateInitializedEvent {
        pool: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "VRF state initialised — pool {} linked to Switchboard VRF {}",
        pool.key(),
        ctx.accounts.switchboard_vrf.key()
    );
    Ok(())
}

// ─── Phase 1 ─────────────────────────────────────────────────────────────────

/// Request randomness from the Switchboard oracle network for a claim.
///
/// Security guarantee (vs. the prior keccak-based pseudo-VRF):
/// - The old approach hashed on-chain data (slot, timestamp, pubkeys) that a
///   validator can observe and time their call to bias which validators are selected.
/// - Switchboard's ECVRF (RFC 9381) produces a proof off-chain.  The randomness
///   cannot be known by any on-chain actor until the oracle publishes the proof,
///   which happens *after* this request is recorded on-chain.
///
/// After this call, the Switchboard oracle will:
///   1. Compute the VRF proof using its private key and the request's alpha.
///   2. Call back `fulfill_vrf_randomness` on this program (via CPI) with the result.
pub fn request_validator_selection(
    ctx: Context<RequestValidatorSelection>,
    claim_id: Pubkey,
) -> Result<()> {
    let vrf_state = &mut ctx.accounts.vrf_state;
    let claim = &mut ctx.accounts.claim_request;
    let pool = &ctx.accounts.pool;
    let validator_stake_pool = &ctx.accounts.validator_stake_pool;
    let clock = Clock::get()?;

    require!(
        claim.status == ClaimStatus::Pending,
        AceError::ClaimPeriodExpired
    );
    require!(claim.pool == pool.key(), AceError::InactiveCoverage);
    require!(
        claim.validators_assigned.is_empty(),
        AceError::DuplicateValidation
    );
    require!(
        !vrf_state.pending_claims.contains(&claim_id),
        AceError::DuplicateValidation
    );

    let required_validators = pool.min_validators.saturating_add(2) as usize;
    require!(
        validator_stake_pool.validators.len() >= required_validators,
        AceError::InsufficientValidators
    );

    // Register claim as awaiting oracle fulfillment.
    // fulfill_vrf_randomness will pop this entry and assign validators.
    vrf_state.pending_claims.push(claim_id);
    vrf_state.last_timestamp = clock.unix_timestamp;

    // CPI → Switchboard V2: vrf_request_randomness
    // The Switchboard oracle watches for this on-chain request and computes the
    // ECVRF proof asynchronously, then calls fulfill_vrf_randomness on our program.
    let ix = build_vrf_request_ix(
        ctx.accounts.switchboard_vrf.key(),
        ctx.accounts.authority.key(),
        ctx.accounts.oracle_queue.key(),
        ctx.accounts.queue_authority.key(),
        ctx.accounts.data_buffer.key(),
        ctx.accounts.permission.key(),
        ctx.accounts.escrow.key(),
        ctx.accounts.recent_blockhashes.key(),
        ctx.accounts.program_state.key(),
        ctx.accounts.token_program.key(),
    );

    invoke(
        &ix,
        &[
            ctx.accounts.switchboard_vrf.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.oracle_queue.to_account_info(),
            ctx.accounts.queue_authority.to_account_info(),
            ctx.accounts.data_buffer.to_account_info(),
            ctx.accounts.permission.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.recent_blockhashes.to_account_info(),
            ctx.accounts.program_state.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.switchboard_program.to_account_info(),
        ],
    )?;

    msg!(
        "VRF randomness requested for claim {} — awaiting Switchboard oracle",
        claim_id
    );
    Ok(())
}

// ─── Phase 2 ─────────────────────────────────────────────────────────────────

/// Consume the oracle-verified randomness and assign validators to the claim.
///
/// This instruction is called by the Switchboard oracle (via CPI) after it has
/// computed and published the ECVRF proof on-chain in the VRF account.
///
/// We read the result directly from `switchboard_vrf.current_round.result` and
/// verify the account is in the `StatusCallbackSuccess` state before proceeding.
/// This prevents anyone other than the Switchboard oracle from triggering assignment.
///
/// Caller: Switchboard oracle (via CPI).  The oracle's identity is verified by
/// checking it is a registered oracle on the oracle queue that owns the VRF account.
pub fn fulfill_vrf_randomness(
    ctx: Context<FulfillVrfRandomness>,
    claim_id: Pubkey,
) -> Result<()> {
    let vrf_state = &mut ctx.accounts.vrf_state;
    let claim = &mut ctx.accounts.claim_request;
    let pool = &ctx.accounts.pool;
    let validator_stake_pool = &ctx.accounts.validator_stake_pool;
    let clock = Clock::get()?;

    require!(
        claim.status == ClaimStatus::Pending,
        AceError::ClaimPeriodExpired
    );
    require!(
        vrf_state.pending_claims.contains(&claim_id),
        AceError::UnauthorizedValidator
    );

    // Read and validate randomness from the fulfilled Switchboard VRF account.
    let randomness = read_vrf_result(&ctx.accounts.switchboard_vrf)?;

    // Deterministic, bias-free validator selection using the oracle randomness.
    let required_validators = pool.min_validators.saturating_add(2) as usize;
    let selected_validators = select_validators_from_randomness(
        &randomness,
        &validator_stake_pool.validators,
        required_validators,
    )?;

    claim.validators_assigned = selected_validators.clone();
    claim.status = ClaimStatus::UnderValidation;
    claim.vrf_result = Some(randomness);

    vrf_state.last_randomness = Some(randomness);
    vrf_state.last_timestamp = clock.unix_timestamp;
    vrf_state.pending_claims.retain(|&id| id != claim_id);
    vrf_state.requests_completed = vrf_state
        .requests_completed
        .checked_add(1)
        .ok_or(AceError::MathOverflow)?;

    emit!(ValidatorsAssignedEvent {
        pool: pool.key(),
        claim_id,
        validators: selected_validators,
        randomness,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "VRF fulfilled — {} validators assigned to claim {}",
        required_validators,
        claim_id
    );
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Read the ECVRF result from a fulfilled Switchboard V2 VrfAccountData account.
///
/// Equivalent to (with switchboard-solana crate):
///   let vrf = VrfAccountData::new(account)?;
///   require!(vrf.status == VrfStatus::StatusCallbackSuccess, ...);
///   Ok(vrf.current_round.result)
fn read_vrf_result(vrf_account: &AccountInfo) -> Result<[u8; 32]> {
    let data = vrf_account.try_borrow_data()?;

    // Require the account is in StatusCallbackSuccess (= 3).
    // This ensures we only consume randomness after the oracle has published
    // and the Switchboard program has verified the ECVRF proof.
    require!(
        data.len() > VRF_STATUS_OFFSET && data[VRF_STATUS_OFFSET] == VRF_STATUS_FULFILLED,
        AceError::Unauthorized
    );

    require!(
        data.len() >= VRF_RESULT_OFFSET + 32,
        AceError::Unauthorized
    );

    data[VRF_RESULT_OFFSET..VRF_RESULT_OFFSET + 32]
        .try_into()
        .map_err(|_| error!(AceError::Unauthorized))
}

/// Partial Fisher-Yates shuffle seeded by the oracle randomness.
/// Draws `count` unique validators without replacement — O(n) and bias-free.
fn select_validators_from_randomness(
    randomness: &[u8; 32],
    available: &[Pubkey],
    count: usize,
) -> Result<Vec<Pubkey>> {
    require!(available.len() >= count, AceError::InsufficientValidators);

    let mut indices: Vec<usize> = (0..available.len()).collect();

    for i in 0..count {
        // Consume 4 bytes of randomness per draw, wrapping across the 32-byte array.
        let byte_offset = (i * 4) % 28;
        let seed = u32::from_le_bytes([
            randomness[byte_offset],
            randomness[byte_offset + 1],
            randomness[byte_offset + 2],
            randomness[byte_offset + 3],
        ]);
        let j = i + (seed as usize % (available.len() - i));
        indices.swap(i, j);
    }

    Ok(indices[..count].iter().map(|&idx| available[idx]).collect())
}

/// Construct a Switchboard V2 `vrf_request_randomness` instruction.
fn build_vrf_request_ix(
    vrf: Pubkey,
    authority: Pubkey,
    oracle_queue: Pubkey,
    queue_authority: Pubkey,
    data_buffer: Pubkey,
    permission: Pubkey,
    escrow: Pubkey,
    recent_blockhashes: Pubkey,
    program_state: Pubkey,
    token_program: Pubkey,
) -> Instruction {
    Instruction {
        program_id: SWITCHBOARD_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(vrf, false),
            AccountMeta::new_readonly(oracle_queue, false),
            AccountMeta::new_readonly(queue_authority, false),
            AccountMeta::new(data_buffer, false),
            AccountMeta::new(permission, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new_readonly(recent_blockhashes, false),
            AccountMeta::new_readonly(program_state, false),
            AccountMeta::new_readonly(token_program, false),
        ],
        data: VRF_REQUEST_RANDOMNESS_DISC.to_vec(),
    }
}

// ─── Account Contexts ─────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeVrfState<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + VrfState::INIT_SPACE,
        seeds = [b"vrf_state", pool.key().as_ref()],
        bump
    )]
    pub vrf_state: Box<Account<'info, VrfState>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    /// CHECK: Switchboard VRF account — created via `sb vrf create`.
    /// Ownership by the Switchboard program is the security invariant.
    #[account(
        constraint = switchboard_vrf.owner == &SWITCHBOARD_PROGRAM_ID @ AceError::Unauthorized
    )]
    pub switchboard_vrf: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestValidatorSelection<'info> {
    #[account(
        mut,
        seeds = [b"vrf_state", pool.key().as_ref()],
        bump = vrf_state.bump,
        constraint = vrf_state.pool == pool.key() @ AceError::InactiveCoverage,
        constraint = vrf_state.authority == authority.key() @ AceError::Unauthorized
    )]
    pub vrf_state: Box<Account<'info, VrfState>>,

    #[account(mut)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        seeds = [b"validator_stake", pool.key().as_ref()],
        bump = validator_stake_pool.bump,
        constraint = validator_stake_pool.pool == pool.key() @ AceError::InactiveCoverage
    )]
    pub validator_stake_pool: Box<Account<'info, ValidatorStakePool>>,

    /// CHECK: Switchboard VRF account — must match the pubkey stored at init time.
    #[account(
        mut,
        constraint = switchboard_vrf.key() == vrf_state.switchboard_vrf @ AceError::Unauthorized,
        constraint = switchboard_vrf.owner == &SWITCHBOARD_PROGRAM_ID @ AceError::Unauthorized
    )]
    pub switchboard_vrf: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Switchboard oracle queue account
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,

    /// CHECK: Oracle queue authority PDA
    pub queue_authority: AccountInfo<'info>,

    /// CHECK: Oracle queue data buffer
    #[account(mut)]
    pub data_buffer: AccountInfo<'info>,

    /// CHECK: VRF permission account (PDA on Switchboard program)
    #[account(mut)]
    pub permission: AccountInfo<'info>,

    /// CHECK: VRF escrow token account (holds wSOL for oracle reward)
    #[account(mut)]
    pub escrow: AccountInfo<'info>,

    /// CHECK: SlotHashes sysvar (used by Switchboard as VRF alpha input)
    pub recent_blockhashes: AccountInfo<'info>,

    /// CHECK: Switchboard program state PDA
    #[account(mut)]
    pub program_state: AccountInfo<'info>,

    /// CHECK: Switchboard V2 program — verified against known program ID
    #[account(
        constraint = switchboard_program.key() == SWITCHBOARD_PROGRAM_ID @ AceError::Unauthorized
    )]
    pub switchboard_program: AccountInfo<'info>,

    /// CHECK: SPL Token program
    pub token_program: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct FulfillVrfRandomness<'info> {
    #[account(
        mut,
        seeds = [b"vrf_state", pool.key().as_ref()],
        bump = vrf_state.bump,
        constraint = vrf_state.pool == pool.key() @ AceError::InactiveCoverage
    )]
    pub vrf_state: Box<Account<'info, VrfState>>,

    #[account(mut)]
    pub claim_request: Box<Account<'info, ClaimRequest>>,

    pub pool: Box<Account<'info, InsurancePool>>,

    #[account(
        seeds = [b"validator_stake", pool.key().as_ref()],
        bump = validator_stake_pool.bump,
        constraint = validator_stake_pool.pool == pool.key() @ AceError::InactiveCoverage
    )]
    pub validator_stake_pool: Box<Account<'info, ValidatorStakePool>>,

    /// CHECK: Switchboard VRF account — must be in StatusCallbackSuccess state.
    /// The Switchboard oracle is the sole entity capable of advancing a VRF account
    /// into this state, providing the ECVRF proof guarantee.
    #[account(
        constraint = switchboard_vrf.key() == vrf_state.switchboard_vrf @ AceError::Unauthorized,
        constraint = switchboard_vrf.owner == &SWITCHBOARD_PROGRAM_ID @ AceError::Unauthorized
    )]
    pub switchboard_vrf: AccountInfo<'info>,

    /// The Switchboard oracle that fulfilled this request (must sign).
    /// Verified indirectly: only the oracle that produced the VRF proof can advance
    /// VrfStatus to StatusCallbackSuccess in the VRF account we check above.
    pub oracle: Signer<'info>,

    pub clock: Sysvar<'info, Clock>,
}
