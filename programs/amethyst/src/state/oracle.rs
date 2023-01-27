use anchor_lang::prelude::*;
use jet_proto_proc_macros::assert_size;

#[derive(Debug, PartialEq)]
pub enum PriceFeedResult {
    /// Price is within the maximum confidence interval threshold.
    Confident(f64),
    /// The maximum confidence interval thresholds were exceeded, thus price is bounded.
    Bounded(BoundedPrice),
}

#[derive(Debug, PartialEq)]
pub struct BoundedPrice {
    /// The price.
    pub price: f64,
    /// The lower bound price.
    pub lower_bound: f64,
    /// The higher bound price.
    pub higher_bound: f64,
}

#[assert_size(aligns, 40)]
#[derive(Debug)]
pub struct SwitchboardOracleInfo {
    /// The aggregator account.
    pub aggregator_account: Pubkey,
    /// The maximum confidence interval threshold.
    pub max_confidence_interval: f64,
}

#[assert_size(aligns, 40)]
#[derive(Debug)]
pub struct PythOracleInfo {
    /// The pyth price account.
    pub price_account: Pubkey,
    /// The maximum confidence interval threshold.
    pub max_confidence_interval: u64,
}