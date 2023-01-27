use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

use crate::{
    constants::{BASIS_POINTS_DIVISOR, SECONDS_IN_HOUR},
    error::ErrorCode,
    utils::{get_funding_fee, get_position_fee, price::get_next_average_price, usd_to_token},
};

use super::{Cache, Config};

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

#[assert_size(aligns, 288)]
#[account]
#[repr(C)]
pub struct Vault {
    /// The vault signer seed bump.
    pub vault_signer_seed_bump: [u8; 1], // 1
    /// Whether the vault has dynamic fees.
    pub has_dynamic_fees: bool, // 2
    /// Whether the vault represents a stable coin.
    pub is_stable: bool, // 3
    /// The index of the [`Vault`]'s
    pub cache_index: u8, // 4
    /// The decimals of the underlying token.
    pub decimals: u8, // 5
    padding: [u8; 7], // 12
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
    pub deposits: u128, // 192
    /// The amount of assets reserved for position.
    pub reserved: u128, // 208
    /// The funding rate index.
    pub cumulative_funding_rate: u128, // 224
    /// The debt amount on the vault.
    pub debt_amount: u128, // 240
    /// The amount of guaranteed USD.
    ///
    /// This represents the amount of USD that is guaranteed by open leverage positions.
    /// This value is used to calculate redemption values to sell debt.
    /// This value might become out of sync, it is possible for the actual value to be lower
    /// in the case of sudden price movements, it should be corrected after necessary liquidations happen.
    pub guaranteed_usd: u128, // 256
    /// The timestamp of the last funding update.
    pub last_funding_update: u64, // 264
    padding2: [u64; 3], // 288
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

    /// Gets the target debt amount for this vault.
    pub fn get_target_debt_amount(&self, cache: &Cache) -> u128 {
        if cache.lp_token_supply == 0 {
            return 0;
        }
        let weight = cache.get_weight(self.cache_index as usize);
        cache
            .lp_token_supply
            .checked_mul(weight)
            .and_then(|n| n.checked_div(cache.get_total_token_weights()))
            .unwrap()
            .into()
    }

    /// Gets the next funding rate.
    pub fn get_next_funding_rate(&self, unix_timestamp: u64) -> u128 {
        let intervals = unix_timestamp
            .checked_sub(self.last_funding_update)
            .and_then(|n| n.checked_div(SECONDS_IN_HOUR))
            .unwrap();
        let funding_rate_factor: u128 = 100_000; // TODO:
        let funding_rate = funding_rate_factor
            .checked_mul(self.reserved)
            .and_then(|n| n.checked_mul(intervals.into()))
            .and_then(|n| n.checked_div(self.deposits))
            .unwrap();
        funding_rate
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
    pub fn increase_reserved(&mut self, amount: u128) {
        self.reserved += amount;
    }

    /// Decreases the amount of reserved assets.
    pub fn decrease_reserved(&mut self, amount: u128) -> Result<()> {
        self.reserved = self.reserved.checked_sub(amount).unwrap();
        Ok(())
    }

    /// Increases the pool amount.
    pub fn increase_pool_amount(&mut self, amount: u128) {
        self.deposits += amount;
    }

    /// Decreases the pool amount.
    pub fn decrease_pool_amount(&mut self, amount: u128) -> Result<()> {
        self.deposits = self.reserved.checked_sub(amount).unwrap();
        Ok(())
    }

    /// Increases the debt amount.
    pub fn increase_debt_amount(&mut self, amount: u128) {
        self.debt_amount += amount;
    }

    /// Decreases the debt amount.
    pub fn decrease_debt_amount(&mut self, amount: u128) -> Result<()> {
        self.debt_amount = self.debt_amount.checked_sub(amount).unwrap();
        Ok(())
    }

    /// Increases the guaranteed USD amount.
    pub fn increase_guaranteed_usd(&mut self, amount: u128) {
        self.guaranteed_usd += amount;
    }

    /// Decreases the guaranteed USD amount.
    pub fn decrease_guaranteed_usd(&mut self, amount: u128) -> Result<()> {
        self.guaranteed_usd = self.guaranteed_usd.checked_sub(amount).unwrap();
        Ok(())
    }

    pub fn get_utilisation(&self) -> f64 {
        if self.deposits == 0 {
            return 0f64;
        }
        self.reserved.checked_div(self.deposits).unwrap() as f64
    }

    /// Updates the funding rate if enough time has passed in order for it to be updated.
    pub fn update_funding_rate(&mut self, clock: &Clock) -> Result<()> {
        let unix_timestamp: u64 = clock
            .unix_timestamp
            .try_into()
            .or(Err(ErrorCode::InvalidTimestampConversion))?;
        if self.last_funding_update + SECONDS_IN_HOUR <= unix_timestamp {
            self.cumulative_funding_rate =
                self.cumulative_funding_rate + self.get_next_funding_rate(unix_timestamp);
            self.last_funding_update = unix_timestamp;
        }
        Ok(())
    }

    pub fn collect_margin_fees(
        &mut self,
        config: &Config,
        vault_cache: &VaultCache,
        size_delta: u64,
        size: u64,
        funding_rate: u64,
    ) -> Result<u128> {
        let mut fee = get_position_fee(config, size_delta);
        fee += get_funding_fee(&self, size, funding_rate);

        let underlying_fee = usd_to_token(fee.into(), vault_cache.oracle_price, self.decimals)?;
        // todo: add fee

        Ok(underlying_fee)
    }
}
