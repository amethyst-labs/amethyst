use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::B_ESCROW,
    contexts::{impl_change_position_ctx, ChangePositionContext},
    error::ErrorCode,
    events::PositionIncreased,
    state::{
        position::{Direction, Position},
        vault::{Vault, VaultCache},
        Config,
    },
    utils::{get_next_average_price, token_to_usd, usd_to_token},
};

#[derive(Accounts)]
pub struct IncreasePosition<'info> {
    /// The config.
    pub config: Box<Account<'info, Config>>,

    /// The vault's cache.
    #[account(
        mut,
        has_one = vault
    )]
    pub vault_cache: Box<Account<'info, VaultCache>>,

    /// The vault of an asset.
    #[account(
        mut,
        has_one = token_mint,
        has_one = token_vault,
        has_one = vault_signer,
    )]
    pub vault: Box<Account<'info, Vault>>,

    /// The vault's token account.
    #[account(
        mut,
        token::authority = vault.vault_signer,
        token::mint = token_mint
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    /// The vault's token account authority.
    /// CHECK: The vault signer is checked.
    pub vault_signer: AccountInfo<'info>,

    /// The user's position.
    #[account(
        mut,
        has_one = authority,
    )]
    pub position: Box<Account<'info, Position>>,

    /// The escrow of the position.
    /// CHECK: The escrow PDA for the position.
    #[account(
        seeds = [
            B_ESCROW,
            position.key().as_ref(),
            token_mint.key().as_ref()
        ],
        bump = position.escrow_bump_seed[0]
    )]
    pub escrow: AccountInfo<'info>,

    /// The position's token account.
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = escrow
    )]
    pub position_vault: Box<Account<'info, TokenAccount>>,

    /// The user's token account.
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = authority,
    )]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    /// The position's asset.
    pub token_mint: Box<Account<'info, Mint>>,

    /// The position's authority.
    pub authority: Signer<'info>,

    /// The payer of the transaction fee and rent of accounts being initialised.
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> IncreasePosition<'info> {
    /// Perform validation.
    ///
    /// In this case we want to validate that the size delta is valid.
    fn validate(&self, size_delta: u64) -> Result<()> {
        require!(size_delta != 0, ErrorCode::InvalidSizeDelta);
        Ok(())
    }

    /// Increase the position size
    fn process(&mut self, clock: &Clock, size_delta: u64) -> Result<()> {
        let unix_timestamp: u64 = clock
            .unix_timestamp
            .try_into()
            .or(Err(ErrorCode::InvalidTimestampConversion))?;
        // update the funding rate of the vault before proceeding
        self.vault.update_funding_rate(clock)?;

        let decimals = self.vault.decimals;
        // get the latest price from the vault cache
        let price = 0;

        // if the position size is zero, the avg entry price is the current price
        if self.position.size == 0 {
            self.position.avg_entry_price = 0;
        }

        // if the position size isn't zero and the size delta
        if self.position.size != 0 && size_delta != 0 {
            self.position.avg_entry_price = get_next_average_price(
                self.position.size.into(),
                self.position.avg_entry_price.into(),
                size_delta.into(),
                price, // todo
            )?;
        }

        // colelct margin fees
        let fee = self.vault.collect_margin_fees(
            &self.config,
            &self.vault_cache,
            size_delta,
            self.position.size,
            self.position.last_funding_index,
        )?;
        // transfer collateral and
        let collateral_delta = 0;
        let collateral_delta_usd = token_to_usd(collateral_delta, price, decimals)?;

        self.position.collateral += collateral_delta_usd as u64;
        require!(
            self.position.collateral as u128 >= fee,
            ErrorCode::InsufficientCollateralForFee
        );
        self.position.collateral = self.position.collateral.checked_sub(fee as u64).unwrap();
        self.position.size += size_delta;
        self.position.last_funding_index = self.vault_cache.funding_index;
        self.position.last_funding_payment = unix_timestamp;

        let reserve_delta = usd_to_token(size_delta.into(), price, decimals)?;
        self.position.reserved_amount = reserve_delta as u64;

        // TODO: add checks in [increase_reserved] for reserved <= pool_amount
        self.vault.increase_reserved(reserve_delta);

        match self.position.direction {
            Direction::Long => {
                // increase the amount of (position.size - position.collateral)
                // if a fee is a charged on the collateral then this value needs to increase by the same amount
                // since (position.size - position.collateral) will have increased that much
                self.vault
                    .increase_guaranteed_usd((size_delta as u128 + fee).into());
                self.vault
                    .decrease_guaranteed_usd(collateral_delta_usd.into())?;
                // treat the collateral as part of the pool
                self.vault.increase_pool_amount(collateral_delta.into());
                // fees need to be taken from the pool since they are taken from collateral
                // and collateral is part of the pool
                self.vault
                    .decrease_pool_amount(usd_to_token(fee, price, decimals)?)?;
            }
            Direction::Short => {
                if self.vault_cache.short_open_interest == 0 {
                    self.vault_cache.short_avg_entry_price = 0; // todo
                } else {
                    // calculate next short average entry price and increase short open interest
                    let _ = self
                        .vault_cache
                        .get_next_short_average_entry_price(size_delta)?;
                }
            }
        }

        emit!(PositionIncreased {
            position: self.position.key(),
            authority: self.authority.key(),
            size_delta: size_delta.into(),
            collateral_delta_usd,
            fee,
            price
        });

        Ok(())
    }

    /// Perform validation after performing an action.
    ///
    /// In this specific case we need to validate that, after potentially withdrawing collateral
    /// from the position, we do not exceed the maximum allowed leverage for this vault.
    fn post_validation(&self) -> Result<()> {
        let leverage = self.position.leverage();
        require!(
            leverage < self.vault.max_leverage.into(),
            ErrorCode::PositionLeverageExceedsLimit
        );
        Ok(())
    }
}

pub fn handler(ctx: Context<IncreasePosition>, size_delta: u64) -> Result<()> {
    let clock = Clock::get()?;
    ctx.accounts.validate(size_delta)?;
    ctx.accounts.process(&clock, size_delta)?;
    ctx.accounts.post_validation()?;
    Ok(())
}

impl_change_position_ctx! { IncreasePosition<'info> }
