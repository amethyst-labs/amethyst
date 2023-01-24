use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::state::vault::Vault;

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub liquidity_pool: Box<Account<'info, Vault>>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,
}

impl<'info> CreateVault<'info> {
    /// Perform validation.
    ///
    ///
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(ctx: Context<CreateVault>) -> Result<()> {
    ctx.accounts.validate()?;
    Ok(())
}
