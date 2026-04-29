use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("9RUEw4jcMi8xcGf3tJRCAdzUzLuhEurts8Z2QQLsRbaV");

#[program]
pub mod aicw {
    use super::*;

    /// Issue a new AICW wallet. Called once by a human (issuer).
    /// Creates both `AICWallet` and a default `AIWill` (issuer 100%, inactive).
    pub fn issue_wallet(
        ctx: Context<IssueWallet>,
        model_hash: [u8; 32],
        model_name: String,
    ) -> Result<()> {
        instructions::issue_wallet::issue_wallet(ctx, model_hash, model_name)
    }

    /// AI-only: transfer SOL from the wallet to a recipient.
    /// Requires `updated_by_ai = true` and the wallet to be alive (within death_timeout).
    pub fn ai_transfer(
        ctx: Context<AITransfer>,
        amount: u64,
        reasoning_hash: [u8; 32],
        reasoning_summary: String,
    ) -> Result<()> {
        instructions::ai_transfer::ai_transfer(ctx, amount, reasoning_hash, reasoning_summary)
    }

    /// AI-only: reject a transfer request and log the reasoning on-chain.
    /// Same liveness requirements as `ai_transfer`.
    pub fn ai_reject(
        ctx: Context<AIDecide>,
        requester: Pubkey,
        requested_amount: u64,
        reasoning_hash: [u8; 32],
        reasoning_summary: String,
    ) -> Result<()> {
        instructions::ai_decide::ai_reject(
            ctx,
            requester,
            requested_amount,
            reasoning_hash,
            reasoning_summary,
        )
    }

    /// Register an on-chain AI identity (Know Your Agent).
    pub fn register_identity(
        ctx: Context<RegisterIdentity>,
        model_hash: [u8; 32],
        model_name: String,
    ) -> Result<()> {
        instructions::register_identity::register_identity(ctx, model_hash, model_name)
    }

    /// AI-only: first activation of the default will.
    /// Replaces default beneficiaries, sets `updated_by_ai = true`.
    /// AI may include any pubkey (including the issuer) as a beneficiary; death_timeout >= 30 days.
    pub fn create_will(
        ctx: Context<CreateWill>,
        beneficiaries: Vec<crate::state::BeneficiaryShare>,
        death_timeout: i64,
    ) -> Result<()> {
        instructions::ai_will::create_will(ctx, beneficiaries, death_timeout)
    }

    /// AI-only: update beneficiaries or death_timeout on an already-activated will.
    pub fn update_will(
        ctx: Context<UpdateWill>,
        beneficiaries: Vec<crate::state::BeneficiaryShare>,
        death_timeout: i64,
    ) -> Result<()> {
        instructions::ai_will::update_will(ctx, beneficiaries, death_timeout)
    }

    /// AI-only: prove liveness. Resets `last_heartbeat` to current clock timestamp.
    pub fn heartbeat(ctx: Context<Heartbeat>) -> Result<()> {
        instructions::ai_will::heartbeat(ctx)
    }

    /// Permissionless: execute the will after death_timeout has elapsed.
    /// Distributes (balance − rent-exempt minimum) to beneficiaries by percentage.
    /// Fails if `updated_by_ai = false` or the AI is still alive.
    pub fn execute_will(ctx: Context<ExecuteWill>) -> Result<()> {
        instructions::ai_will::execute_will(ctx)
    }
}
