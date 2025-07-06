use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount};

use crate::state::{Pool, Position, Tick};
use crate::math::compute_amount::*;
use crate::errors::{ Errors };


#[derive(Accounts)]
#[instruction(tick_lower: i32, tick_upper: i32, liquidity: u64)]
pub struct ProvideLiquidity<'info> {
    #[account(mut,
        seeds = [
            b"pool",
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(init, payer = owner,
        seeds = [
            b"position",
            pool.key().as_ref(),
            owner.key().as_ref(),
            tick_lower.to_le_bytes().as_ref(),
            tick_upper.to_le_bytes().as_ref(),
        ],
        bump,
        space = 8 + 32 + 32 + 32 + 32 + 16 + 4 + 8 + 2 + 1, // Position account size
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = pool,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = pool,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = owner,
    )]
    pub owner_token_a: Account<'info, TokenAccount>,

    #[account(mut, 
    token::mint = pool.token_b_mint,
    token::authority = owner,
    )]
    pub owner_token_b: Account<'info, TokenAccount>,

    #[account(mut,
    seeds = [b"tick", pool.key().as_ref(), &tick_lower.to_le_bytes()],
    bump,
    )]
    pub lower_tick: Account<'info, Tick>,

    #[account(mut,
    seeds = [b"tick", pool.key().as_ref(), &tick_upper.to_le_bytes()],
    bump,
    )]
    pub upper_tick: Account<'info, Tick>,

    
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
        
}


pub fn provide_liquidity_handler(
    ctx: Context<ProvideLiquidity>,
    tick_lower: i32,
    tick_upper: i32,
    liquidity: u128,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let owner = &ctx.accounts.owner;
    let lower_tick = &mut ctx.accounts.lower_tick;
    let upper_tick = &mut ctx.accounts.upper_tick;

    // Initialize position
    position.owner = owner.key();
    position.tick_lower = tick_lower;
    position.tick_upper = tick_upper;
    position.liquidity = liquidity;
    position.token_a_earned = 0;
    position.token_b_earned = 0;
    position.fee_growth_inside_a = 0;
    position.fee_growth_inside_b = 0;
    position.bump = ctx.bumps.position;

    // Initialize lower tick if not already
    if !lower_tick.initialized {
        lower_tick.tick_index = tick_lower;
        lower_tick.liquidity_net = 0;
        lower_tick.fee_growth_outside_a = 0;
        lower_tick.fee_growth_outside_b = 0;
        lower_tick.initialized = true;
        lower_tick.bump = ctx.bumps.lower_tick;
        if !pool.initialized_ticks.contains(&tick_lower) {
            pool.initialized_ticks.push(tick_lower);
            pool.initialized_ticks.sort();
        }
    }
    lower_tick.liquidity_net = lower_tick
        .liquidity_net
        .checked_add(liquidity as i128)
        .ok_or(Errors::MultiplicationOverflow)?;

    // Initialize upper tick if not already
    if !upper_tick.initialized {
        upper_tick.tick_index = tick_upper;
        upper_tick.liquidity_net = 0;
        upper_tick.fee_growth_outside_a = 0;
        upper_tick.fee_growth_outside_b = 0;
        upper_tick.initialized = true;
        upper_tick.bump = ctx.bumps.upper_tick;
        if !pool.initialized_ticks.contains(&tick_upper) {
            pool.initialized_ticks.push(tick_upper);
            pool.initialized_ticks.sort();
        }
    }
    upper_tick.liquidity_net = upper_tick
        .liquidity_net
        .checked_sub(liquidity as i128)
        .ok_or(Errors::MultiplicationOverflow)?;

    // Update pool liquidity if position is active at current tick
    if tick_lower <= pool.tick_current && pool.tick_current < tick_upper {
        pool.liquidity = pool
            .liquidity
            .checked_add(liquidity)
            .ok_or(Errors::MultiplicationOverflow)?;
    }

    // Compute required amounts for adding liquidity
    let (amount_a, amount_b) = compute_amount(
        liquidity,
        tick_upper,
        tick_lower,
        pool.tick_current,
    )?;

    let amount_a_u64 = u64::try_from(amount_a).map_err(|_| Errors::TokenMaxExceeded)?;
    let amount_b_u64 = u64::try_from(amount_b).map_err(|_| Errors::TokenMaxExceeded)?;

    // Transfer token A
    let cpi_accounts = Transfer {
        from: ctx.accounts.owner_token_a.to_account_info(),
        to: ctx.accounts.token_a_vault.to_account_info(),
        authority: owner.to_account_info(),
    };
    token::transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        amount_a_u64,
    )?;

    // Transfer token B
    let cpi_accounts = Transfer {
        from: ctx.accounts.owner_token_b.to_account_info(),
        to: ctx.accounts.token_b_vault.to_account_info(),
        authority: owner.to_account_info(),
    };
    token::transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
        amount_b_u64,
    )?;

    Ok(())
}
