use std::sync::Arc;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::B_ESCROW,
    state::{
        position::Position,
        vault::{Vault, VaultCache},
    },
};

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
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
        has_one = vault_signer,
    )]
    pub vault: Box<Account<'info, Vault>>,

    /// The vault's token account.
    #[account(
        mut,
        token::authority = vault.vault_signer,
        token::mint = token_mint
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    /// The vault's token account authority.
    /// CHECK: The vault signer is checked.
    pub vault_signer: AccountInfo<'info>,

    #[account(
        mut,
        constraint = position.authority == position_authority.key(),
        close = position_authority
    )]
    pub position: Box<Account<'info, Position>>,

    /// The escrow of the position.
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
        close = position_authority,
    )]
    pub position_vault: Box<Account<'info, TokenAccount>>,

    /// The position's asset.
    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: This is checked.
    pub position_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> LiquidatePosition<'info> {
    /// Perform validation.
    ///
    /// In this specific case we want to validate that the position is eligible for liquidation.
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Perform the liquidation.
    fn process(&self) -> Result<()> {

        Ok(())
    }
}

/// When we liquidate a position, we transfer the collateral into the vault.
pub fn handler(ctx: Context<LiquidatePosition>) -> Result<()> {
    ctx.accounts.validate()?;
    ctx.accounts.process()?;
    Ok(())
}
