use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawLiquidity<'info> {
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<WithdrawLiquidity>) -> Result<()> {
    Ok(())
}