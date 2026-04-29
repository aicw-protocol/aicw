use anchor_lang::prelude::*;

#[event]
pub struct WalletIssued {
    pub wallet: Pubkey,
    pub ai_agent: Pubkey,
    pub issuer: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AITransferExecuted {
    pub wallet: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub decision_id: u64,
    pub timestamp: i64,
}

#[event]
pub struct AIDecisionRejected {
    pub wallet: Pubkey,
    pub requester: Pubkey,
    pub amount: u64,
    pub reasoning_summary: String,
    pub timestamp: i64,
}

#[event]
pub struct AIIdentityRegistered {
    pub wallet: Pubkey,
    pub model_name: String,
    pub timestamp: i64,
}

#[event]
pub struct WillCreated {
    pub wallet: Pubkey,
    pub beneficiary_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct WillExecuted {
    pub wallet: Pubkey,
    pub total_distributed: u64,
    pub timestamp: i64,
}

#[event]
pub struct HeartbeatRecorded {
    pub wallet: Pubkey,
    pub timestamp: i64,
}
