use crate::{
    constants::BASIS_POINTS_DIVISOR,
    state::{Cache, Config, Vault},
};

/// Gets the position fee.
pub fn get_position_fee(config: &Config, size_delta: u64) -> u64 {
    if size_delta == 0 {
        return 0;
    }
    let after_fee = size_delta
        .checked_mul(
            BASIS_POINTS_DIVISOR
                .checked_sub(config.margin_fee_bps.into())
                .unwrap(),
        )
        .and_then(|n| n.checked_div(BASIS_POINTS_DIVISOR))
        .unwrap();
    size_delta.checked_sub(after_fee).unwrap()
}

/// Gets the funding fee for a given position
pub fn get_funding_fee(vault: &Vault, size: u64, funding_rate: u64) -> u64 {
    if size == 0 {
        return 0;
    }

    let funding_rate = vault
        .cumulative_funding_rate
        .checked_sub(funding_rate.into())
        .unwrap();

    let funding_fee = funding_rate.checked_mul(size.into()).unwrap();
    funding_fee as u64
}

/// Gets the fee to apply for an operation in a vault, denominated in basis points.
pub fn get_fee_bps(
    cache: &Cache,
    vault: &Vault,
    debt_delta: u128,
    fee_bps: u16,
    tax_bps: u16,
    increment: bool,
) -> u64 {
    if !vault.has_dynamic_fees {
        return fee_bps.into();
    }

    let initial_amount = vault.debt_amount;
    let mut next_amount = initial_amount + debt_delta;

    if !increment {
        next_amount = if debt_delta > initial_amount {
            0
        } else {
            initial_amount.checked_sub(debt_delta).unwrap()
        };
    }

    let target_amount = vault.get_target_debt_amount(cache);
    if target_amount == 0 {
        return fee_bps.into();
    }

    let initial_diff = if initial_amount > target_amount {
        initial_amount.checked_sub(target_amount).unwrap()
    } else {
        target_amount.checked_sub(initial_amount).unwrap()
    };

    let next_diff = if next_amount > target_amount {
        next_amount.checked_sub(target_amount).unwrap()
    } else {
        target_amount.checked_sub(next_amount).unwrap()
    };

    if next_diff < initial_diff {
        let rebate_bps = (tax_bps as u128)
            .checked_mul(initial_diff)
            .and_then(|n| n.checked_div(target_amount))
            .unwrap();
        if rebate_bps > fee_bps as u128 {
            return 0;
        } else {
            return (fee_bps as u128).checked_sub(rebate_bps).unwrap() as u64;
        }
    }

    let mut average_diff = (initial_diff + next_diff).checked_div(2).unwrap();
    if average_diff > target_amount {
        average_diff = target_amount;
    }

    let tax_bps = (tax_bps as u128)
        .checked_mul(average_diff)
        .and_then(|n| n.checked_div(target_amount))
        .unwrap();
    fee_bps as u64 + tax_bps as u64
}
