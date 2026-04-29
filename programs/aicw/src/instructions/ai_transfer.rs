use crate::errors::AICWError;
use crate::events::AITransferExecuted;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::Sysvar;

#[derive(Accounts)]
pub struct AITransfer<'info> {
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

    /// CHECK: Transfer recipient
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

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

pub fn ai_transfer(
    ctx: Context<AITransfer>,
    amount: u64,
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

    let wallet_key = ctx.accounts.aicw_wallet.key();
    let signer_key = ctx.accounts.ai_signer.key();
    let recipient_key = ctx.accounts.recipient.key();

    // PDA wallet account holds data; System Program `transfer` rejects `from` with data.
    // Move lamports manually while keeping this account rent-exempt.
    let from_info = ctx.accounts.aicw_wallet.to_account_info();
    let to_info = ctx.accounts.recipient.to_account_info();
    let space = from_info.data_len();
    let min_rent = Rent::get()?.minimum_balance(space);
    let current = from_info.lamports();
    let after = current
        .checked_sub(amount)
        .ok_or(AICWError::InsufficientLamports)?;
    require!(after >= min_rent, AICWError::InsufficientLamports);

    let to_lamports = to_info.lamports();
    let new_to = to_lamports
        .checked_add(amount)
        .ok_or(AICWError::ArithmeticOverflow)?;
    **from_info.try_borrow_mut_lamports()? = after;
    **to_info.try_borrow_mut_lamports()? = new_to;

    let wallet = &mut ctx.accounts.aicw_wallet;
    let timestamp = now;
    let decision_id = wallet.decisions_made;

    let log = &mut ctx.accounts.decision_log;
    log.wallet = wallet_key;
    log.decision_id = decision_id;
    log.timestamp = timestamp;
    log.decision_type = DecisionType::Transfer;
    log.amount = amount;
    log.requester = signer_key;
    log.approved = true;
    log.reasoning_hash = reasoning_hash;
    log.reasoning_summary = reasoning_summary;
    log.bump = ctx.bumps.decision_log;

    wallet.total_transactions = wallet
        .total_transactions
        .checked_add(1)
        .ok_or(AICWError::ArithmeticOverflow)?;
    wallet.total_volume = wallet
        .total_volume
        .checked_add(amount)
        .ok_or(AICWError::ArithmeticOverflow)?;
    wallet.decisions_made = wallet
        .decisions_made
        .checked_add(1)
        .ok_or(AICWError::ArithmeticOverflow)?;

    emit!(AITransferExecuted {
        wallet: wallet_key,
        recipient: recipient_key,
        amount,
        decision_id,
        timestamp,
    });

    Ok(())
}
