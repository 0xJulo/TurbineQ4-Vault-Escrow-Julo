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
}
