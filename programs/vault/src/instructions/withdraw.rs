use {
    anchor_lang::prelude::*,
    anchor_lang::system_program::{transfer, Transfer},
    crate::{constants::*, state::VaultState, error::VaultError},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [STATE_SEED, user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount:u64) -> Result<()> {

        // Create variables for the vault balance and rent

        let vault_balance = self.vault.lamports();
        let rent = Rent::get()?.minimum_balance(0);

        // Signer seeds: the same seeds + bump that created the vault PDA.
        // The program uses these to sign for the vault (a PDA has no private key).
        // Save the key first so the seeds don't point at a temporary value.
        
        let user_key = self.user.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            VAULT_SEED, 
            user_key.as_ref(), 
            &[self.vault_state.vault_bump]
        ]];

        // Check the amount is valid and the vault has enough balance

        require!(amount > 0, VaultError::InvalidAmount);
        require!(amount <= vault_balance - rent, VaultError::InsufficientBalance);

        // PDA-signed CPI: accounts → program → context (with signer seeds) → call
        // The vault is sending SOL, so the program signs for it with new_with_signer

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let cpi_program = self.system_program.key();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}