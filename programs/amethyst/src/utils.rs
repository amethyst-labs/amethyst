use crate::error::ErrorCode;
use anchor_lang::prelude::*;
use std::ops::Add;

pub fn with_signer_pda<'info, T: ToAccountInfo<'info>>(acc_info: &T) -> AccountInfo<'info> {
    let mut acc_info = acc_info.to_account_info();
    acc_info.is_signer = true;
    acc_info
}

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
