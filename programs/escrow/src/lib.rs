pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DKEwRWy6jJQq4YgLBsxisLQgb5aYJfeVW4dTMJojnped");

#[program]
pub mod escrow {
    use super::*;

    // Initialising the escrow and deposit are the same action.
    // They are two functions called off the back of the same instruction.

    pub fn make(
        ctx: Context<Make>, 
        seed: u64, 
        receive: u64, 
        amount_a: u64,
        expiration: Option<i64>,
    ) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps, expiration)?;
        ctx.accounts.deposit_tokens(amount_a)?;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit_tokens()?;
        ctx.accounts.withdraw_and_close()?;
        Ok(())
    }
}
