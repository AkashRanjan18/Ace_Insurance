use anchor_lang::prelude::*;

#[error_code]
pub enum AceError {
    #[msg("Premium amount must be greater than zero")]
    InvalidPremiumAmount,

    #[msg("Coverage amount must be greater than premium amount")]
    InvalidCoverageAmount,

    #[msg("Minimum validators must be at least 3")]
    InsufficientValidators,

    #[msg("Claim period must be positive")]
    InvalidClaimPeriod,

    #[msg("Coverage amount exceeds pool maximum")]
    ExcessiveCoverageAmount,

    #[msg("User coverage is not active")]
    InactiveCoverage,

    #[msg("Claim amount exceeds user coverage")]
    ExcessiveClaimAmount,

    #[msg("Claim period has expired or incident predates join date")]
    ClaimPeriodExpired,

    #[msg("Insufficient pool funds for claim payout")]
    InsufficientPoolFunds,

    #[msg("Validator is not authorized for this claim")]
    UnauthorizedValidator,

    #[msg("Validator has already validated this claim")]
    DuplicateValidation,

    #[msg("Stake amount is below minimum required (0.1 SOL)")]
    InsufficientStake,

    #[msg("Validator reputation is too low for auditor tier")]
    LowReputation,

    #[msg("Premium payment is overdue")]
    PremiumOverdue,

    #[msg("Mathematical overflow occurred")]
    MathOverflow,

    #[msg("Unauthorized access attempt")]
    Unauthorized,

    #[msg("Account is already initialized")]
    AlreadyInitialized,

    #[msg("Invalid timestamp provided")]
    InvalidTimestamp,

    #[msg("Hack evidence hash cannot be empty")]
    EmptyEvidenceHash,

    #[msg("Affected protocol address is invalid or empty")]
    InvalidProtocolAddress,

    #[msg("Wallets balance mismatch: loss must be positive")]
    InvalidWalletLoss,

    #[msg("Fast-track is not active for this pool")]
    FastTrackNotActive,

    #[msg("Emergency payout already received for this claim")]
    EmergencyPayoutAlreadyReceived,

    #[msg("Fast-track proposal expired")]
    FastTrackProposalExpired,

    #[msg("Only auditor-tier validators can approve fast-track")]
    AuditorRequiredForFastTrack,

    #[msg("Insufficient auditor approvals for fast-track")]
    InsufficientAuditorApprovals,

    #[msg("User does not meet OCCR discount eligibility")]
    OccrDiscountNotEligible,

    #[msg("String is way too long")]
    StringTooLong,
}