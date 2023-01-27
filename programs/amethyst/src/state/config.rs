use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

use crate::args::CreateConfigArgs;

#[assert_size(aligns, 64)]
#[account]
#[repr(C)]
pub struct Config {
    /// The config version.
    pub version: u8, // 1
    padding: [u8; 15], // 16

    /// The tax.
    pub tax_bps: u16, // 18
    /// The stable tax.
    pub stable_tax_bps: u16, // 20
    /// The fee for minting and redeeming LP positions.
    pub mint_burn_fee_bps: u16, // 22
    /// The fee for non-stable swaps.
    pub swap_fee_bps: u16, // 24
    /// The fee for stable swaps.
    pub stable_swap_fee_bps: u16, // 26
    /// The fee applied to margin position.
    pub margin_fee_bps: u16, // 28
    padding2: [u16; 2], // 32

    pub authority: Pubkey, // 64
}

impl Config {
    pub fn init(&mut self, authority: Pubkey, args: &CreateConfigArgs) {
        self.authority = authority;
        self.tax_bps = args.tax_bps;
        self.stable_tax_bps = args.stable_tax_bps;
        self.mint_burn_fee_bps = args.mint_burn_fee_bps;
        self.swap_fee_bps = args.swap_fee_bps;
        self.margin_fee_bps = args.margin_fee_bps;
    }
}
