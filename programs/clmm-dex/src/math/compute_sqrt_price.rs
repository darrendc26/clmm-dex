use anchor_lang::prelude::*;
use crate::errors::Errors;

pub fn get_new_sqrt_price_from_input(sqrt_price_current: u128, liquidity: u128, amount_in: u128, a_to_b: bool) -> Result<u128> {
    if a_to_b {
        // Swapping token A for token B 
        // This affects the amount of token A, so we use the token A formula:
        // sqrt_price_new = (liquidity * sqrt_price_current) / (liquidity + amount_in * sqrt_price_current)
        // But we need to be careful about the scaling factor
        
        let numerator = liquidity.checked_mul(sqrt_price_current).ok_or(Errors::MathError)?;
        
        // For token0 input, we need to account for the price scaling
        // The amount affects the price denominator
        let denominator_addition = amount_in.checked_mul(sqrt_price_current).ok_or(Errors::MathError)?
            .checked_shr(64).ok_or(Errors::MathError)?; // Adjust for Q64.64 scaling
        
        let denominator = liquidity.checked_add(denominator_addition).ok_or(Errors::MathError)?;
        
        if denominator == 0 {
            return Err(Errors::DivisionByZero.into());
        }
        
        Ok(numerator.checked_div(denominator).ok_or(Errors::MathError)?)
    } else {
        // Swapping token B (token1) for token A (token0)
        // This affects the amount of token1, so we use the token1 formula:
        // sqrt_price_new = sqrt_price_current + (amount_in << 64) / liquidity
        
        let quotient = (amount_in << 64).checked_div(liquidity).ok_or(Errors::DivisionByZero)?;
        Ok(sqrt_price_current.checked_add(quotient).ok_or(Errors::MathError)?)
    }
}

pub fn get_new_sqrt_price_from_output(sqrt_price_current: u128, liquidity: u128, amount_out: u128, a_to_b: bool) -> Result<u128> {
    if a_to_b {
        // Outputting token B and inputting token A
        // This removes token B from the pool, so we use the token B formula:
        // sqrt_price_new = sqrt_price_current - (amount_out in Q64.64) / liquidity
        
        let quotient = (amount_out << 64).checked_div(liquidity).ok_or(Errors::DivisionByZero)?;
        Ok(sqrt_price_current.checked_sub(quotient).ok_or(Errors::MathError)?)
    } else {
        // Outputting token A and inputting token B
        // This removes token A from the pool, so we use the token A formula:
        // sqrt_price_new = (liquidity * sqrt_price_current) / (liquidity - amount_out * sqrt_price_current)
        
        let numerator = liquidity.checked_mul(sqrt_price_current).ok_or(Errors::MathError)?;
        
        // For token A output, we subtract from the denominator
        let denominator_subtraction = amount_out.checked_mul(sqrt_price_current).ok_or(Errors::MathError)?
            .checked_shr(64).ok_or(Errors::MathError)?; // Adjust for Q64.64 scaling
        
        if liquidity <= denominator_subtraction {
            return Err(Errors::InsufficientLiquidity.into());
        }
        
        let denominator = liquidity.checked_sub(denominator_subtraction).ok_or(Errors::MathError)?;
        
        if denominator == 0 {
            return Err(Errors::DivisionByZero.into());
        }
        
        Ok(numerator.checked_div(denominator).ok_or(Errors::MathError)?)
    }
}

