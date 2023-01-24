use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

#[assert_size(aligns, 32)]
#[account]
#[repr(C)]
pub struct Cache {
    /// The authority.
    pub authority: Pubkey, // 32
}

impl Cache {}
