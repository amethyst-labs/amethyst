pub mod args;
pub mod constants;
pub mod contexts;
pub mod error;
pub mod events;
pub mod ids;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use args::*;
use instructions::*;
use state::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod amethyst {
    use super::*;

    pub fn change_position_collateral(
        ctx: Context<ChangePositionCollateral>,
        value_change: PositionValueChange,
    ) -> Result<()> {
        instructions::change_position_collateral::handler(ctx, value_change)
    }

    pub fn change_position_size(
        ctx: Context<ChangePositionSize>,
        value_change: PositionValueChange,
    ) -> Result<()> {
        instructions::change_position_size::handler(ctx, value_change)
    }

    pub fn close_global_cache(ctx: Context<CloseGlobalCache>) -> Result<()> {
        instructions::close_global_cache::handler(ctx)
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::close_position::handler(ctx)
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        instructions::close_vault::handler(ctx)
    }

    pub fn create_global_cache(ctx: Context<CreateGlobalCache>) -> Result<()> {
        instructions::create_global_cache::handler(ctx)
    }

    pub fn create_vault(ctx: Context<CloseVault>) -> Result<()> {
        instructions::close_vault::handler(ctx)
    }

    pub fn enter_position(ctx: Context<EnterPosition>, args: EnterPositionArgs) -> Result<()> {
        instructions::enter_position::handler(ctx, args)
    }

    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        instructions::liquidate_position::handler(ctx)
    }

    pub fn pay_funding(ctx: Context<PayFunding>) -> Result<()> {
        instructions::pay_funding::handler(ctx)
    }

    pub fn swap(ctx: Context<Swap>, args: SwapArgs) -> Result<()> {
        instructions::swap::handler(ctx, args)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
