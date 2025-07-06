use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("The token max exceeded")]    
    TokenMaxExceeded,
    #[msg("The multiplication overflow")]    
    MultiplicationOverflow,
    #[msg("Invalid tick range provided.")]
    InvalidTickRange,
    #[msg("Invalid tick spacing.")]
    InvalidTickSpacing,
    #[msg("Invalid fee growth.")]
    InvalidFeeGrowth,
    #[msg("Invalid pool.")]
    InvalidPool,
    #[msg("Invalid position.")]
    InvalidPosition,
    #[msg("Math error occurred.")]
    MathError,
    #[msg("Tick not found.")]
    TickNotFound,
    #[msg("Too many iterations.")]
    TooManyIterations,
    #[msg("Invalid amount.")]
    InvalidAmount,
    #[msg("Division by zero.")]
    DivisionByZero,
    #[msg("Insufficient liquidity.")]
    InsufficientLiquidity,
    #[msg("Invalid fee rate.")]
    InvalidFeeRate,
    #[msg("Invalid tick.")]
    InvalidTick,
}