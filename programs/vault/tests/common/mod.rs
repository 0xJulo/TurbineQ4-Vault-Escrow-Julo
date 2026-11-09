use {
    anchor_lang::{
        solana_program::instruction::Instruction,
        prelude::Pubkey,
    },
    litesvm::LiteSVM,
    litesvm::types::TransactionResult,
    solana_keypair::Keypair,    
    solana_message::{Message, VersionedMessage},
    solana_transaction::versioned::VersionedTransaction,
    solana_signer::Signer,
    vault::{VAULT_SEED, STATE_SEED},
};

// TestContext: A struct that holds the initial setup for a test
// so that you can focus on the instructions within the test itself.
// Handle all this once and it can be reused. 

pub struct TestContext {
    pub svm: LiteSVM,
    pub user: Keypair,
    pub vault_state: Pubkey,
    pub vault_bump: u8,
    pub vault: Pubkey,
    pub state_bump: u8,
}

// Setup: Build a fresh chain with your program loaded and a funded user
// This function will be used at the start of each test, providing it with a 
// fresh chain and a funded user.

pub fn setup() -> TestContext {
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

    let (vault_state, state_bump) = Pubkey::find_program_address(
        &[STATE_SEED, user.pubkey().as_ref()],
        &vault::ID,
    );

    let (vault, vault_bump) = Pubkey::find_program_address(
        &[VAULT_SEED, user.pubkey().as_ref()],
        &vault::ID
    );

    TestContext {
        svm,
        user,
        vault_state,
        vault_bump,
        vault,
        state_bump,
    }
}

// Send instruction: Wraps an instruction in a transaction, signs it as the user, and sends it.
// Reused by every test. Only the instruction changes.
// Returns Ok if the transaction succeeded, Err if it failed.

pub fn send_instruction(
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