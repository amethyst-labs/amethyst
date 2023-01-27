use anchor_lang::prelude::*;

use crate::state::position::Direction;

#[derive(Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct CreateConfigArgs {
    /// The tax.
    pub tax_bps: u16,
    /// The stable tax.
    pub stable_tax_bps: u16,
    /// The fee for minting and redeeming LP positions.
    pub mint_burn_fee_bps: u16,
    /// The fee for non-stable swaps.
    pub swap_fee_bps: u16,
    /// The fee for stable swapsz
    pub stable_swap_fee_bps: u16,
    /// The fee applied to margin position.
    pub margin_fee_bps: u16,
}

#[derive(Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct EnterPositionArgs {
    /// The user's collateral to be used.
    pub collateral: u64,
    /// The total position size.
    pub size: u64,
    /// The direction of the position.
    pub direction: Direction,
}

#[derive(Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct SwapArgs {
    /// The amount of asset A to be swapped.
    pub amount_in: u64,
    /// The minimum amount of asset B.
    pub min_amount_out: u64,
}
