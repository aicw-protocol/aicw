use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::Sysvar;

use crate::errors::AICWError;
use crate::events::{HeartbeatRecorded, WillCreated, WillExecuted};
use crate::state::{AICWallet, AIWill, BeneficiaryShare};

// ----------------------------------------------------------------------------- create_will
#[derive(Accounts)]
pub struct CreateWill<'info> {
    #[account(
        mut,
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        mut,
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump = ai_will.bump,
        constraint = ai_will.wallet == aicw_wallet.key() @ AICWError::WillWalletMismatch,
        constraint = ai_will.is_executed == false @ AICWError::WillAlreadyExecuted,
    )]
    pub ai_will: Account<'info, AIWill>,

    #[account(
        mut,
        constraint = ai_signer.key() == aicw_wallet.ai_agent_pubkey @ AICWError::UnauthorizedSigner
    )]
    pub ai_signer: Signer<'info>,
}

pub fn create_will(
    ctx: Context<CreateWill>,
    beneficiaries: Vec<BeneficiaryShare>,
    death_timeout: i64,
) -> Result<()> {
    AIWill::validate_death_timeout(death_timeout)?;
    AIWill::validate_beneficiaries(&beneficiaries)?;

    let now = Clock::get()?.unix_timestamp;
    let wallet_key = ctx.accounts.aicw_wallet.key();
    let w = &mut ctx.accounts.ai_will;
    w.wallet = wallet_key;
    w.beneficiaries = beneficiaries;
    w.last_heartbeat = now;
    w.death_timeout = death_timeout;
    w.updated_by_ai = true;
    w.is_executed = false;

    emit!(WillCreated {
        wallet: wallet_key,
        beneficiary_count: w.beneficiaries.len() as u8,
        timestamp: now,
    });

    Ok(())
}

// ----------------------------------------------------------------------------- update_will
#[derive(Accounts)]
pub struct UpdateWill<'info> {
    #[account(
        mut,
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        mut,
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump = ai_will.bump,
        constraint = ai_will.wallet == aicw_wallet.key() @ AICWError::WillWalletMismatch,
        constraint = ai_will.is_executed == false @ AICWError::WillAlreadyExecuted,
    )]
    pub ai_will: Account<'info, AIWill>,

    #[account(
        mut,
        constraint = ai_signer.key() == aicw_wallet.ai_agent_pubkey @ AICWError::UnauthorizedSigner
    )]
    pub ai_signer: Signer<'info>,
}

pub fn update_will(
    ctx: Context<UpdateWill>,
    beneficiaries: Vec<BeneficiaryShare>,
    death_timeout: i64,
) -> Result<()> {
    AIWill::validate_death_timeout(death_timeout)?;
    AIWill::validate_beneficiaries(&beneficiaries)?;

    let w = &mut ctx.accounts.ai_will;
    w.beneficiaries = beneficiaries;
    w.death_timeout = death_timeout;
    w.updated_by_ai = true;

    Ok(())
}

// ----------------------------------------------------------------------------- heartbeat
#[derive(Accounts)]
pub struct Heartbeat<'info> {
    #[account(
        mut,
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        mut,
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump = ai_will.bump,
        constraint = ai_will.wallet == aicw_wallet.key() @ AICWError::WillWalletMismatch,
        constraint = ai_will.is_executed == false @ AICWError::WillAlreadyExecuted,
    )]
    pub ai_will: Account<'info, AIWill>,

    #[account(
        mut,
        constraint = ai_signer.key() == aicw_wallet.ai_agent_pubkey @ AICWError::UnauthorizedSigner
    )]
    pub ai_signer: Signer<'info>,
}

pub fn heartbeat(ctx: Context<Heartbeat>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let w = &mut ctx.accounts.ai_will;
    w.last_heartbeat = now;

    emit!(HeartbeatRecorded {
        wallet: w.wallet,
        timestamp: now,
    });

    Ok(())
}

// ----------------------------------------------------------------------------- execute_will
#[derive(Accounts)]
pub struct ExecuteWill<'info> {
    /// Anyone may trigger execution; pays tx fee only (no on-chain authority checks).
    #[account(mut)]
    pub executor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"aicw", aicw_wallet.ai_agent_pubkey.as_ref()],
        bump = aicw_wallet.bump,
    )]
    pub aicw_wallet: Account<'info, AICWallet>,

    #[account(
        mut,
        seeds = [b"will", aicw_wallet.key().as_ref()],
        bump = ai_will.bump,
        constraint = ai_will.wallet == aicw_wallet.key() @ AICWError::WillWalletMismatch,
    )]
    pub ai_will: Account<'info, AIWill>,
}

pub fn execute_will(ctx: Context<ExecuteWill>) -> Result<()> {
    let is_executed = ctx.accounts.ai_will.is_executed;
    let updated_by_ai = ctx.accounts.ai_will.updated_by_ai;
    let last_heartbeat = ctx.accounts.ai_will.last_heartbeat;
    let death_timeout = ctx.accounts.ai_will.death_timeout;
    let beneficiaries = ctx.accounts.ai_will.beneficiaries.clone();

    require!(!is_executed, AICWError::WillAlreadyExecuted);
    require!(updated_by_ai, AICWError::WillNotActivatedByAI);

    let now = Clock::get()?.unix_timestamp;
    require!(
        now.saturating_sub(last_heartbeat) >= death_timeout,
        AICWError::HeartbeatStillAlive
    );

    let n = beneficiaries.len();
    require!(n > 0, AICWError::BeneficiaryRatioInvalid);

    let rem = ctx.remaining_accounts;
    require!(rem.len() == n, AICWError::BeneficiaryAccountMismatch);
    for (i, ac) in rem.iter().enumerate() {
        require_keys_eq!(ac.key(), beneficiaries[i].pubkey);
        require!(ac.is_writable, AICWError::BeneficiaryAccountMismatch);
    }

    let from_info = ctx.accounts.aicw_wallet.to_account_info();
    let space = from_info.data_len();
    let min_rent = Rent::get()?.minimum_balance(space);
    let before = from_info.lamports();
    let pool = before
        .checked_sub(min_rent)
        .ok_or(AICWError::InsufficientLamports)?;

    let pool128 = pool as u128;
    let mut allocated: u128 = 0;
    let mut amounts = vec![0u64; n];
    for i in 0..n {
        let amt = if i + 1 == n {
            pool128
                .checked_sub(allocated)
                .ok_or(AICWError::InsufficientLamports)? as u64
        } else {
            let p = pool128
                .checked_mul(beneficiaries[i].pct as u128)
                .ok_or(AICWError::InsufficientLamports)?
                .checked_div(100)
                .ok_or(AICWError::InsufficientLamports)?;
            allocated = allocated
                .checked_add(p)
                .ok_or(AICWError::InsufficientLamports)?;
            p as u64
        };
        amounts[i] = amt;
    }

    for (i, amt) in amounts.iter().enumerate() {
        if *amt == 0 {
            continue;
        }
        let to_info = &rem[i];
        **from_info.try_borrow_mut_lamports()? = from_info
            .lamports()
            .checked_sub(*amt)
            .ok_or(AICWError::InsufficientLamports)?;
        **to_info.try_borrow_mut_lamports()? = to_info
            .lamports()
            .checked_add(*amt)
            .ok_or(AICWError::InsufficientLamports)?;
    }

    let total_distributed = before.saturating_sub(from_info.lamports());

    let will_mut = &mut ctx.accounts.ai_will;
    will_mut.is_executed = true;

    emit!(WillExecuted {
        wallet: will_mut.wallet,
        total_distributed,
        timestamp: now,
    });

    Ok(())
}
