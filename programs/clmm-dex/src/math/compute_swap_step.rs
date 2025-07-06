use crate::errors::Errors;
use crate::math::compute_sqrt_price::*;

// Calculates the swap step, how much amount_in, out, and fees we need to pay
pub fn compute_swap_step(
    sqrt_price_current: u128,
    sqrt_price_target: u128,
    liquidity: u128,
    amount_remaining: u128,
    fee_rate: u128, // e.g., 3000 = 0.3% (basis points)
    exact_in: bool, // true for exact input, false for exact output
) -> Result<(u128, u128, u128, u128), Errors> {
    if sqrt_price_current == sqrt_price_target {
        return Ok((0, 0, sqrt_price_current, 0));
    }

    let zero_for_one = sqrt_price_target < sqrt_price_current;
    let fee_denominator = 1_000_000u128; // 100% in basis points

    let mut amount_in: u128;
    let amount_out: u128;
    let sqrt_price_next: u128;

    if exact_in {
        // We know the input amount (after fees), calculate output
        let amount_remaining_less_fee = amount_remaining
            .checked_mul(fee_denominator.checked_sub(fee_rate).ok_or(Errors::InvalidFeeRate)?)
            .ok_or(Errors::MathError)?
            .checked_div(fee_denominator)
            .ok_or(Errors::MathError)?;

        if zero_for_one {
            // Swapping token A for token B 
            // Calculate max amount A needed to reach target price
            let amount_in_max = get_amount_a_delta(
                sqrt_price_target,
                sqrt_price_current,
                liquidity,
                true, // round up
            )?;

            if amount_remaining_less_fee >= amount_in_max {
                // We can reach the target price
                amount_in = amount_in_max;
                sqrt_price_next = sqrt_price_target;
            } else {
                // We can't reach target, calculate new price
                amount_in = amount_remaining_less_fee;
                sqrt_price_next = get_new_sqrt_price_from_input(
                    sqrt_price_current,
                    liquidity,
                    amount_in,
                    true,
                ).map_err(|_| Errors::MathError)?;
            }

            // Calculate amount B out
            amount_out = get_amount_b_delta(
                sqrt_price_next,
                sqrt_price_current,
                liquidity,
                false, // round down
            )?;

        } else {
            // Swapping token B for token A
            // Calculate max amount B needed to reach target price
            let amount_in_max = get_amount_b_delta(
                sqrt_price_current,
                sqrt_price_target,
                liquidity,
                true, // round up
            )?;

            if amount_remaining_less_fee >= amount_in_max {
                // We can reach the target price
                amount_in = amount_in_max;
                sqrt_price_next = sqrt_price_target;
            } else {
                // We can't reach target, calculate new price
                amount_in = amount_remaining_less_fee;
                sqrt_price_next = get_new_sqrt_price_from_input(
                    sqrt_price_current,
                    liquidity,
                    amount_in,
                    false,
                ).map_err(|_| Errors::MathError)?;
            }

            // Calculate amount A out
            amount_out = get_amount_a_delta(
                sqrt_price_current,
                sqrt_price_next,
                liquidity,
                false, // round down
            )?;
        }

        // Add back fee to get total amount_in including fee
        amount_in = amount_in
            .checked_mul(fee_denominator)
            .ok_or(Errors::MathError)?
            .checked_div(fee_denominator.checked_sub(fee_rate).ok_or(Errors::InvalidFeeRate)?)
            .ok_or(Errors::MathError)?;

    } else {
        // Exact output - we know desired output amount
        if zero_for_one {
            // Swapping token A for token B, want exact amount B out
            let amount_out_max = get_amount_b_delta(
                sqrt_price_target,
                sqrt_price_current,
                liquidity,
                false, // round down
            )?;

            if amount_remaining >= amount_out_max {
                // We can reach the target price
                amount_out = amount_out_max;
                sqrt_price_next = sqrt_price_target;
            } else {
                // We can't reach target
                amount_out = amount_remaining;
                sqrt_price_next = get_new_sqrt_price_from_output(
                    sqrt_price_current,
                    liquidity,
                    amount_out,
                    true,
                ).map_err(|_| Errors::MathError)?;
            }

            // Calculate required amount0 in
            amount_in = get_amount_a_delta(
                sqrt_price_next,
                sqrt_price_current,
                liquidity,
                true, // round up
            )?;

        } else {
            // Swapping token B for token A, want exact amount A out
            let amount_out_max = get_amount_a_delta(
                sqrt_price_current,
                sqrt_price_target,
                liquidity,
                false, // round down
            )?;

            if amount_remaining >= amount_out_max {
                // We can reach the target price
                amount_out = amount_out_max;
                sqrt_price_next = sqrt_price_target;
            } else {
                // We can't reach target
                amount_out = amount_remaining;
                sqrt_price_next = get_new_sqrt_price_from_output(
                    sqrt_price_current,
                    liquidity,
                    amount_out,
                    false,
                ).map_err(|_| Errors::MathError)?;
            }

            // Calculate required amount B in
            amount_in = get_amount_b_delta(
                sqrt_price_current,
                sqrt_price_next,
                liquidity,
                true, // round up
            )?;
        }

        // Add fee to input amount
        amount_in = amount_in
            .checked_mul(fee_denominator)
            .ok_or(Errors::MathError)?
            .checked_div(fee_denominator.checked_sub(fee_rate).ok_or(Errors::InvalidFeeRate)?)
            .ok_or(Errors::MathError)?;
    }

    // Calculate fee amount
    let fee_amount = amount_in
        .checked_mul(fee_rate)
        .ok_or(Errors::MathError)?
        .checked_div(fee_denominator)
        .ok_or(Errors::MathError)?;

    Ok((amount_in, amount_out, sqrt_price_next, fee_amount))
}

// Helper functions for amount calculations
fn get_amount_a_delta(
    sqrt_ratio_a: u128,
    sqrt_ratio_b: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<u128, Errors> {
    if sqrt_ratio_a > sqrt_ratio_b {
        return get_amount_a_delta(sqrt_ratio_b, sqrt_ratio_a, liquidity, round_up);
    }

    let numerator1 = liquidity << 64;
    let numerator2 = sqrt_ratio_b.checked_sub(sqrt_ratio_a).ok_or(Errors::MathError)?;
    let denominator = sqrt_ratio_b.checked_mul(sqrt_ratio_a).ok_or(Errors::MathError)?;

    if denominator == 0 {
        return Err(Errors::DivisionByZero.into());
    }

    let result = numerator1
        .checked_mul(numerator2)
        .ok_or(Errors::MathError)?
        .checked_div(denominator)
        .ok_or(Errors::MathError)?;

    // For rounding, we'd need more sophisticated logic in a real implementation
    Ok(result)
}

fn get_amount_b_delta(
    sqrt_ratio_a: u128,
    sqrt_ratio_b: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<u128, Errors> {
    if sqrt_ratio_a > sqrt_ratio_b {
        return get_amount_b_delta(sqrt_ratio_b, sqrt_ratio_a, liquidity, round_up);
    }

    let result = liquidity
        .checked_mul(sqrt_ratio_b.checked_sub(sqrt_ratio_a).ok_or(Errors::MathError)?)
        .ok_or(Errors::MathError)?
        .checked_shr(64)
        .ok_or(Errors::MathError)?;

    Ok(result)
}