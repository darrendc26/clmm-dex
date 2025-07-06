#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod math;
pub mod errors;

use instructions::*;

declare_id!("DPphpdUTwYSGBAG7oytXFNsVu58zBFhyJmdRfhmkuPTY");

#[program]
pub mod clmm_dex {
    use super::*;

    // Initialize pool
    pub fn initialize_pool(ctx: Context<InitializePool>,
         sqrt_price_x64: u128,
         tick_spacing: u16, 
         fee: u8,
         fee_growth_global_a: u128,
         fee_growth_global_b: u128,
        ) -> Result<()> {
            initialize_pool_handler(ctx, sqrt_price_x64, tick_spacing, fee, fee_growth_global_a, fee_growth_global_b)
    }

// Provide liquidity
    pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, 
        tick_lower: i32, 
        tick_upper: i32, 
        liquidity: u128
    ) -> Result<()> {
        provide_liquidity_handler(ctx, tick_lower, tick_upper, liquidity)
    }   

    // Remove liquidity
    pub fn remove_liquidity(ctx: Context<RemoveLiquidity>
    ) -> Result<()> {
        remove_liquidity_handler(ctx)
    }

    // Swap
    pub fn swap(ctx: Context<Swap>, 
        amount_in: u64, 
        a_to_b: bool
    ) -> Result<()> {
        swap_handler(ctx, amount_in, a_to_b)
    }
}


