use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint, TokenAccount};
use crate::state::pool::*; 
use crate::math::tick_math::*;

// Sqrt_price = sqrt(price) * 2^64
#[derive(Accounts)]
#[instruction(sqrt_price_x64: u128, tick_spacing: u16)]
pub struct InitializePool<'info> {
    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [
            b"pool",
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
        ],
        bump,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 16 + 4 + 16 + 2 + 1 + 16 + 16 + 16 + 16 + 1,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = admin,
        token::mint = token_mint_a,
        token::authority = pool,
    )]
    pub token_vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = token_mint_b,
        token::authority = pool,
    )]
    pub token_vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_pool_handler(
    ctx: Context<InitializePool>,
    sqrt_price_x64: u128,
    tick_spacing: u16,
    fee: u8,
    fee_growth_global_a: u128,
    fee_growth_global_b: u128,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // Initialize pool state with provided values
    pool.pool_authority = ctx.accounts.admin.key();
    pool.token_a_mint = ctx.accounts.token_mint_a.key();
    pool.token_b_mint = ctx.accounts.token_mint_b.key();
    pool.token_a_vault = ctx.accounts.token_vault_a.key();
    pool.token_b_vault = ctx.accounts.token_vault_b.key();
    pool.sqrt_price = sqrt_price_x64;
    pool.tick_current = get_tick_from_sqrt_price(sqrt_price_x64)?;
    pool.liquidity = 0;
    pool.tick_spacing = tick_spacing;
    pool.fee = fee;
    pool.fee_growth_global_a = fee_growth_global_a;
    pool.fee_growth_global_b = fee_growth_global_b;
    pool.protocol_fee_a = 0;
    pool.protocol_fee_b = 0;
    pool.bump = ctx.bumps.pool;
    pool.initialized_ticks = vec![pool.tick_current];


    Ok(())
}
