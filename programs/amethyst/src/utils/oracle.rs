use crate::{
    constants::{
        ORACLE_PRICE_FEED_TTL_SECS, ORACLE_PRICE_TARGET_EXPONENT, PYTH_FEED_DECIMALS,
        SWITCHBOARD_FEED_DECIMALS, SWITCHBOARD_FEED_EXPONENT,
    },
    error::ErrorCode,
    state::{BoundedPrice, PriceFeedResult},
};
use anchor_lang::prelude::*;
use pyth_sdk_solana::PriceFeed;
use std::ops::{Div, Mul};
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

fn get_target_exponent(base_decimals: i32, current_expo: i32, target_expo: i32) -> Result<i32> {
    let decimals = base_decimals
        .checked_add(current_expo)
        .unwrap()
        .checked_sub(target_expo)
        .unwrap();
    Ok(decimals)
}

/// Scales the given amount from current exponent to target exponent.
pub fn scale_amount(
    amount: u128,
    base_decimals: i32,
    current_expo: i32,
    target_expo: i32,
) -> Result<u128> {
    // scale the price to the target decimals
    let target_decimals = get_target_exponent(base_decimals, current_expo, target_expo)?;
    let decimal_adj = 10u64.pow(target_decimals.abs() as u32) as u128;

    #[cfg(test)]
    msg!(
        "Target Decimals: {} - Decimals Adj.: {}",
        target_decimals,
        decimal_adj
    );

    let scaled_amount = if target_decimals < 0 {
        amount.checked_div(decimal_adj).unwrap()
    } else {
        amount.checked_mul(decimal_adj).unwrap()
    };

    #[cfg(test)]
    msg!("Scaled Amount: {}", scaled_amount);

    Ok(scaled_amount)
}

/// Scales a given price from current exponent to target exponent.
pub fn scale_price(
    unscaled_price: f64,
    base_decimals: i32,
    current_expo: i32,
    target_expo: i32,
) -> Result<u128> {
    // scale the price to the target decimals
    let target_decimals = get_target_exponent(base_decimals, current_expo, target_expo)?;
    let decimal_adj = 10u64.pow(target_decimals.abs() as u32) as f64;

    #[cfg(test)]
    msg!(
        "Target Decimals: {} - Decimals Adj.: {}",
        target_decimals,
        decimal_adj
    );

    let scaled_price = if target_decimals < 0 {
        unscaled_price.div(decimal_adj)
    } else {
        unscaled_price.mul(decimal_adj)
    };

    #[cfg(test)]
    msg!("Scaled Price: {}", scaled_price);

    Ok(scaled_price as u128)
}

/// Gets an asset's from a Switchboard [`AggregatorAccountData`] and applies bounds if necessary.
pub fn get_switchboard_price(
    aggregator_account: &AggregatorAccountData,
    max_confidence_threshold: f64,
    unix_timestamp: i64,
) -> Result<PriceFeedResult> {
    let price_result: f64 = aggregator_account.get_result()?.try_into()?;

    aggregator_account
        .check_staleness(unix_timestamp, ORACLE_PRICE_FEED_TTL_SECS as i64)
        .map_err(|_| error!(ErrorCode::StaleOracleFeed))?;

    let std_deviation: f64 = aggregator_account
        .latest_confirmed_round
        .std_deviation
        .try_into()?;

    let price_result = scale_price(
        price_result,
        SWITCHBOARD_FEED_DECIMALS,
        SWITCHBOARD_FEED_EXPONENT,
        ORACLE_PRICE_TARGET_EXPONENT,
    )?;

    let std_deviation = scale_price(
        std_deviation,
        SWITCHBOARD_FEED_DECIMALS,
        SWITCHBOARD_FEED_EXPONENT,
        ORACLE_PRICE_TARGET_EXPONENT,
    )?;

    msg!(
        "Price Feed Result: {} - Std. Deviation: {}",
        price_result,
        std_deviation
    );

    match aggregator_account
        .check_confidence_interval(SwitchboardDecimal::from_f64(max_confidence_threshold))
    {
        Ok(()) => Ok(PriceFeedResult::Confident(price_result)),
        Err(_) => {
            let higher_bound = price_result + std_deviation;
            let lower_bound = price_result - std_deviation;
            msg!(
                "Confidence Interval Exceeded - Lower Bound: {} - Higher Bound: {}",
                lower_bound,
                higher_bound
            );
            Ok(PriceFeedResult::Bounded(BoundedPrice {
                price: price_result,
                higher_bound,
                lower_bound,
            }))
        }
    }
}

