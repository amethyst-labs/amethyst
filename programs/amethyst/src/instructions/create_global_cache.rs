use anchor_lang::prelude::*;

use crate::state::cache::Cache;

#[derive(Accounts)]
pub struct CreateGlobalCache<'info> {
    #[account(zero)]
    pub cache: Box<Account<'info, Cache>>,

    pub authority: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateGlobalCache<'info> {
    /// Perform validation.
    ///
    ///
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(ctx: Context<CreateGlobalCache>) -> Result<()> {
    ctx.accounts.validate()?;
    Ok(())
}
