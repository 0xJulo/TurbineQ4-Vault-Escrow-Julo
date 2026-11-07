use anchor_lang::prelude::*;

// This will act as the seed for the PDA
// State holds the bumps, and this has the seeds,
// And the combination comes later in the code. 

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

#[constant]
pub const STATE_SEED: &[u8] = b"state";
