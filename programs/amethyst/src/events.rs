use anchor_lang::prelude::*;

use crate::state::position::Direction;

#[event]
pub struct PositionOpened {
    /// The position itself.
    pub position: Pubkey,
    /// The authority of the position.
    pub authority: Pubkey,
    /// The position size.
    pub size: u64,
    /// The collateral of the position.
    pub collateral: u64,
    /// The direction of the position.
    pub direction: Direction,
}

#[event]
pub struct PositionIncreased {
    /// The position itself.
    pub position: Pubkey,
    /// The authority of the position.
    pub authority: Pubkey,
    /// The size delta.
    pub size_delta: u128,
    /// The collateral delta.
    pub collateral_delta_usd: u128,
    /// The fee paid.
    pub fee: u128,
    /// The oracle price.
    pub price: u64,
}
