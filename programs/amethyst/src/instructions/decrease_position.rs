use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token};

use crate::{state::{position::Position, Vault, VaultCache}, constants::B_ESCROW, error::ErrorCode};

#[derive(Accounts)]
pub struct DecreasePosition<'info> {
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

    /// The user's position.
    #[account(
        mut,
        has_one = authority,
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

    /// The position's token account.
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = escrow
    )]
    pub position_vault: Box<Account<'info, TokenAccount>>,

    /// The user's token account.
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = authority,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// The position's asset.
    pub token_mint: Box<Account<'info, Mint>>,

    /// The position's authority.
    pub authority: Signer<'info>,

    /// The payer of the transaction fee and rent of accounts being initialised.
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> DecreasePosition<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    fn process(&self, size: u64) -> Result<()> {

        

        Ok(())
    }


    /// Perform validation after performing an action.
    ///
    /// In this specific case we need to validate that, after potentially withdrawing collateral
    /// from the position, we do not exceed the maximum allowed leverage for this vault.
    fn post_validation(&self) -> Result<()> {
        let leverage = self.position.leverage();
        require!(
            leverage < self.vault.max_leverage.into(),
            ErrorCode::PositionLeverageExceedsLimit
        );
        Ok(())
    }
}

pub fn handler(ctx: Context<DecreasePosition>, size: u64) -> Result<()> {
    ctx.accounts.validate()?;
    ctx.accounts.process(size)?;
    ctx.accounts.post_validation()?;
    Ok(())
}