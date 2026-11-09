// Tests for the initialize instruction.
// Shared setup (fresh chain, funded user, PDAs) and sending live in tests/common.
// Each test: build the instruction → send it → check the results.

mod common;

use {
    anchor_lang::{
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    solana_signer::Signer,
    common::{setup, send_instruction},
};

#[test]
fn test_initialize() {

    // setup() gives a fresh chain, a funded user and both PDAs (from common).
    // ctx is mutable because sending transactions changes ctx.svm.

    let mut ctx = setup();

    // Create the instruction to initialize the vault.
    // Every Solana instruction, in any program, has the same three things:
    // 1. The program ID
    // 2. The data - which instruction plus any arguments
    // 3. The accounts that are being passed to the instruction

    let instruction = Instruction::new_with_bytes(
        vault::ID,
        &vault::instruction::Initialize {}.data(),
        vault::accounts::Init {
            user: ctx.user.pubkey(),
            vault_state: ctx.vault_state,
            vault: ctx.vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    // Sign and send the instruction (send_instruction is in common)

    send_instruction(&mut ctx.svm, &ctx.user, instruction).unwrap();

    // Test the results

    // Check the vault balance is the rent-exempt minimum
    let rent = ctx.svm.minimum_balance_for_rent_exemption(0);
    let vault_balance = ctx.svm.get_balance(&ctx.vault).unwrap();
    assert_eq!(vault_balance, rent, "Vault should hold rent-exempt minimum");

    // Check the vault_state account was created
    assert!(ctx.svm.get_account(&ctx.vault_state).is_some(), "vault_state should exist");
}