use anchor_lang::prelude::*;

use crate::state::VaultState; // Import the VaultState from the state.rs file
use crate::constants::*; // Bring all constants into the file

// use crate::{constants::*, state::VaultState} - both of themm at the same time

#[derive(Accounts)]
pub struct Init<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    // This account needs to be created, hence the init
    // You define who is going to create it (payer) and how much
    // space it will take up from the state struct (space).
    // Seeds are derived from the constants and the user key

    #[account(
        init,
        payer = user,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
        seeds = [STATE_SEED, user.key().as_ref()],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

// impl here is essentually saying "here are the functions for..."
// Then adding the struct that you have defined
// 'info needs to be written in both of these as a lifetime
// Then you can write in the instruction functions

impl<'info> Init<'info> {

    // Initialize will need to take in the struct you have created
    // and the bumps that Anchor needs

    pub fn initialize(&mut self, bumps: &InitBumps) -> Result<()> {
        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        Ok(())
    }
}