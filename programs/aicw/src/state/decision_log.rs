use anchor_lang::prelude::*;

#[account]
pub struct DecisionLog {
    pub wallet: Pubkey,
    pub decision_id: u64,
    pub timestamp: i64,
    pub decision_type: DecisionType,
    pub amount: u64,
    pub requester: Pubkey,
    pub approved: bool,
    pub reasoning_hash: [u8; 32],
    pub reasoning_summary: String,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum DecisionType {
    Transfer,
    RejectTransfer,
    IssueChildWallet,
    PolicyUpdate,
}

impl DecisionLog {
    pub const MAX_REASONING_SUMMARY_LEN: usize = 200;

    pub const LEN: usize = 8 // discriminator
        + 32                 // wallet
        + 8                  // decision_id
        + 8                  // timestamp
        + 1                  // decision_type (enum)
        + 8                  // amount
        + 32                 // requester
        + 1                  // approved
        + 32                 // reasoning_hash
        + 4 + Self::MAX_REASONING_SUMMARY_LEN // reasoning_summary
        + 1;                 // bump
}
