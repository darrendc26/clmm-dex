use anchor_lang::prelude::*;

// space = 8 + 4 + 16 + 16 + 16 + 1 + 1;
#[account]
pub struct Tick {
    pub tick_index: i32,
    pub liquidity_net: i128,
    pub fee_growth_outside_a: u128,
    pub fee_growth_outside_b: u128,
    pub initialized: bool,
    pub bump: u8,
}
