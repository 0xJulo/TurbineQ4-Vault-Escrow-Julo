use anchor_lang::prelude::*;
use crate::state::Escrow;
use crate::constants::ESCROW_SEED;
use crate::error::EscrowError;

#[derive(Accounts)]
pub struct Update<'info> {
    pub maker: Signer<'info>,

    #[account(
        mut,
        has_one = maker,
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
}

impl<'info> Update<'info> {
    pub fn update_escrow(&mut self, receive: u64, expiration: Option<i64>) -> Result<()> {

        // Check for expiration
    
        let now = Clock::get()?.unix_timestamp;

        if let Some(expiration) = expiration {
            require!(expiration > now, EscrowError::InvalidExpiration);
        }

        // Set the escrow struct
        self.escrow.receive = receive;
        self.escrow.expiration = expiration;

        Ok(())
    }
}