use anchor_lang::prelude::*;

// InitSpace is used to work out how much bytes the struct will take
// It gives you a constant, VaultState::INIT_SPACE
// Anchor gives you an 8-byte discriminator at the start of every account

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
