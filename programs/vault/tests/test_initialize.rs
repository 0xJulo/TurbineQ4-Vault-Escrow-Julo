// When testing, you want to perform the set up first
// Create the fake chain for LiteSVM, load the program and make a user with some SOL
// Could this be done once and then used across different tests?
// Find the PDAs from the seeds that are used in the program
// Say which instructions to call and which accounts to pass

mod common;

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    vault::{VAULT_SEED, STATE_SEED},
    solana_signer::Signer,
    common::{setup, send_instruction},
};

#[test]
fn test_initialize() {

    // Only svm is mutable here because the transaction changes the chain.

    let (mut svm, user) = setup();

    // Find the PDA for the vault state and the vault.

    let (vault_state, state_bump) = Pubkey::find_program_address(
        &[STATE_SEED, user.pubkey().as_ref()],
        &vault::ID,
    );

    let (vault, vault_bump) = Pubkey::find_program_address(
        &[VAULT_SEED, user.pubkey().as_ref()],
        &vault::ID
    );

    // Create the instruction to initialize the vault.
    // Every Solana instruction, in any program, has the same three things:
    // 1. The program ID
    // 2. The data - which instruction plus any arguments
    // 3. The accounts that are being passed to the instruction

    let instruction = Instruction::new_with_bytes(
        vault::ID,
        &vault::instruction::Initialize {}.data(),
        vault::accounts::Init {
            user: user.pubkey(),
            vault_state,
            vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    // Send instruction function from above

    send_instruction(&mut svm, &user, instruction).unwrap();

    // Test the results

    // Check the vault balance is the rent-exempt minimum
    let rent = svm.minimum_balance_for_rent_exemption(0);
    let vault_balance = svm.get_balance(&vault).unwrap();
    assert_eq!(vault_balance, rent, "Vault should hold rent-exempt minimum");

    // Check the vault_state account was created
    assert!(svm.get_account(&vault_state).is_some(), "vault_state should exist");
}