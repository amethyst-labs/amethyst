use anchor_lang::prelude::*;

use crate::state::{position::Position, vault::Vault};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub vault: Box<Account<'info, Vault>>,

    #[account(
        mut,
        has_one = authority,
        close = rent_destination
    )]
    pub position: Box<Account<'info, Position>>,

    /// CHECK: We do not need to check this.
    pub rent_destination: AccountInfo<'info>,

    pub authority: Signer<'info>,
}

impl<'info> ClosePosition<'info> {
    /// Perform validation.
    ///
    /// In this specific case we only need to validate that
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// To close out the position we have to perform PNL logic.
pub fn handler(ctx: Context<ClosePosition>) -> Result<()> {
    ctx.accounts.validate()?;
    Ok(())
}
