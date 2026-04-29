use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::AICWError;
use crate::events::AIIdentityRegistered;

#[derive(Accounts)]
pub struct RegisterIdentity<'info> {
    #[account(
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        mut,
        constraint = ai_signer.key() == aicw_wallet.ai_agent_pubkey
            @ AICWError::UnauthorizedSigner
    )]
    pub ai_signer: Signer<'info>,

    #[account(
        init,
        payer = ai_signer,
        space = AIIdentity::LEN,
        seeds = [b"identity", aicw_wallet.key().as_ref()],
        bump
    )]
    pub ai_identity: Account<'info, AIIdentity>,

    pub system_program: Program<'info, System>,
}

pub fn register_identity(
    ctx: Context<RegisterIdentity>,
    model_hash: [u8; 32],
    model_name: String,
) -> Result<()> {
    require!(
        model_name.len() <= AIIdentity::MAX_MODEL_NAME_LEN,
        AICWError::ModelNameTooLong
    );

    let identity = &mut ctx.accounts.ai_identity;
    identity.owner_wallet = ctx.accounts.aicw_wallet.key();
    identity.model_hash = model_hash;
    identity.model_name = model_name.clone();
    identity.reputation_score = 500;
    identity.total_predictions = 0;
    identity.correct_predictions = 0;
    identity.accuracy_rate = 0;
    identity.interaction_count = 0;
    identity.last_interaction = Clock::get()?.unix_timestamp;
    identity.bump = ctx.bumps.ai_identity;

    emit!(AIIdentityRegistered {
        wallet: ctx.accounts.aicw_wallet.key(),
        model_name,
        timestamp: identity.last_interaction,
    });

    Ok(())
}