/// Gets an asset's price from a Pyth [`PriceFeed`] and applies bounds if necessary.
pub fn get_pyth_price(
    price_feed: &PriceFeed,
    max_confidence_threshold: u64,
    unix_timestamp: i64,
) -> Result<PriceFeedResult> {
    let price = match price_feed.get_price_no_older_than(unix_timestamp, ORACLE_PRICE_FEED_TTL_SECS)
    {
        Some(p) => p,
        None => {
            return Err(ErrorCode::StaleOracleFeed.into());
        }
    };
    let price_result = scale_price(
        price.price as f64,
        PYTH_FEED_DECIMALS,
        price.expo,
        ORACLE_PRICE_TARGET_EXPONENT,
    )?;
    let std_deviation = scale_price(
        price.conf as f64,
        PYTH_FEED_DECIMALS,
        price.expo,
        ORACLE_PRICE_TARGET_EXPONENT,
    )?;

    msg!(
        "Price Feed Result: {} - Std. Deviation: {}",
        price_result,
        std_deviation
    );

    if price.conf < max_confidence_threshold {
        Ok(PriceFeedResult::Confident(price_result))
    } else {
        let higher_bound = price_result + std_deviation;
        let lower_bound = price_result - std_deviation;
        msg!(
            "Confidence Interval Exceeded - Lower Bound: {} - Higher Bound: {}",
            lower_bound,
            higher_bound
        );
        Ok(PriceFeedResult::Bounded(BoundedPrice {
            price: price_result,
            higher_bound,
            lower_bound,
        }))
    }
}

#[cfg(test)]
mod tests {
    use pyth_sdk_solana::state::{
        AccountType, PriceAccount, PriceInfo, PriceStatus, MAGIC, VERSION_2,
    };
    use switchboard_v2::{
        aggregator::{AggregatorResolutionMode, Hash},
        AggregatorRound,
    };

    use super::*;

    #[test]
    pub fn test_scale_pyth_price() -> Result<()> {
        let prices = [
            2_292_133_500_000f64, // btcusd from pyth
            159_405_889_054f64,   // ethusd from pyth
            2_428_596_998f64,     // solusd from pyth
        ];
        let current_exponents = [-8, -8, -8];
        let target_exponents = [-10, -10, -10];
        let results = [
            229_213_350_000_000u128,
            15_940_588_905_400u128,
            242_859_699_800u128,
        ];

        for (idx, price) in prices.iter().enumerate() {
            let scaled_price =
                scale_price(*price, 0, current_exponents[idx], target_exponents[idx])?;
            assert_eq!(scaled_price as u128, results[idx]);
        }

        Ok(())
    }

    #[test]
    pub fn test_scale_switchboard_price() -> Result<()> {
        let prices = [
            0.0000010379908497333, // bonk from sbv2
            0.95275399047246,      // orca from sbv2
            0.326177379,           // srm from sbv2
        ];
        let current_exponents = [-0, 0, 0];
        let target_exponents = [-10, -10, -10];
        let results = [10_379, 9_527_539_904, 3_261_773_790];

        for (idx, price) in prices.iter().enumerate() {
            let scaled_price =
                scale_price(*price, 0, current_exponents[idx], target_exponents[idx])?;
            assert_eq!(scaled_price, results[idx]);
        }

        Ok(())
    }

