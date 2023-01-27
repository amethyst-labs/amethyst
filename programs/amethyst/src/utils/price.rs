use std::ops::Add;

use crate::error::ErrorCode;
use anchor_lang::prelude::*;

pub fn get_next_average_price(
    current_position_size: u128,
    current_average_price: u64,
    position_size_delta: u64,
    current_price: u64,
) -> Result<u64> {
    let notional_delta = position_size_delta.checked_mul(current_price).unwrap();
    let avg_price = current_position_size
        .checked_mul(current_average_price.into())
        .and_then(|n| n.checked_add(notional_delta as u128))
        .and_then(|n: u128| n.checked_div(current_position_size.add(position_size_delta as u128)))
        .unwrap();
    avg_price
        .try_into()
        .or(Err(ErrorCode::InvalidAveragePrice.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_next_average_price() -> Result<()> {
        let current_size = 3;
        let current_average_price = 9_666_670_000;
        let current_price = 11_000_000_000;
        let next_average_price =
            get_next_average_price(current_size, current_average_price, 2, current_price)?;
        assert_eq!(next_average_price, 10_200_002_000);
        Ok(())
    }
}
