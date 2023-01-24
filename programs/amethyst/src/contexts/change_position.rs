use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    error::ErrorCode,
    state::{
        position::Position,
        vault::{Vault, VaultCache},
    },
    utils::with_signer_pda,
};

pub trait ChangePositionContext<'info> {
    fn vault_cache(&self) -> &Account<'info, VaultCache>;
    fn vault(&self) -> &Account<'info, Vault>;
    fn token_vault(&self) -> &Account<'info, TokenAccount>;
    fn vault_signer(&self) -> &AccountInfo<'info>;
    fn position(&self) -> &Account<'info, Position>;
    fn position_vault(&self) -> &Account<'info, TokenAccount>;
    fn escrow(&self) -> &AccountInfo<'info>;
    fn user_token_account(&self) -> &Account<'info, TokenAccount>;
    fn authority(&self) -> &Signer<'info>;
    fn token_program(&self) -> &Program<'info, Token>;

    /// Deposits collateral into a [`Position`].
    fn deposit_collateral(&self, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidTokenAmount);
        let cpi_program = self.token_program().to_account_info();
        let cpi_accounts = Transfer {
            from: self.user_token_account().to_account_info(),
            to: self.position_vault().to_account_info(),
            authority: self.authority().to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)
    }

    /// Withdraws collateral from a [`Position`].
    fn withdraw_collateral(&self, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidTokenAmount);
        let cpi_signer = with_signer_pda(self.escrow());
        let cpi_seeds = &[&self.position().escrow_signer_seeds()[..]];
        let cpi_program = self.token_program().to_account_info();
        let cpi_accounts = Transfer {
            from: self.position_vault().to_account_info(),
            to: self.user_token_account().to_account_info(),
            authority: cpi_signer,
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, cpi_seeds);
        transfer(cpi_ctx, amount)
    }

    /// Increases the total position size in a [`Position`] by transferring
    /// funds from the [`Vault`]'s token account to the escrow.
    fn increase_size(&self, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidTokenAmount);
        let cpi_signer = with_signer_pda(self.vault_signer());
        let cpi_seeds = &[&self.vault().vault_signer_seeds()[..]];
        let cpi_program = self.token_program().to_account_info();
        let cpi_accounts = Transfer {
            from: self.token_vault().to_account_info(),
            to: self.position_vault().to_account_info(),
            authority: cpi_signer,
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, cpi_seeds);
        transfer(cpi_ctx, amount)
    }

    /// Decreases the total position size in a [`Position`] by transferring
    /// funds from the escrow back into the [`Vault`]'s token account.
    fn decrease_size(&self, amount: u64) -> Result<()> {
        require!(amount > 0, ErrorCode::InvalidTokenAmount);
        let cpi_signer = with_signer_pda(self.escrow());
        let cpi_seeds = &[&self.position().escrow_signer_seeds()[..]];
        let cpi_program = self.token_program().to_account_info();
        let cpi_accounts = Transfer {
            from: self.position_vault().to_account_info(),
            to: self.token_vault().to_account_info(),
            authority: cpi_signer,
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, cpi_seeds);
        transfer(cpi_ctx, amount)
    }
}

macro_rules! impl_change_position_ctx {
    ($struct:ty) => {
        impl<'info> ChangePositionContext<'info> for $struct {
            fn vault_cache(&self) -> &Account<'info, VaultCache> {
                &self.vault_cache
            }
            fn vault(&self) -> &Account<'info, Vault> {
                &self.vault
            }
            fn token_vault(&self) -> &Account<'info, TokenAccount> {
                &self.token_vault
            }
            fn vault_signer(&self) -> &AccountInfo<'info> {
                &self.vault_signer
            }
            fn position(&self) -> &Account<'info, Position> {
                &self.position
            }
            fn position_vault(&self) -> &Account<'info, TokenAccount> {
                &self.position_vault
            }
            fn escrow(&self) -> &AccountInfo<'info> {
                &self.escrow
            }
            fn user_token_account(&self) -> &Account<'info, TokenAccount> {
                &self.user_token_account
            }
            fn authority(&self) -> &Signer<'info> {
                &self.authority
            }
            fn token_program(&self) -> &Program<'info, Token> {
                &self.token_program
            }
        }
    };
}

pub(crate) use impl_change_position_ctx;