    #[test]
    pub fn test_get_pyth_price_confident() -> Result<()> {
        let price_account = PriceAccount {
            magic: MAGIC,
            ver: VERSION_2,
            atype: AccountType::Price as u32,
            expo: -8,
            agg: PriceInfo {
                price: 2_292_133_500_000,
                conf: 335_477_026,
                status: PriceStatus::Trading,
                pub_slot: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let price_feed = price_account.to_price_feed(&Pubkey::default());

        let price_feed_result = get_pyth_price(&price_feed, 500_000_000, 15)?;

        assert!(price_feed_result == PriceFeedResult::Confident(229_213_350_000_000u128));

        Ok(())
    }

    #[test]
    pub fn test_get_pyth_price_bounded() -> Result<()> {
        let price_account = PriceAccount {
            magic: MAGIC,
            ver: VERSION_2,
            atype: AccountType::Price as u32,
            expo: -8,
            agg: PriceInfo {
                price: 2_292_133_500_000,
                conf: 335_477_026,
                status: PriceStatus::Trading,
                pub_slot: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let price_feed = price_account.to_price_feed(&Pubkey::default());

        let price_feed_result = get_pyth_price(&price_feed, 250_000_000, 15)?;

        assert!(
            price_feed_result
                == PriceFeedResult::Bounded(BoundedPrice {
                    price: 229_213_350_000_000,
                    lower_bound: 229_179_802_297_400,
                    higher_bound: 229_246_897_702_600
                })
        );

        Ok(())
    }

    #[test]
    pub fn test_get_switchboard_price_confident() -> Result<()> {
        let aggregator_account_data = AggregatorAccountData {
            name: [u8::default(); 32],
            metadata: [u8::default(); 128],
            _reserved1: [u8::default(); 32],
            queue_pubkey: Pubkey::default(),
            oracle_request_batch_size: u32::default(),
            min_oracle_results: u32::default(),
            min_job_results: u32::default(),
            min_update_delay_seconds: u32::default(),
            start_after: i64::default(),
            variance_threshold: SwitchboardDecimal::default(),
            force_report_period: i64::default(),
            expiration: i64::default(),
            consecutive_failure_count: u64::default(),
            next_allowed_update_time: i64::default(),
            is_locked: false,
            crank_pubkey: Pubkey::default(),
            latest_confirmed_round: AggregatorRound {
                result: SwitchboardDecimal::from_f64(22_921.335),
                std_deviation: SwitchboardDecimal::from_f64(3.35477026),
                ..Default::default()
            },
            previous_confirmed_round_result: SwitchboardDecimal::from_f64(22_921.335),
            previous_confirmed_round_slot: 0,
            disable_crank: false,
            job_weights: [u8::default(); 16],
            creation_timestamp: i64::default(),
            resolution_mode: AggregatorResolutionMode::ModeRoundResolution,
            _ebuf: [u8::default(); 138],
            current_round: AggregatorRound {
                result: SwitchboardDecimal::from_f64(22_921.335),
                std_deviation: SwitchboardDecimal::from_f64(3.35477026),
                ..Default::default()
            },
            job_pubkeys_data: [Pubkey::default(); 16],
            job_hashes: [Hash::default(); 16],
            job_pubkeys_size: u32::default(),
            jobs_checksum: [u8::default(); 32],
            authority: Pubkey::default(),
            history_buffer: Pubkey::default(),
        };
        let price_feed_result = get_switchboard_price(&aggregator_account_data, 5.0, 15)?;

        assert!(price_feed_result == PriceFeedResult::Confident(229_213_350_000_000));

        Ok(())
    }

    #[test]
    pub fn test_get_switchboard_price_bounded() -> Result<()> {
        let aggregator_account_data = AggregatorAccountData {
            name: [u8::default(); 32],
            metadata: [u8::default(); 128],
            _reserved1: [u8::default(); 32],
            queue_pubkey: Pubkey::default(),
            oracle_request_batch_size: u32::default(),
            min_oracle_results: u32::default(),
            min_job_results: u32::default(),
            min_update_delay_seconds: u32::default(),
            start_after: i64::default(),
            variance_threshold: SwitchboardDecimal::default(),
            force_report_period: i64::default(),
            expiration: i64::default(),
            consecutive_failure_count: u64::default(),
            next_allowed_update_time: i64::default(),
            is_locked: false,
            crank_pubkey: Pubkey::default(),
            latest_confirmed_round: AggregatorRound {
                result: SwitchboardDecimal::from_f64(22_921.335),
                std_deviation: SwitchboardDecimal::from_f64(3.35477026),
                ..Default::default()
            },
            previous_confirmed_round_result: SwitchboardDecimal::from_f64(22_921.335),
            previous_confirmed_round_slot: 0,
            disable_crank: false,
            job_weights: [u8::default(); 16],
            creation_timestamp: i64::default(),
            resolution_mode: AggregatorResolutionMode::ModeRoundResolution,
            _ebuf: [u8::default(); 138],
            current_round: AggregatorRound {
                result: SwitchboardDecimal::from_f64(22_921.335),
                std_deviation: SwitchboardDecimal::from_f64(3.35477026),
                ..Default::default()
            },
            job_pubkeys_data: [Pubkey::default(); 16],
            job_hashes: [Hash::default(); 16],
            job_pubkeys_size: u32::default(),
            jobs_checksum: [u8::default(); 32],
            authority: Pubkey::default(),
            history_buffer: Pubkey::default(),
        };
        let price_feed_result = get_switchboard_price(&aggregator_account_data, 2.5, 15)?;

        assert!(
            price_feed_result
                == PriceFeedResult::Bounded(BoundedPrice {
                    price: 229_213_350_000_000,
                    lower_bound: 229_179_802_297_400,
                    higher_bound: 229_246_897_702_600
                })
        );

        Ok(())
    }
}
