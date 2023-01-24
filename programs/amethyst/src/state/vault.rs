use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

use crate::utils::get_next_average_price;

#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub enum OracleType {
    /// Pyth oracle price feeds.
    Pyth,
    /// Switchboard V2 oracle price feeds.
    SwitchboardV2,
}

impl Default for OracleType {
    fn default() -> Self {
        Self::Pyth
    }
}

#[assert_size(aligns, 160)]
#[account]
#[repr(C)]
pub struct VaultCache {
    /// The type of oracle price feed.
    pub oracle_type: OracleType, // 1
    padding: [u8; 15], // 16

    /// The authority.
    pub authority: Pubkey, // 48
    /// The vault
    pub vault: Pubkey, // 80

    /// The total amount of deposits.
    pub total_deposits: u128, // 96

    /// The long open interest.
    pub long_open_interest: u128, // 112
    /// The short open interest.
    pub short_open_interest: u128, // 128

    /// The average entry price of all long positions.
    pub long_avg_entry_price: u64, // 136
    /// The average entry price of all short positions.
    pub short_avg_entry_price: u64, // 144

    /// The oracle price.
    pub oracle_price: u64, // 152
    /// The funding index.
    pub funding_index: u64, // 160
}

impl VaultCache {
    /// Initialize the vault cache.
    pub fn init(&mut self, authority: Pubkey, vault: Pubkey) {
        self.authority = authority;
        self.vault = vault;
    }

    /// Get the next average entry price for long positions.
    ///
    /// If this gets called we know for a fact that this is a long position being opened
    /// or being increased.
    pub fn get_next_long_average_entry_price(&mut self, long_position_delta: u64) -> Result<u64> {
        let next_average_price = get_next_average_price(
            self.long_open_interest,
            self.long_avg_entry_price,
            long_position_delta,
            self.oracle_price,
        )?;

        self.long_avg_entry_price = next_average_price;
        self.long_open_interest += long_position_delta as u128;

        Ok(next_average_price)
    }

    /// Get the next average price for short positions.
    ///
    /// If this gets called we know for a fact that this is a short position being opened
    /// or being increased.
    pub fn get_next_short_average_entry_price(&mut self, short_position_delta: u64) -> Result<u64> {
        let next_average_price = get_next_average_price(
            self.short_open_interest,
            self.short_avg_entry_price,
            short_position_delta,
            self.oracle_price,
        )?;

        self.short_avg_entry_price = next_average_price;
        self.short_open_interest += short_position_delta as u128;
        Ok(next_average_price)
    }

    /// Decreases the long open interest.
    pub fn decrease_long_open_interest(&mut self, amount: u64) {
        self.long_open_interest = self.long_open_interest.checked_sub(amount as u128).unwrap();
    }

    /// Decreases the short open interest.
    pub fn decrease_short_open_interest(&mut self, amount: u64) {
        self.short_open_interest = self
            .short_open_interest
            .checked_sub(amount as u128)
            .unwrap();
    }
}

#[assert_size(aligns, 224)]
#[account]
#[repr(C)]
pub struct Vault {
    /// The vault signer seed bump.
    pub vault_signer_seed_bump: [u8; 1], // 1
    padding: [u8; 11], // 12
    /// The maximum allowed leverage for this vault, represented in basis points.
    pub max_leverage: u32, // 16

    /// The address of the vault.
    pub self_address: Pubkey, // 48
    /// The authority of the liquidity pool.
    pub authority: Pubkey, // 80
    /// The token mint.
    pub token_mint: Pubkey, // 112
    /// The token account.
    pub token_vault: Pubkey, // 144
    /// The vault signer PDA.
    pub vault_signer: Pubkey, // 176
    /// The amount of deposits.
    pub deposits: u64, // 184
    /// The amount of assets reserved for position.
    pub reserved: u64, // 192
    /// The funding rate index.
    pub funding_rate_index: u64, // 200
    padding2: [u64; 3],
}

impl Vault {
    /// Gets the vault's signer seeds.
    pub fn vault_signer_seeds(&self) -> [&[u8]; 3] {
        use crate::constants::B_ESCROW;
        [
            B_ESCROW,
            self.self_address.as_ref(),
            &self.vault_signer_seed_bump,
        ]
    }

    /// Initialize the vault.
    pub fn init(
        &mut self,
        self_address: Pubkey,
        authority: Pubkey,
        token_mint: Pubkey,
        token_vault: Pubkey,
        vault_signer: Pubkey,
    ) {
        self.self_address = self_address;
        self.authority = authority;
        self.token_mint = token_mint;
        self.token_vault = token_vault;
        self.vault_signer = vault_signer;
    }

    /// Increases the amount of reserved assets.
    pub fn increase_reserved(&mut self, amount: u64) {
        self.reserved += amount;
    }

    /// Decreases the amount of reserved assets.
    pub fn decrease_reserved(&mut self, amount: u64) -> Result<()> {
        self.reserved = self.reserved.checked_sub(amount).unwrap();
        Ok(())
    }
}
