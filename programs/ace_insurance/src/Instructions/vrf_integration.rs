use anchor_lang::prelude::*;

use crate::errors::*;
use crate::events::*;
use crate::states::*;
use anchor_lang::solana_program::keccak;

/// Initialize VRF state for a pool
pub fn initialize_vrf_state(
    ctx: Context<InitializeVrfState>,
) -> Result<()> {
    let vrf_state = &mut ctx.accounts.vrf_state;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    vrf_state.pool = pool.key();
    vrf_state.switchboard_vrf = Pubkey::default();
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

    msg!("VRF state initialized for hack pool {}", pool.key());
    Ok(())
}

/// Request validator selection using VRF for a claim
pub fn request_validator_selection(
    ctx: Context<RequestValidatorSelection>,
    claim_id: Pubkey,
) -> Result<()> {
    let vrf_state = &mut ctx.accounts.vrf_state;
    let claim = &mut ctx.accounts.claim_request;
    let pool = &ctx.accounts.pool;
    let validator_stake_pool = &ctx.accounts.validator_stake_pool;
    let clock = Clock::get()?;

    // Verify claim is pending and needs validators
    require!(
        claim.status == ClaimStatus::Pending,
        AceError::ClaimPeriodExpired
    );
    require!(
        claim.pool == pool.key(),
        AceError::InactiveCoverage
    );
    require!(
        claim.validators_assigned.is_empty(),
        AceError::DuplicateValidation
    );

    // Need min_validators + 2 for hack expertise diversity
    let required_validators = pool.min_validators.saturating_add(2) as usize;
    require!(
        validator_stake_pool.validators.len() >= required_validators,
        AceError::InsufficientValidators
    );

    // Generate pseudo-randomness using hash of inputs
    let randomness = generate_randomness(
        &claim_id,
        &pool.key(),
        clock.unix_timestamp,
        clock.slot,
    );

    // Select validators randomly
    let selected_validators = select_random_validators(
        &randomness,
        &validator_stake_pool.validators,
        required_validators,
    )?;

    // Assign validators to claim
    claim.validators_assigned = selected_validators.clone();
    claim.status = ClaimStatus::UnderValidation;
    claim.vrf_result = Some(randomness);

    // Update VRF state
    vrf_state.last_randomness = Some(randomness);
    vrf_state.last_timestamp = clock.unix_timestamp;
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
        "Assigned {} validators to hack claim {}",
        required_validators,
        claim_id
    );

    Ok(())
}

/// Generate pseudo-randomness for validator selection
/// Uses sha256 hash function
fn generate_randomness(
    claim_id: &Pubkey,
    pool_id: &Pubkey,
    timestamp: i64,
    slot: u64,
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(claim_id.as_ref());
    data.extend_from_slice(pool_id.as_ref());
    data.extend_from_slice(&timestamp.to_le_bytes());
    data.extend_from_slice(&slot.to_le_bytes());

    // Use Keccak256 - built into Solana/Anchor
    let hash_result = keccak::hash(&data);
    
    // Returns the [u8; 32] array directly
    hash_result.to_bytes()
}

/// Select random validators from available pool
fn select_random_validators(
    randomness: &[u8; 32],
    available_validators: &[Pubkey],
    num_required: usize,
) -> Result<Vec<Pubkey>> {
    require!(
        available_validators.len() >= num_required,
        AceError::InsufficientValidators
    );

    let mut selected = Vec::new();
    let mut used_indices = Vec::new();

    for i in 0..num_required {
        let start_byte = (i * 4) % 28;
        let index_seed = u32::from_le_bytes([
            randomness[start_byte],
            randomness[start_byte + 1],
            randomness[start_byte + 2],
            randomness[start_byte + 3],
        ]);

        let mut index = (index_seed as usize) % available_validators.len();
        while used_indices.contains(&index) {
            index = (index + 1) % available_validators.len();
        }

        used_indices.push(index);
        selected.push(available_validators[index]);
    }

    Ok(selected)
}

// ----------------------------
// Account Validation Contexts
// ----------------------------

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

    pub clock: Sysvar<'info, Clock>,
}
