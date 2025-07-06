use anchor_lang::prelude::*;
    // space = 8 + 32 + 32 + 32 + 32 + 32 + 16 + 4 + 16 + 2 + 1 + 16 + 16 + 16 + 16 + 1;
    #[account]
    pub struct Pool{
    pub pool_authority: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub sqrt_price: u128,
    pub tick_current: i32,
    pub liquidity: u128,
    pub tick_spacing: u16,
    pub initialized_ticks: Vec<i32>,
    pub fee: u8,
    pub fee_growth_global_a: u128,
    pub fee_growth_global_b: u128,
    pub protocol_fee_a: u128,
    pub protocol_fee_b: u128,
    pub bump: u8,
}

