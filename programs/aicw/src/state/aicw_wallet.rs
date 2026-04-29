use anchor_lang::prelude::*;

#[account]
pub struct AICWallet {
    pub wallet_id: [u8; 32],
    pub ai_agent_pubkey: Pubkey,
    pub issuer_pubkey: Pubkey,
    pub created_at: i64,

    pub model_hash: [u8; 32],
    pub generation: u8,
    pub parent_wallet: Option<Pubkey>,

    pub allowed_programs: Vec<Pubkey>,

    pub total_transactions: u64,
    pub total_volume: u64,
    pub decisions_made: u64,
    pub decisions_rejected: u64,

    pub verifiable_autonomy_proof: [u8; 64],
    pub bump: u8,
}

impl AICWallet {
    pub const MAX_ALLOWED_PROGRAMS: usize = 10;

    pub const LEN: usize = 8 // discriminator
        + 32               // wallet_id
        + 32               // ai_agent_pubkey
        + 32               // issuer_pubkey
        + 8                // created_at
        + 32               // model_hash
        + 1                // generation
        + 1 + 32           // parent_wallet (Option<Pubkey>)
        + 4 + (32 * Self::MAX_ALLOWED_PROGRAMS) // allowed_programs vec
        + 8                // total_transactions
        + 8                // total_volume
        + 8                // decisions_made
        + 8                // decisions_rejected
        + 64               // verifiable_autonomy_proof
        + 1;               // bump
}
