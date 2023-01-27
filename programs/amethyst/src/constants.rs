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
