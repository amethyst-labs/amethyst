use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<WithdrawCollateral>) -> Result<()> {
    Ok(())
}