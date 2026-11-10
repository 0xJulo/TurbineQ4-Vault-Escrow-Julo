use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint, 
    TokenAccount, 
    TokenInterface, 
    transfer_checked, 
    TransferChecked,
};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::Escrow;
use crate::constants::ESCROW_SEED;
use crate::error::EscrowError;

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

    // This function is for creating a new escrow.

    pub fn init_escrow(
        &mut self, 
        seed: u64, 
        receive: u64, bumps: &MakeBumps,
        expiration: Option<i64>,
    ) -> Result<()> {

        // Check for expiration
    
        let now = Clock::get()?.unix_timestamp;

        if let Some(expiration) = expiration {
            require!(expiration > now, EscrowError::InvalidExpiration);
        }

        // Set the escrow struct
        self.escrow.set_inner(Escrow { 
            seed, 
            maker: self.maker.key(), 
            mint_a: self.mint_a.key(), 
            mint_b: self.mint_b.key(), 
            receive, 
            bump: bumps.escrow, 
            expiration,
        });

        Ok(())
    }

    // The tokens will move from the makers ATA account into the escrow vault.

    pub fn deposit_tokens(&mut self, amount: u64) -> Result<()> {

        // Same idea as the vault, collect the accounts
        // state which program will be doing the transfer,
        // build the context of the pair of them,
        // then send the transfer. 

        let cpi_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_program = self.token_program.key();

        let cpi_ctx = CpiContext::new(cpi_program.key(), cpi_accounts);

        // When you do transfer checked, you need to provide the decimals
        // as it checks this against the mint.

        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;

        Ok(())
    }
}

