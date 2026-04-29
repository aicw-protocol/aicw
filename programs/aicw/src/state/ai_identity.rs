use anchor_lang::prelude::*;

#[account]
pub struct AIIdentity {
    pub owner_wallet: Pubkey,
    pub model_hash: [u8; 32],
    pub model_name: String,

    pub reputation_score: u16,
    pub total_predictions: u64,
    pub correct_predictions: u64,
    pub accuracy_rate: u16,

    pub interaction_count: u64,
    pub last_interaction: i64,

    pub bump: u8,
}

impl AIIdentity {
    pub const MAX_MODEL_NAME_LEN: usize = 64;

    pub const LEN: usize = 8 // discriminator
        + 32                 // owner_wallet
        + 32                 // model_hash
        + 4 + Self::MAX_MODEL_NAME_LEN // model_name (String)
        + 2                  // reputation_score
        + 8                  // total_predictions
        + 8                  // correct_predictions
        + 2                  // accuracy_rate
        + 8                  // interaction_count
        + 8                  // last_interaction
        + 1;                 // bump
}
