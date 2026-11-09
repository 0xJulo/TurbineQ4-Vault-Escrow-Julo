use {
    anchor_lang::prelude::*,
    anchor_lang::system_program::{transfer, Transfer},
    crate::{constants::*, state::VaultState},
};

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        close = user,
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

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {

        // Create variables for the vault balance

        let vault_balance = self.vault.lamports();

        // Signer seeds: the same seeds + bump that created the vault PDA.
        // The program uses these to sign for the vault (a PDA has no private key).
        // Save the key first so the seeds don't point at a temporary value.
        
        let user_key = self.user.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            VAULT_SEED, 
            user_key.as_ref(), 
            &[self.vault_state.vault_bump]
        ]];

        // PDA-signed CPI: accounts → program → context (with signer seeds) → call
        // The vault is sending SOL, so the program signs for it with new_with_signer

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let cpi_program = self.system_program.key();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, vault_balance)?;

        Ok(())
    }
}