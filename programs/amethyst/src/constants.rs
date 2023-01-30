/// The decimals of a pyth price feed.
pub const PYTH_FEED_DECIMALS: i32 = 0;

/// The current exponent of a switchboard price feed.
pub const SWITCHBOARD_FEED_EXPONENT: i32 = 0;
/// The decimals of a switchboard price feed.
pub const SWITCHBOARD_FEED_DECIMALS: i32 = 0;

/// The target exponent for oracle price.
/// A large exponent is used so we can easily retain accuracy when consuming price feeds
/// where the value of the underlying is very small, e.g BONK.
pub const ORACLE_PRICE_TARGET_EXPONENT: i32 = -10;

/// The target exponent when scaling a USD value amount while performing conversion from native
/// token amount to USD amount.
pub const USD_CONVERSION_TARGET_EXPONENT: i32 = 0;

/// The current exponent when scaling a USD value amount while performing conversion from USD amount
/// to native token amount.
pub const TOKEN_CONVERSION_CURRENT_EXPONENT: i32 = 0;

/// The decimals of the quote token.
pub const QUOTE_TOKEN_DECIMALS: u32 = 6;

/// The number of seconds in an hour.
pub const SECONDS_IN_HOUR: u64 = 3600;

/// The basis points divisor.
pub const BASIS_POINTS_DIVISOR: u64 = 10_000;

/// The time to live for an oracle price, in slots.
pub const ORACLE_PRICE_TTL_SLOTS: u64 = 30;

/// The time to live for the price coming from an oracle feed, in seconds.
pub const ORACLE_PRICE_FEED_TTL_SECS: u64 = 30;

/// The [`Position`] seed.
pub const B_POSITION: &[u8] = b"POSITION";
/// The [`Position`]'s escrow seed.
pub const B_ESCROW: &[u8] = b"ESCROW";
/// The [`Position's`] escrow token account seed.
pub const B_ESCROW_TOKEN_ACCOUNT: &[u8] = b"ESCROW_TOKEN_ACCOUNT";
