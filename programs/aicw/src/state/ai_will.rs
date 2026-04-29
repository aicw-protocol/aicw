use anchor_lang::prelude::*;

/// One beneficiary line: pubkey + share percent (0–100). Sum of all shares must be 100.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, PartialEq, Eq)]
pub struct BeneficiaryShare {
    pub pubkey: Pubkey,
    pub pct: u8,
}

#[account]
pub struct AIWill {
    /// Linked AICW wallet (PDA) pubkey
    pub wallet: Pubkey,
    pub beneficiaries: Vec<BeneficiaryShare>,
    pub last_heartbeat: i64,
    pub death_timeout: i64,
    pub updated_by_ai: bool,
    pub is_executed: bool,
    pub bump: u8,
}

impl AIWill {
    pub const MAX_BENEFICIARIES: usize = 10;
    pub const DEATH_TIMEOUT_SECONDS: i64 = 30 * 24 * 60 * 60;

    pub const LEN: usize = 8  // discriminator
        + 32                // wallet
        + 4 + (Self::MAX_BENEFICIARIES * (32 + 1)) // vec<BeneficiaryShare>
        + 8                 // last_heartbeat
        + 8                 // death_timeout
        + 1                 // updated_by_ai
        + 1                 // is_executed
        + 1; // bump

    pub fn validate_beneficiaries(beneficiaries: &[BeneficiaryShare]) -> Result<()> {
        require!(
            !beneficiaries.is_empty() && beneficiaries.len() <= Self::MAX_BENEFICIARIES,
            crate::errors::AICWError::BeneficiaryRatioInvalid
        );
        let mut sum: u16 = 0;
        for b in beneficiaries {
            sum = sum
                .checked_add(b.pct as u16)
                .ok_or(crate::errors::AICWError::BeneficiaryRatioInvalid)?;
        }
        require!(sum == 100, crate::errors::AICWError::BeneficiaryRatioInvalid);
        Ok(())
    }

    pub fn validate_death_timeout(death_timeout: i64) -> Result<()> {
        require!(
            death_timeout >= Self::DEATH_TIMEOUT_SECONDS,
            crate::errors::AICWError::InvalidWillParameters
        );
        Ok(())
    }

    pub fn is_alive(&self, now: i64) -> bool {
        now.saturating_sub(self.last_heartbeat) < self.death_timeout
    }
}
