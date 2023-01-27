use crate::constants::QUOTE_TOKEN_DECIMALS;

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
    let usd_amount = token_amount.checked_mul(token_price.into()).unwrap();
    let scaled_usd_amount = scale_amount(usd_amount, QUOTE_TOKEN_DECIMALS as i32, -8, 0)?;
    Ok(adjust_decimals(scaled_usd_amount, decimals))
}

/// Converts a USD amount to a token amount according to the given token price.
pub fn usd_to_token(usd_amount: u128, token_price: u64, decimals: u8) -> Result<u128> {
    // first we scale the usd amount to -6 exponent
    let scaled_usd_amount = scale_amount(usd_amount, QUOTE_TOKEN_DECIMALS as i32, 0, -6)?;
    let token_amount = scaled_usd_amount.checked_div(token_price.into()).unwrap();
    // finally we adjust token amount for 6 decimals on the quote token
    let adjusted_token_amount = adjust_decimals(token_amount, decimals);
    Ok(adjusted_token_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_adjust_decimals() -> Result<()> {
        let scaled_token_value = 106_000_000_000;
        // e.g 1_000_000 BONK in native token amount scaled from -8 exponent to 0 exponent

        let adjusted_amount = adjust_decimals(scaled_token_value, 5);

        assert_eq!(adjusted_amount, 1_060_000);

        Ok(())
    }

    #[test]
    pub fn test_token_to_usd() -> Result<()> {
        let token_price = 106;
        let token_amount = 100_000_000_000;
        // e.g 1_000_000 BONK in native token amount scaled from -8 exponent to 0 exponent

        let usd_amount = token_to_usd(token_amount, token_price, 5)?;

        assert_eq!(usd_amount, 1_060_000);

        Ok(())
    }

    #[test]
    pub fn test_usd_to_token() -> Result<()> {
        let usd_amount = 1_060_000;
        let token_price = 106;

        let token_amount = usd_to_token(usd_amount, token_price, 5)?;

        assert_eq!(token_amount, 100_000_000_000);

        Ok(())
    }
}
