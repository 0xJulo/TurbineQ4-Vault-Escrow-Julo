use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    Mint, 
    TokenAccount, 
    TokenInterface, 
    transfer_checked, 
    TransferChecked,
    close_account,
    CloseAccount,
};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::Escrow;
use crate::constants::ESCROW_SEED;
use crate::error::EscrowError;

#[derive(Accounts)]
pub struct Take<'info>{

    // Who is involved

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mut)]
    pub taker: Signer<'info>,

    // The mints that are involved
    // Boxed to move their data to the heap: Take has so many accounts
    // that they overflow Solana's 4 KB stack limit otherwise.

    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    // How the mints affec the user

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // The accounts

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // The underlying programs

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {

    // Step 1: Tokens go straight to the maker

    pub fn deposit_tokens(&mut self) -> Result<()> {

        // Check for expiration

        let now = Clock::get()?.unix_timestamp;

        if let Some(exp) = self.escrow.expiration {
            require!(now <= exp, EscrowError::EscrowExpired);
        }
        
        // Create the CPI

        let cpi_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(), 
            to: self.maker_ata_b.to_account_info(), 
            mint: self.mint_b.to_account_info(), 
            authority: self.taker.to_account_info(), 
        };

        let cpi_program = self.token_program.key();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, self.escrow.receive, self.mint_b.decimals)?;
        Ok(())
    }

    // Step 2: Withdraw and close the escrow

    pub fn withdraw_and_close(&mut self) -> Result<()> {

        // Get the PDA details

        let maker_key = self.maker.key();
        let seed_bytes = self.escrow.seed.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[
            ESCROW_SEED, 
            maker_key.as_ref(), 
            seed_bytes.as_ref(),
            &[self.escrow.bump]
        ]];

        // Create the CPI

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(), 
            to: self.taker_ata_a.to_account_info(), 
            mint: self.mint_a.to_account_info(), 
            authority: self.escrow.to_account_info(), 
        };

        let cpi_program = self.token_program.key();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        // Close the vault

        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let close_ctx = CpiContext::new_with_signer(cpi_program, close_accounts, signer_seeds);

        close_account(close_ctx)?;

        Ok(())
    }
}