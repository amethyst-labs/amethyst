use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

use crate::constants::BASIS_POINTS_DIVISOR;

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
pub enum PositionValueChange {
    /// Increase a value.
    Increase(u64),
    /// Decrease a value.
    Decrease(u64),
}

impl Default for PositionValueChange {
    fn default() -> Self {
        Self::Increase(0)
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
pub enum Direction {
    /// A long position where the collateral and underlying represent an asset.
    Long,
    /// A short position where the collateral and underlying represent a stablecoin.
    Short,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Long
    }
}

#[assert_size(aligns, 160)]
#[account]
#[repr(C)]
pub struct Position {
    /// The escrow bump seed.
    pub escrow_bump_seed: [u8; 1], // 1
    padding: [u8; 15], // 16

    pub self_address: Pubkey, // 48
    /// The position authority.
    pub authority: Pubkey, // 80
    /// The token mint of the position.
    pub token_mint: Pubkey, // 12

    /// The collateral of the position.
    pub collateral: u64, // 120
    /// The position size.
    pub size: u64, // 128
    /// The average entry price.
    pub avg_entry_price: u64, // 136
    /// The last funding payment index.
    pub last_funding_index: u64, // 144
    /// The timestamp of the last funding payment.
    pub last_funding_payment: u64, // 152
    padding2: [u64; 1], // 128
}

impl Position {
    /// Gets the escrow's signer seeds.
    pub fn escrow_signer_seeds(&self) -> [&[u8]; 4] {
        use crate::constants::B_ESCROW;
        [
            B_ESCROW,
            self.self_address.as_ref(),
            self.token_mint.as_ref(),
            &self.escrow_bump_seed,
        ]
    }

    /// The leverage on this position.
    /// This value is represented in basis points.
    pub fn leverage(&self) -> u64 {
        self.size
            .checked_mul(BASIS_POINTS_DIVISOR)
            .and_then(|n| n.checked_div(self.collateral))
            .unwrap()
    }

    /// Initializes the position
    pub fn init(
        &mut self,
        authority: Pubkey,
        token_mint: Pubkey,
        collateral: u64,
        size: u64,
        avg_entry_price: u64,
    ) {
        self.authority = authority;
        self.token_mint = token_mint;
        self.collateral = collateral;
        self.size = size;
        self.avg_entry_price = avg_entry_price;
    }

    /// Pays funding.
    pub fn pay_funding(&mut self, funding_index: u64, current_timestamp: u64) {
        self.last_funding_index = funding_index;
        self.last_funding_payment = current_timestamp;
    }
}
