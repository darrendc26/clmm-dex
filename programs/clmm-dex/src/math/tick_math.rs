use anchor_lang::prelude::*;
use crate::state::Tick;
use crate::errors::Errors;

// Calculates the tick that corresponds to a given sqrt price
pub fn get_tick_from_sqrt_price(sqrt_price_x64: u128) -> Result<i32> {
    let sqrt_price = sqrt_price_x64 as f64 / (2u64.pow(64) as f64);
    let log_base = 1.0001_f64.ln();
    let tick = 2.0 * sqrt_price.ln() / log_base;

    Ok(tick.floor() as i32)
}

// Calculates the sqrt price from a tick
pub fn get_sqrt_price_from_tick(tick: i32) -> Result<u128> {
    let sqrt_price_x64 =  1.0001_f64.powf(tick as f64 / 2.0) * (2u64.pow(64) as f64);
    Ok(sqrt_price_x64 as u128)
}

// Calculates the next tick from a tick
pub fn get_next_tick(
    tick_current: i32,
    initialized_ticks: &Vec<i32>,
    a_to_b: bool
) -> Result<i32> {
    let next_tick = if a_to_b {
        initialized_ticks.iter().filter(|&&x| x < tick_current).max().copied()
    } else {
        initialized_ticks.iter().filter(|&&x| x > tick_current).min().copied()
    };

    match next_tick {
        Some(tick) => Ok(tick),
        None => Err(Errors::TickNotFound.into())
    }
}

// Updates the tick account in terms of fees
pub fn cross_tick(
    tick_account: &mut Tick,
    fee_growth_global_a: u128,
    fee_growth_global_b: u128,
    a_to_b: bool
) -> Result<()> {
    if a_to_b {
        // Update fee growth global
        tick_account.fee_growth_outside_a = fee_growth_global_a.checked_sub(tick_account.fee_growth_outside_a).ok_or(Errors::MathError)?;
    } else {
        // Update fee growth global
        tick_account.fee_growth_outside_b = fee_growth_global_b.checked_sub(tick_account.fee_growth_outside_b).ok_or(Errors::MathError)?;
    }
    Ok(())
}
