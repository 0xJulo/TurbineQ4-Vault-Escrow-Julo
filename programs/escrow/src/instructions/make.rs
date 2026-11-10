use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::Escrow;
use crate::constants::ESCROW_SEED;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    // This constraint ensures that the mint is
    //using the token program interface.

    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // Prove that the maker has tokens for this. 
    // associated_token::mint = mint_a = that it holds token A
    // associated_token::authority = maker = that the maker owns it
    // associated_token::token_program = token_program = it belongs to the same token program as the mint

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = Escrow::DISCRIMINATOR.len() + Escrow::INIT_SPACE,
        seeds = [ESCROW_SEED, maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // The vault is a new token account which will hold Token A from the maker
    // until the escrow is taken or refunded. 
    // It is owned by the escrow PDA, the authority is set to the escrow
    // so only the program can release the tokens by signing as escrow. 
    // The maker pays rent to create it. 

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    pub fn init_escrow(
        &mut self, 
        seed: u64, 
        receive: u64, bumps: &MakeBumps
    ) -> Result<()> {
        self.escrow.set_inner(Escrow { 
            seed, 
            maker: self.maker.key(), 
            mint_a: self.mint_a.key(), 
            mint_b: self.mint_b.key(), 
            receive, 
            bump: bumps.escrow, });
        Ok(())
    }
}