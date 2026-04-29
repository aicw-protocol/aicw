use crate::errors::AICWError;
use crate::events::AIDecisionRejected;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AIDecide<'info> {
    #[account(
        mut,
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump = ai_will.bump,
        constraint = ai_will.wallet == aicw_wallet.key() @ AICWError::WillWalletMismatch,
        constraint = ai_will.is_executed == false @ AICWError::WillAlreadyExecuted,
    )]
    pub ai_will: Account<'info, AIWill>,

    #[account(
        mut,
        constraint = ai_signer.key() == aicw_wallet.ai_agent_pubkey
            @ AICWError::UnauthorizedSigner
    )]
    pub ai_signer: Signer<'info>,

    #[account(
        init,
        payer = ai_signer,
        space = DecisionLog::LEN,
        seeds = [
            b"decision",
            aicw_wallet.key().as_ref(),
            aicw_wallet.decisions_made.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub decision_log: Account<'info, DecisionLog>,

    pub system_program: Program<'info, System>,
}

pub fn ai_reject(
    ctx: Context<AIDecide>,
    requester: Pubkey,
    requested_amount: u64,
    reasoning_hash: [u8; 32],
    reasoning_summary: String,
) -> Result<()> {
    require!(
        reasoning_summary.len() <= DecisionLog::MAX_REASONING_SUMMARY_LEN,
        AICWError::ReasoningSummaryTooLong
    );
    require!(
        ctx.accounts.ai_will.updated_by_ai,
        AICWError::WillNotActivatedByAI
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        ctx.accounts.ai_will.is_alive(now),
        AICWError::WalletPastDeathTimeout
    );

    let wallet = &mut ctx.accounts.aicw_wallet;

    let log = &mut ctx.accounts.decision_log;
    log.wallet = wallet.key();
    log.decision_id = wallet.decisions_made;
    log.timestamp = now;
    log.decision_type = DecisionType::RejectTransfer;
    log.amount = requested_amount;
    log.requester = requester;
    log.approved = false;
    log.reasoning_hash = reasoning_hash;
    log.reasoning_summary = reasoning_summary;
    log.bump = ctx.bumps.decision_log;

    wallet.decisions_made = wallet
        .decisions_made
        .checked_add(1)
        .ok_or(AICWError::ArithmeticOverflow)?;
    wallet.decisions_rejected = wallet
        .decisions_rejected
        .checked_add(1)
        .ok_or(AICWError::ArithmeticOverflow)?;

    emit!(AIDecisionRejected {
        wallet: wallet.key(),
        requester,
        amount: requested_amount,
        reasoning_summary: log.reasoning_summary.clone(),
        timestamp: log.timestamp,
    });

    Ok(())
}
