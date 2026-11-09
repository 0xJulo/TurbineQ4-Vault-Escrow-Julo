use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub seed: u64, // Used to identify a particular escrow
    pub maker: Pubkey, // The person who created the escrow
    pub mint_a: Pubkey, // The token the maker is offering
    pub mint_b: Pubkey, // The token the maker wants to receive
    pub receive: u64, // The amount of token B that the maker wants
    pub bump: u8, // Saved for PDA checking and signing 
}
