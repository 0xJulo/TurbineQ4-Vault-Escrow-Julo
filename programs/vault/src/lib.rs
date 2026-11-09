pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8ZDScW5CB5o9NYKzqoM1w4ukRP34uT4Gh4H7NSvsMGRM");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Init>) -> Result<()> {

        // #[derive(Accounts)] macro create a hidden struct that holds the bumps

        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}
