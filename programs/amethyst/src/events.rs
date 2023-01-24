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
