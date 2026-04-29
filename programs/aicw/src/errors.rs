use anchor_lang::prelude::*;

#[error_code]
pub enum AICWError {
    #[msg("Unauthorized signer. AI agent key mismatch.")]
    UnauthorizedSigner,

    #[msg("Target program not in allowed list.")]
    ProgramNotAllowed,

    #[msg("AI identity not registered.")]
    IdentityNotRegistered,

    #[msg("Insufficient reputation score for this operation.")]
    InsufficientReputation,

    #[msg("Model name exceeds maximum length.")]
    ModelNameTooLong,

    #[msg("Reasoning summary exceeds maximum length.")]
    ReasoningSummaryTooLong,

    #[msg("Insufficient lamports for transfer (including rent-exempt minimum).")]
    InsufficientLamports,

    #[msg("Beneficiary share percents must sum to 100 (1-10 beneficiaries).")]
    BeneficiaryRatioInvalid,

    #[msg("Will cannot execute yet: heartbeat within death timeout window.")]
    HeartbeatStillAlive,

    #[msg("Will has already been executed.")]
    WillAlreadyExecuted,

    #[msg("AIWill wallet field does not match this AICW wallet.")]
    WillWalletMismatch,

    #[msg("Remaining accounts must match beneficiaries in order and be writable.")]
    BeneficiaryAccountMismatch,

    #[msg("heartbeat_interval and death_timeout must be positive.")]
    InvalidWillParameters,

    #[msg("AI must activate the will before this operation.")]
    WillNotActivatedByAI,

    #[msg("AICW wallet is past its death timeout.")]
    WalletPastDeathTimeout,

    #[msg("Arithmetic overflow in counter or lamport balance update.")]
    ArithmeticOverflow,
}
