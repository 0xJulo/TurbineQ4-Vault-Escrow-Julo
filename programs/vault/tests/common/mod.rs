use {
    anchor_lang::{
        solana_program::instruction::Instruction,
    },
    litesvm::LiteSVM,
    litesvm::types::TransactionResult,
    solana_keypair::Keypair,    
    solana_message::{Message, VersionedMessage},
    solana_transaction::versioned::VersionedTransaction,
    solana_signer::Signer,
};

// Setup: Build a fresh chain with your program loaded and a funded user
// This function will be used at the start of each test, providing it with a 
// fresh chain and a funded user.

pub fn setup() -> (LiteSVM, Keypair) {
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