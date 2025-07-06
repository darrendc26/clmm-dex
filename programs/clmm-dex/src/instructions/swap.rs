use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount};
use crate::state::*;
use crate::errors::*;
use crate::math::tick_math::*;
use crate::math::compute_swap_step::*;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut,
        seeds = [
            b"position",
            pool.key().as_ref(),
            user.key().as_ref(),
            position.tick_lower.to_le_bytes().as_ref(),
            position.tick_upper.to_le_bytes().as_ref(),
        ],
        bump = position.bump,
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub user_token_a_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub pool_token_a_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_token_b_vault: Account<'info, TokenAccount>,
    pub pool_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}


pub fn swap_handler(ctx: Context<Swap>, amount_in: u64, a_to_b: bool) -> Result<()> {
    let initialized_ticks = &ctx.accounts.pool.initialized_ticks.clone();
    let pool = &mut ctx.accounts.pool;
    let mut tick_current = pool.tick_current;
    let mut sqrt_price = pool.sqrt_price;
    let mut liquidity = pool.liquidity as u128;
    let mut amount_remaining = amount_in as u128;
    let mut total_amount_out: u128 = 0;
    let fee_rate = pool.fee as u128;

    // Iterate through ticks until we exhaust the amount_in
    while amount_remaining > 0 {
        // Get next tick
        let next_tick = get_next_tick(tick_current, initialized_ticks, a_to_b)?;
        // Get next sqrt price
        let next_sqrt_price = get_sqrt_price_from_tick(next_tick)?;

        // Calculate swap step and how much amount_in, out, and fees we need to pay
        let (step_in, step_out, next_price, fee_amount) = compute_swap_step(
            sqrt_price,
            next_sqrt_price,
            liquidity,
            amount_remaining,
            fee_rate,
            true,
        )?;
        // Update amount remaining and total amount out
        amount_remaining = amount_remaining
            .checked_sub(step_in + fee_amount)
            .ok_or(Errors::MathError)?;

        total_amount_out = total_amount_out
            .checked_add(step_out)
            .ok_or(Errors::MathError)?;

        sqrt_price = next_price;
        tick_current = next_tick;

        // Update fee growth global for a and b based on direction of swap
        if a_to_b {
            pool.fee_growth_global_a = pool.fee_growth_global_a
                .checked_add(fee_amount)
                .ok_or(Errors::MathError)?;
        } else {
            pool.fee_growth_global_b = pool.fee_growth_global_b
                .checked_add(fee_amount)
                .ok_or(Errors::MathError)?;
        }

        if sqrt_price == next_sqrt_price {
            // We're crossing a tick → update liquidity
            // Use the account info directly without storing it
            let tick_account_info = &ctx.remaining_accounts[0];

            // Deserialize tick account from account data
            let mut tick_account = Tick::try_deserialize(&mut &tick_account_info.data.borrow()[..])?;
            
            // Validate that this is the correct tick account
            require_eq!(tick_account.tick_index, next_tick, Errors::InvalidTick);
            

            let fee_growth_global_a = pool.fee_growth_global_a;
            let fee_growth_global_b = pool.fee_growth_global_b;

            // Update tick account in terms of fee 
            cross_tick(&mut tick_account,fee_growth_global_a, fee_growth_global_b, a_to_b)?;

            // Serialize tick account back to account data
            tick_account.serialize(&mut &mut tick_account_info.data.borrow_mut()[..])?;

            // Update liquidity
            let liquidity_net = tick_account.liquidity_net;
            liquidity = if a_to_b {
                liquidity.checked_sub(liquidity_net as u128)
            } else {
                liquidity.checked_add(liquidity_net as u128)
            }
            .ok_or(Errors::MathError)?;
        } else {
            // Swap is done before crossing the tick
            break;
        }
    }

    // Update pool state
    pool.tick_current = tick_current;
    pool.sqrt_price = sqrt_price;
    pool.liquidity = liquidity;

    // Perform token transfers
    if a_to_b {
        // user pays A, receives B
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_a_account.to_account_info(),
            to: ctx.accounts.pool_token_a_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount_in,
        )?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_b_vault.to_account_info(),
            to: ctx.accounts.user_token_b_account.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            total_amount_out as u64,
            )?;

    } else {
        // user pays B, receives A
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_b_account.to_account_info(),
            to: ctx.accounts.pool_token_b_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
            amount_in,
            )?;

            let cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_a_vault.to_account_info(),
                to: ctx.accounts.user_token_a_account.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            };
            token::transfer(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts),
                total_amount_out as u64,
                )?;
        }
    
    Ok(())
}

