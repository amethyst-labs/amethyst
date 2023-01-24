use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    args::SwapArgs,
    error::ErrorCode,
    state::{
        cache::Cache,
        vault::{Vault, VaultCache},
    },
};

#[derive(Accounts)]
pub struct Swap<'info> {
    /// The global cache.
    pub cache: Box<Account<'info, Cache>>,

    /// The vault for asset A.
    #[account(
        mut,
        constraint = vault_a.token_vault == token_account_a.key(),
        constraint = vault_a.token_mint == token_account_a.key()
    )]
    pub vault_a: Box<Account<'info, Vault>>,

    /// The vault cache for asset A.
    #[account(
        mut,
        constraint = vault_a_cache.vault == vault_a.key()
    )]
    pub vault_a_cache: Box<Account<'info, VaultCache>>,

    /// The vault's token account for asset A.
    #[account(
        mut,
        token::mint = token_mint_a,
        token::authority = vault_a.vault_signer
    )]
    pub token_account_a: Box<Account<'info, TokenAccount>>,

    /// The token mint of asset A.
    pub token_mint_a: Box<Account<'info, Mint>>,

    /// The user's token account of asset A.
    #[account(
        mut,
        token::mint = token_mint_a,
        token::authority = authority,
    )]
    pub user_token_account_a: Box<Account<'info, TokenAccount>>,

    /// The vault for asset B.
    #[account(
        mut,
        constraint = vault_b.token_vault == token_account_b.key(),
        constraint = vault_b.token_mint == token_mint_b.key()
    )]
    pub vault_b: Box<Account<'info, Vault>>,

    /// The vault cache for asset B.
    #[account(
        mut,
        constraint = vault_b_cache.vault == vault_b.key()
    )]
    pub vault_b_cache: Box<Account<'info, VaultCache>>,

    /// The vault's token account for asset B.
    #[account(
        mut,
        token::mint = token_mint_b,
        token::authority = vault_b.vault_signer
    )]
    pub token_account_b: Box<Account<'info, TokenAccount>>,

    /// The token mint of asset B.
    pub token_mint_b: Box<Account<'info, Mint>>,

    /// The user's token account of asset B.
    #[account(
        mut,
        token::mint = token_mint_b,
        token::authority = authority,
    )]
    pub user_token_account_b: Box<Account<'info, TokenAccount>>,

    /// The user's wallet.
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> Swap<'info> {
    /// Perform validation.
    ///
    /// In this specific case we need to check if there is enough liquidity available
    /// to perform this swap.
    fn validate(&self, min_amount_out: u64) -> Result<()> {
        let available_liquidity = self
            .vault_b
            .deposits
            .checked_sub(self.vault_b.reserved)
            .unwrap();
        require!(
            available_liquidity > min_amount_out,
            ErrorCode::InsufficientLiquidityForSwap
        );
        Ok(())
    }
}

/// Here we perform a swap from token A to token B, if there is enough liqudiity available.
pub fn handler(ctx: Context<Swap>, args: SwapArgs) -> Result<()> {
    ctx.accounts.validate(args.min_amount_out)?;
    Ok(())
}
