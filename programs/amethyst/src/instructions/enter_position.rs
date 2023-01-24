use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    args::EnterPositionArgs,
    constants::{B_ESCROW, B_POSITION},
    contexts::{impl_change_position_ctx, ChangePositionContext},
    error::ErrorCode,
    state::{
        position::{Direction, Position},
        vault::{Vault, VaultCache},
    },
};

#[derive(Accounts)]
pub struct EnterPosition<'info> {
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
        init,
        seeds = [
            B_POSITION,
            authority.key().as_ref(),
            token_mint.key().as_ref()
        ],
        bump,
        space = size_of::<Position>() + 8,
        payer = payer
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
        init,
        payer = payer,
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

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> EnterPosition<'info> {
    /// Perform validation.
    ///
    /// In this specific case we need to validate that the desired position size
    /// is not greater than the available liquidity in the vault.
    fn validate(&self, amount: u64) -> Result<()> {
        let available_liquidity = self
            .vault
            .deposits
            .checked_sub(self.vault.reserved)
            .unwrap();
        require!(
            available_liquidity < amount,
            ErrorCode::InsufficientLiquidityToEnterPosition
        );
        Ok(())
    }

    /// Enters the position.
    ///
    /// We do this by transferring the user's collateral to the escrow along with
    /// the remaining funds from the vault's token account, which are then marked as reserved.
    fn enter_position(&mut self, args: &EnterPositionArgs) -> Result<()> {
        let reserved = args.size.checked_sub(args.collateral).unwrap();

        self.deposit_collateral(args.collateral)?;
        self.increase_size(reserved)?;

        let position = &mut self.position;
        let vault = &mut self.vault;
        let vault_cache = &mut self.vault_cache;

        position.init(
            self.authority.key(),
            self.token_mint.key(),
            args.collateral,
            args.size,
            0u64,
        );

        let _ = match args.direction {
            Direction::Long => vault_cache.get_next_long_average_entry_price(args.size),
            Direction::Short => vault_cache.get_next_short_average_entry_price(args.size),
        };

        vault.increase_reserved(reserved);

        Ok(())
    }
}

pub fn handler(ctx: Context<EnterPosition>, args: EnterPositionArgs) -> Result<()> {
    ctx.accounts.validate(args.size)?;
    ctx.accounts.enter_position(&args)?;
    Ok(())
}

impl_change_position_ctx! { EnterPosition<'info> }
