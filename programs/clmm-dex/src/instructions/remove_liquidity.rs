use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::{ Errors };
use crate::state::{Pool, Position};
use crate::math::compute_amount::*;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(
        mut,
        seeds = [
            b"pool",
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref(),
        ],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [
            b"position",
            pool.key().as_ref(),
            owner.key().as_ref(),
            position.tick_lower.to_le_bytes().as_ref(),
            position.tick_upper.to_le_bytes().as_ref(),
        ],
        close = owner,
        bump = position.bump,
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
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = owner,
    )]
    pub owner_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn remove_liquidity_handler( ctx: Context<RemoveLiquidity>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let position = &mut ctx.accounts.position;

    // Compute amount A and B to remove
    let (amount_a, amount_b) = compute_amount(
        position.liquidity,
        position.tick_lower,
        position.tick_upper,
        pool.tick_current,
    )?;

    let seeds = &[
    b"pool",
    pool.token_a_mint.as_ref(),
    pool.token_b_mint.as_ref(),
    &[pool.bump]];

    let signer = &[&seeds[..]];

let amount_a_u64 = u64::try_from(amount_a).map_err(|_| Errors::TokenMaxExceeded)?;
    let amount_b_u64 = u64::try_from(amount_b).map_err(|_| Errors::TokenMaxExceeded)?;

    // Transfer tokens to user
    let cpi_accounts = Transfer {
        from: ctx.accounts.token_a_vault.to_account_info(),
        to: ctx.accounts.owner_token_a.to_account_info(),
        authority: pool.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, amount_a_u64)?;

    let cpi_accounts = Transfer {
        from: ctx.accounts.token_b_vault.to_account_info(),
        to: ctx.accounts.owner_token_b.to_account_info(),
        authority: pool.to_account_info(),
    };  
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, amount_b_u64)?;

    pool.liquidity = pool.liquidity.checked_sub(position.liquidity).ok_or(Errors::MathError)?;
    Ok(())
}
