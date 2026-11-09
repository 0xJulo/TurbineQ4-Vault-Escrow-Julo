use anchor_lang::prelude::*;

// This constant will be used as the seed for all escrow PDAs
// The maker's key and the seed number from the struct in state.rs
// will make each escrow unique. 

#[constant]
pub const ESCROW_SEED: &[u8] = b"escrow";

