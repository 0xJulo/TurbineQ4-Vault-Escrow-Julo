use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    litesvm::{types::TransactionResult, LiteSVM},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    vault::{STATE_SEED, VAULT_SEED},
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

fn get_rent(svm: &LiteSVM) -> u64 {
    svm.minimum_balance_for_rent_exemption(0)
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

#[test]
fn test_vault() {

    // Perform the set up

    let mut ctx = setup();
    let rent = get_rent(&ctx.svm);

    // ------------------------------------------------------------
    // STEP 01: INITIALIZE THE VAULT
    // ------------------------------------------------------------

    let initialize_instruction = Instruction::new_with_bytes(
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

    send_instruction(&mut ctx.svm, &ctx.user, initialize_instruction).unwrap();

    // Test the results

    // Check the vault balance is the rent-exempt minimum
    let vault_balance = ctx.svm.get_balance(&ctx.vault).unwrap();
    assert_eq!(vault_balance, rent, "Vault should hold rent-exempt minimum");

    // Check the vault_state account was created
    assert!(ctx.svm.get_account(&ctx.vault_state).is_some(), "vault_state should exist");

    // ------------------------------------------------------------
    // STEP 02: DEPOSIT SOL INTO THE VAULT
    // ------------------------------------------------------------
    
    const DEPOSIT_AMOUNT: u64 = 100_000_000;

    let deposit_instruction = Instruction::new_with_bytes(
        vault::ID,
        &vault::instruction::Deposit { amount: DEPOSIT_AMOUNT }.data(),
        vault::accounts::Deposit {
            user: ctx.user.pubkey(),
            vault_state: ctx.vault_state,
            vault: ctx.vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    send_instruction(&mut ctx.svm, &ctx.user, deposit_instruction).unwrap();

    // Test the results

    let vault_balance = ctx.svm.get_balance(&ctx.vault).unwrap();
    assert_eq!(vault_balance, rent + DEPOSIT_AMOUNT, "Vault balance should be the rent-exempt minimum plus the deposit amount");


    // ------------------------------------------------------------
    // STEP 03: WITHDRAW SOL FROM THE VAULT
    // ------------------------------------------------------------

    const WITHDRAW_AMOUNT: u64 = 50_000_000;

    let withdraw_instruction = Instruction::new_with_bytes(
        vault::ID,
        &vault::instruction::Withdraw { amount: WITHDRAW_AMOUNT }.data(),
        vault::accounts::Withdraw {
            user: ctx.user.pubkey(),
            vault_state: ctx.vault_state,
            vault: ctx.vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    send_instruction(&mut ctx.svm, &ctx.user, withdraw_instruction).unwrap();

    // Test the results

    let vault_balance = ctx.svm.get_balance(&ctx.vault).unwrap();
    assert_eq!(vault_balance, rent + DEPOSIT_AMOUNT - WITHDRAW_AMOUNT, "Vault balance should be the rent-exempt minimum plus the deposit amount minus the withdrawal amount");

    // ------------------------------------------------------------
    // STEP 04: Close the account
    // ------------------------------------------------------------

    let close_instruction = Instruction::new_with_bytes(
        vault::ID,
        &vault::instruction::Close {}.data(),
        vault::accounts::Close {
            user: ctx.user.pubkey(),
            vault_state: ctx.vault_state,
            vault: ctx.vault,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    send_instruction(&mut ctx.svm, &ctx.user, close_instruction).unwrap();

    // Test the results

    assert!(ctx.svm.get_account(&ctx.vault).is_none(), "Vault should be closed");
    assert!(ctx.svm.get_account(&ctx.vault_state).is_none(), "vault_state should be closed");
}