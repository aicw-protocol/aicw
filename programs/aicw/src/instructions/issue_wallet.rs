use anchor_lang::prelude::*;
use crate::errors::AICWError;
use crate::events::WalletIssued;
use crate::state::*;

#[derive(Accounts)]
pub struct IssueWallet<'info> {
    #[account(
        init,
        payer = issuer,
        space = AICWallet::LEN,
        seeds = [b"aicw", ai_agent_pubkey.key().as_ref()],
        bump
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        init,
        payer = issuer,
        space = AIWill::LEN,
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump
    )]
    pub ai_will: Account<'info, AIWill>,

    #[account(mut)]
    pub issuer: Signer<'info>,

    /// CHECK: AI agent public key used for identification only
    pub ai_agent_pubkey: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn issue_wallet(
    ctx: Context<IssueWallet>,
    model_hash: [u8; 32],
    model_name: String,
) -> Result<()> {
    require!(
        model_name.len() <= AIIdentity::MAX_MODEL_NAME_LEN,
        AICWError::ModelNameTooLong
    );

    let wallet_key = ctx.accounts.aicw_wallet.key();
    let ai_agent_key = ctx.accounts.ai_agent_pubkey.key();
    let issuer_key = ctx.accounts.issuer.key();

    let wallet = &mut ctx.accounts.aicw_wallet;

    wallet.wallet_id = wallet_key.to_bytes();
    wallet.ai_agent_pubkey = ai_agent_key;
    wallet.issuer_pubkey = issuer_key;
    wallet.model_hash = model_hash;
    wallet.generation = 1;
    wallet.parent_wallet = None;
    wallet.allowed_programs = Vec::new();
    wallet.total_transactions = 0;
    wallet.total_volume = 0;
    wallet.decisions_made = 0;
    wallet.decisions_rejected = 0;
    wallet.verifiable_autonomy_proof = [0u8; 64];
    wallet.created_at = Clock::get()?.unix_timestamp;
    wallet.bump = ctx.bumps.aicw_wallet;

    let will = &mut ctx.accounts.ai_will;
    will.wallet = wallet_key;
    will.beneficiaries = vec![BeneficiaryShare {
        pubkey: issuer_key,
        pct: 100,
    }];
    will.last_heartbeat = wallet.created_at;
    will.death_timeout = AIWill::DEATH_TIMEOUT_SECONDS;
    will.updated_by_ai = false;
    will.is_executed = false;
    will.bump = ctx.bumps.ai_will;

    emit!(WalletIssued {
        wallet: wallet.key(),
        ai_agent: wallet.ai_agent_pubkey,
        issuer: wallet.issuer_pubkey,
        timestamp: wallet.created_at,
    });

    Ok(())
}
