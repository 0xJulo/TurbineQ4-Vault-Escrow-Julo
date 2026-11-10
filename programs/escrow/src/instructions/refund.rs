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
use crate::state::Escrow;
use crate::constants::ESCROW_SEED;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program,
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
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

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {

        // This is a PDA, you need to provide the seed and bump
        // Same as withdraw in the vault program.

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
            to: self.maker_ata_a.to_account_info(), 
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