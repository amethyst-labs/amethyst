use anchor_lang::prelude::*;

use crate::state::cache::Cache;

#[derive(Accounts)]
pub struct CloseGlobalCache<'info> {
    #[account(
        mut,
        has_one = authority,
        close = authority
    )]
    pub global_cache: Box<Account<'info, Cache>>,

    pub authority: Signer<'info>,
}

impl<'info> CloseGlobalCache<'info> {
    /// Perform validation.
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Close the global cache account.
pub fn handler(ctx: Context<CloseGlobalCache>) -> Result<()> {
    ctx.accounts.validate()
}
