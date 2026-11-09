// When testing, you want to perform the set up first
// Create the fake chain for LiteSVM, load the program and make a user with some SOL
// Could this be done once and then used across different tests?
// Find the PDAs from the seeds that are used in the program
// Say which instructions to call and which accounts to pass

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    litesvm::types::TransactionResult,
    vault::{VAULT_SEED, STATE_SEED},
    solana_keypair::Keypair,
    solana_signer::Signer,
    solana_message::{Message, VersionedMessage},
    solana_transaction::versioned::VersionedTransaction,
};

// Setup: Build a fresh chain with your program loaded and a funded user
// This function will be used at the start of each test, providing it with a 
// fresh chain and a funded user.

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let program_id = vault::ID;

    // Bytes is the compiled program, anchor build creates this.
    // The raw program is bytes. 

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/vault.so"
    ));
    svm.add_program(program_id, bytes).unwrap();

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 2_000_000_000).unwrap();

    (svm, user)
}

// Send instruction: Wraps an instruction in a transaction, signs it as the user, and sends it.
// Reused by every test. Only the instruction changes.
// Returns Ok if the transaction succeeded, Err if it failed.

fn send_instruction(
    svm: &mut LiteSVM, 
    user: &Keypair, 
    instruction: Instruction
) -> TransactionResult {
    let blockhash = svm.latest_blockhash();

    // The message holds the content of the transaction before it is signed

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(&user.pubkey()),
        &blockhash,
    );

    let transaction = VersionedTransaction::try_new(
        VersionedMessage::Legacy(message),
        &[user],
    ).unwrap();

    svm.send_transaction(transaction)
}

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