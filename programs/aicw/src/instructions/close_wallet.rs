use anchor_lang::prelude::*;

use crate::errors::AICWError;
use crate::events::WalletClosed;
use crate::state::{AICWallet, AIWill};

/// AI-only: close an unused wallet and return AICWallet + AIWill rent to the issuer.
/// Allowed only when the wallet has no on-chain transfer/reject activity.
#[derive(Accounts)]
pub struct CloseWallet<'info> {
    #[account(
        mut,
        close = rent_recipient,
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        mut,
        close = rent_recipient,
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump = ai_will.bump,
        constraint = ai_will.wallet == aicw_wallet.key() @ AICWError::WillWalletMismatch,
    )]
    pub ai_will: Account<'info, AIWill>,

    #[account(
        mut,
        constraint = ai_signer.key() == aicw_wallet.ai_agent_pubkey @ AICWError::UnauthorizedSigner
    )]
    pub ai_signer: Signer<'info>,

    /// CHECK: must be the original issuer; receives reclaimed rent from both PDAs.
    #[account(
        mut,
        constraint = rent_recipient.key() == aicw_wallet.issuer_pubkey @ AICWError::CloseRecipientMismatch,
    )]
    pub rent_recipient: AccountInfo<'info>,
}

pub fn close_wallet(ctx: Context<CloseWallet>) -> Result<()> {
    let wallet = &ctx.accounts.aicw_wallet;
    let will = &ctx.accounts.ai_will;

    require!(wallet.total_transactions == 0, AICWError::WalletHasOnChainActivity);
    require!(wallet.decisions_made == 0, AICWError::WalletHasOnChainActivity);
    require!(!will.is_executed, AICWError::WillAlreadyExecuted);

    emit!(WalletClosed {
        wallet: wallet.key(),
        issuer: wallet.issuer_pubkey,
        ai_agent: wallet.ai_agent_pubkey,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
