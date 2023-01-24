use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq, Eq)]
pub enum ErrorCode {
    #[msg("The provided signer is invalid.")]
    InvalidSigner,

    #[msg("The provided token mint is invalid.")]
    InvalidTokenMint,

    #[msg("Timestamp could not be converted to u64.")]
    InvalidTimestampConversion,

    #[msg("The given timestamp does not match desired funding interval.")]
    InvalidFundingInterval,

    #[msg("The given token amount is invalid.")]
    InvalidTokenAmount,

    #[msg("The resulting average price is invalid.")]
    InvalidAveragePrice,

    #[msg("The given vault does not have sufficient liquidity to enter the desired position.")]
    InsufficientLiquidityToEnterPosition,

    #[msg("The given vault does not have sufficient liquidity to performt the swap.")]
    InsufficientLiquidityForSwap,

    #[msg("The given vault has deposited assets.")]
    CannotCloseVaultWithDepositedAssets,

    #[msg("The given vault has reserved assets.")]
    CannotCloseVaultWithReservedAssets,

    #[msg("The position's leverage would exceed the maximum leverage allowed.")]
    PositionLeverageExceedsLimit,
}
