use anchor_lang::prelude::*;

use crate::math::tick_math::{get_sqrt_price_from_tick};

pub fn compute_amount(
    liquidity: u128,
    tick_upper: i32,
    tick_lower: i32,
    tick_current: i32,
) -> Result<(u128, u128)> {
    let sqrt_lower_price = get_sqrt_price_from_tick(tick_lower)?;
    let sqrt_upper_price = get_sqrt_price_from_tick(tick_upper)?;
    let sqrt_current_price = get_sqrt_price_from_tick(tick_current)?;

    let liquidity = liquidity as u128;

    // Case 1: Current price below lower → all Token A 
    // a = l * [( sqrt_upper - sqrt_lower)/sqrt_upper * sqrt_lower]
    if sqrt_current_price <= sqrt_lower_price {
        let price_diff = sqrt_upper_price
            .checked_sub(sqrt_lower_price)
            .ok_or(ErrorCode::InvalidTickRange)?;

        let denominator = sqrt_upper_price
            .checked_mul(sqrt_lower_price)
            .ok_or(ErrorCode::MultiplicationOverflow)?;

        let numerator = liquidity
            .checked_mul(price_diff)
            .ok_or(ErrorCode::MultiplicationOverflow)?;

        let amount_a = numerator
            .checked_div(denominator)
            .ok_or(ErrorCode::DivisionByZero)?;

        return Ok((amount_a, 0));
    }

    // Case 2: Current price above upper → all Token B
    // b = l * [sqrt_upper - sqrt_lower]
    if sqrt_current_price >= sqrt_upper_price {
        let price_gap = sqrt_upper_price
            .checked_sub(sqrt_lower_price)
            .ok_or(ErrorCode::InvalidTickRange)?;

        let amount_b = liquidity
            .checked_mul(price_gap)
            .ok_or(ErrorCode::MultiplicationOverflow)?;

        return Ok((0, amount_b));
    }

    // Case 3: Price is inside range → split A and B
    // a = l * [( sqrt_upper - sqrt_lower)/sqrt_upper * sqrt_lower]
    // b = l * [sqrt_upper - sqrt_lower]
    let amount_a_numerator = liquidity
        .checked_mul(sqrt_upper_price - sqrt_current_price)
        .ok_or(ErrorCode::MultiplicationOverflow)?;

    let amount_a_denominator = sqrt_upper_price
        .checked_mul(sqrt_current_price)
        .ok_or(ErrorCode::MultiplicationOverflow)?;

    let amount_a = amount_a_numerator
        .checked_div(amount_a_denominator)
        .ok_or(ErrorCode::DivisionByZero)?;

    let amount_b = liquidity
        .checked_mul(sqrt_current_price - sqrt_lower_price)
        .ok_or(ErrorCode::MultiplicationOverflow)?;

    Ok((amount_a, amount_b))
}


#[error_code]
pub enum ErrorCode {
    #[msg("Multiplication overflow occurred.")]
    MultiplicationOverflow,
    #[msg("Division by zero occurred.")]
    DivisionByZero,
    #[msg("Invalid tick range provided.")]
    InvalidTickRange,
}   