use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::Vault;

#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(
        mut,
        has_one = token_vault,
        has_one = token_mint
    )]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        token::authority = vault.vault_signer,
        token::mint = token_mint
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::authority = authority,
        token::mint = token_mint
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    pub lp_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = payer,
        token::authority = authority,
        token::mint = token_mint
    )]
    pub lp_token_account: Box<Account<'info, TokenAccount>>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

impl<'info> DepositLiquidity<'info> {
    /// Perform validation.
    ///
    ///
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(ctx: Context<DepositLiquidity>) -> Result<()> {
    Ok(())
}
