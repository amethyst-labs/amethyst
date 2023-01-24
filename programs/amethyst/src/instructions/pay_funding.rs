use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{
    constants::{B_ESCROW, SECONDS_IN_HOUR},
    error::ErrorCode,
    state::{position::Position, vault::Vault, VaultCache},
    utils::with_signer_pda,
};

#[derive(Accounts)]
pub struct PayFunding<'info> {
    /// The vault's cache.
    #[account(
        mut,
        has_one = vault
    )]
    pub vault_cache: Box<Account<'info, VaultCache>>,

    /// The vault of an asset.
    #[account(
        mut,
        has_one = token_mint,
        has_one = token_vault,
    )]
    pub vault: Box<Account<'info, Vault>>,

    /// The vault's token account.
    #[account(
        mut,
        token::authority = vault.vault_signer,
        token::mint = token_mint
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub position: Box<Account<'info, Position>>,

    /// CHECK: The escrow PDA for the position.
    #[account(
        seeds = [
            B_ESCROW,
            position.key().as_ref(),
            token_mint.key().as_ref()
        ],
        bump = position.escrow_bump_seed[0]
    )]
    pub escrow: AccountInfo<'info>,

    #[account(
        mut,
        token::mint = token_mint,
        token::authority = escrow
    )]
    pub position_vault: Box<Account<'info, TokenAccount>>,

    pub token_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> PayFunding<'info> {
    /// Perform validation.
    ///
    /// In this specific case we want to validate that enough time has passed for this position
    /// to pay funding.
    fn validate(&self, unix_timestamp: u64) -> Result<()> {
        require!(
            self.position.last_funding_payment + SECONDS_IN_HOUR < unix_timestamp,
            ErrorCode::InvalidFundingInterval
        );
        Ok(())
    }

    /// Transfer the funding amount to the vaults, which are the counter party.
    fn transfer_funding(&self, amount: u64) -> Result<()> {
        let cpi_signer = with_signer_pda(&self.escrow.to_account_info());
        let cpi_seeds = &[&self.position.escrow_signer_seeds()[..]];
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.position_vault.to_account_info(),
            to: self.token_vault.to_account_info(),
            authority: cpi_signer,
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, cpi_seeds);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    /// Pay the funding rate.
    fn pay_funding(&mut self, unix_timestamp: u64) -> Result<()> {
        // TODO: get funding amount
        let amount = 0u64;
        self.transfer_funding(amount)?;
        self.position
            .pay_funding(self.vault_cache.funding_index, unix_timestamp);

        Ok(())
    }
}

/// Funding payments are taken off of the position's collateral.
pub fn handler(ctx: Context<PayFunding>) -> Result<()> {
    let clock = Clock::get()?;
    let unix_timestamp: u64 = clock
        .unix_timestamp
        .try_into()
        .or(Err(ErrorCode::InvalidTimestampConversion))?;

    ctx.accounts.validate(unix_timestamp)?;
    ctx.accounts.pay_funding(unix_timestamp)?;
    Ok(())
}
