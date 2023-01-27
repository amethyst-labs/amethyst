use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

#[assert_size(aligns, 40)]
#[account]
#[repr(C)]
pub struct Cache {
    /// The authority.
    pub authority: Pubkey, // 32

    pub lp_token_supply: u64, // 40
}

impl Cache {
    pub fn get_weight(&self, index: usize) -> u64 {
        0u64
    }

    pub fn get_total_token_weights(&self) -> u64 {
        0u64
    }
}
