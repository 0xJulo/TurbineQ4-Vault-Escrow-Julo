use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    // Used in make and update: the expiry given is not in the future
    #[msg("Expiration must be in the future")]
    InvalidExpiration,

    // Used in take: someone tried to take after the expiry passed
    #[msg("Escrow has expired")]
    EscrowExpired,
}
