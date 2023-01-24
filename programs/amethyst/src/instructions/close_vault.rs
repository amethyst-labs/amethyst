use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::vault::Vault};

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(
        mut,
        has_one = authority,
        close = rent_destination,
    )]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: This does not need to be checked.
    pub rent_destination: AccountInfo<'info>,

    pub authority: Signer<'info>,
}

impl<'info> CloseVault<'info> {
    /// Perform validation.
    ///
    /// In this specific case we just need to validate that the vault does not have:
    /// 1. Reserved assets.
    /// 2. Deposited assets.
    fn validate(&self) -> Result<()> {
        require!(
            self.vault.reserved == 0,
            ErrorCode::CannotCloseVaultWithReservedAssets
        );
        require!(
            self.vault.deposits == 0,
            ErrorCode::CannotCloseVaultWithDepositedAssets
        );
        Ok(())
    }
}

pub fn handler(ctx: Context<CloseVault>) -> Result<()> {
    ctx.accounts.validate()
}
