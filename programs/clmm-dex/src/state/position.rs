use anchor_lang::prelude::*;

#[account]
pub struct Position{
    pub owner: Pubkey, // Owner of the position
    pub tick_lower: i32, // Lower tick of the position
    pub tick_upper: i32, // Upper tick of the position
    pub liquidity: u128, // Liquidity in the position
    pub fee_growth_inside_a: u128, // Fee growth inside for token A
    pub fee_growth_inside_b: u128, // Fee growth inside for token B
    pub token_a_earned: u64, // Amount of token A earned by the position
    pub token_b_earned: u64, // Amount of token B earned by the position
    pub bump: u8, // Bump for PDA derivation
}