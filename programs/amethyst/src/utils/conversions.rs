use crate::constants::{
    ORACLE_PRICE_TARGET_EXPONENT, QUOTE_TOKEN_DECIMALS, USD_CONVERSION_TARGET_EXPONENT,
};

use super::scale_amount;
use anchor_lang::prelude::*;

/// Adjusts the given amount according to decimals.
pub fn adjust_decimals(amount: u128, decimals: u8) -> u128 {
    let decimal_adj = 10u64.pow(decimals as u32);
    amount.checked_div(decimal_adj.into()).unwrap()
}

/// Converts the given amount to native units according to the given decimals.
pub fn to_native_amount(amount: u128, decimals: u8) -> u128 {
    let decimal_adj = 10u64.pow(decimals as u32);
    amount.checked_mul(decimal_adj.into()).unwrap()
}

/// Converts a token amount to a USD amount according to the given token price.
pub fn token_to_usd(token_amount: u128, token_price: u64, decimals: u8) -> Result<u128> {
    #[cfg(test)]
    msg!("t2q - Token Amount: {}", token_amount);

    let usd_amount = token_amount.checked_mul(token_price.into()).unwrap();
    #[cfg(test)]
    msg!("t2q - USD Amount: {}", usd_amount);

    let adjusted_amount = adjust_decimals(usd_amount, decimals);
    #[cfg(test)]
    msg!("t2q - Adjusted USD Amount: {}", adjusted_amount);

    let scaled_usd_amount = scale_amount(
        adjusted_amount,
        QUOTE_TOKEN_DECIMALS as i32,
        ORACLE_PRICE_TARGET_EXPONENT,
        USD_CONVERSION_TARGET_EXPONENT,
    )?;
    #[cfg(test)]
    msg!("t2q - Scaled USD Amount: {}", scaled_usd_amount);
    Ok(scaled_usd_amount)
}

/// Converts a USD amount to a token amount according to the given token price.
pub fn usd_to_token(usd_amount: u128, token_price: u64, decimals: u8) -> Result<u128> {
    #[cfg(test)]
    msg!("q2t - USD Amount: {}", usd_amount);

    // first we scale the usd amount
    let scaled_usd_amount = scale_amount(
        usd_amount,
        0,
        -(QUOTE_TOKEN_DECIMALS as i32),
        ORACLE_PRICE_TARGET_EXPONENT,
    )?;
    #[cfg(test)]
    msg!("q2t - Scaled USD Amount: {}", scaled_usd_amount);

    let token_amount_scaled = scaled_usd_amount.checked_div(token_price.into()).unwrap();
    #[cfg(test)]
    msg!("q2t - Token Amount Scaled: {}", token_amount_scaled);

    // finally we adjust token amount for 6 decimals on the quote token
    let adjusted_token_amount = to_native_amount(token_amount_scaled, decimals);
    #[cfg(test)]
    msg!("q2t - Adjusted Token Amount: {}", adjusted_token_amount);
    Ok(adjusted_token_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_adjust_decimals() -> Result<()> {
        // e.g value of 1_000_000 BONK in native token amount using a price feed scaled to -10 exponent
        let scaled_token_value = 106_000_000_000;

        let adjusted_amount = adjust_decimals(scaled_token_value, 5);

        assert_eq!(adjusted_amount, 1_060_000);

        Ok(())
    }

    #[test]
    pub fn test_to_native_amount() -> Result<()> {
        let scaled_token_value = 1_060_000;

        let adjusted_amount = to_native_amount(scaled_token_value, 5);

        assert_eq!(adjusted_amount, 106_000_000_000);

        Ok(())
    }

    #[test]
    pub fn test_token_to_usd() -> Result<()> {
        let inputs = [
            (100_000_000_000u128, 10_600u64), // e.g 1_000_000 BONK in native token amount
            (1_000_000_000_000_000u128, 242_859_699_800u64), // e.g 1_000_000 SOL in native token amount
            (100_000_000_000_000u128, 15_940_588_905_400u64), // e.g 1_000_000 ETH in native token amount
        ];
        let decimals = [5, 9, 8];
        let results = [
            1_060_000u128,             // 1_000_000 bonk x 0.00000106.. = 1.06 usdc
            24_285_969_980_000u128,    // 1_000_000 sol x 24.28596.. = 24_285_969.980 usdc
            1_594_058_890_540_000u128, // 1_000_000 eth x 1594.0588.. = 1_594_058_890.5400 usdc
        ];

        for (idx, (amount, price)) in inputs.iter().enumerate() {
            let usd_value = token_to_usd(*amount, *price, decimals[idx])?;

            assert_eq!(usd_value, results[idx]);
        }

        Ok(())
    }

    #[test]
    pub fn test_usd_to_token() -> Result<()> {
        let inputs = [
            (1_060_000u128, 10_600u64), // e.g 1_000_000 BONK in native usd token amount
            (24_285_969_980_000u128, 242_859_699_800u64), // e.g 1_000_000 SOL in native usd token amount
            (1_594_058_890_540_000u128, 15_940_588_905_400u64), // e.g 1_000_000 ETH in native usd token amount
        ];
        let decimals = [5, 9, 8];
        let results = [
            100_000_000_000u128,       // 1_000_000_000_000
            1_000_000_000_000_000u128, // 100_000_000_000_000
            100_000_000_000_000u128,
        ];

        for (idx, (amount, price)) in inputs.iter().enumerate() {
            let token_amount = usd_to_token(*amount, *price, decimals[idx])?;
            assert_eq!(token_amount, results[idx]);
        }
        // // e.g value of 1_000_000 BONK
        // let usd_amount = 1_037_900;
        // // e.g BONK in price scaled to -10 exponent
        // let token_price = 10_379;

        // let token_amount = usd_to_token(usd_amount, token_price, 5)?;

        // assert_eq!(token_amount, 100_000_000_000);

        Ok(())
    }
}
