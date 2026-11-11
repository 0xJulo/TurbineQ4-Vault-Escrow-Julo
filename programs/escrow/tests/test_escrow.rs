use {
    anchor_lang::{
        prelude::{Clock, Pubkey},
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::{associated_token::get_associated_token_address, token::TokenAccount},
    escrow::ESCROW_SEED, 
    litesvm::{types::TransactionResult, LiteSVM}, 
    litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo}, 
    solana_keypair::Keypair, solana_message::{Message, VersionedMessage}, 
    solana_signer::Signer, 
    solana_transaction::versioned::VersionedTransaction
};

pub struct TestContext {
    pub svm: LiteSVM,
    maker: Keypair,
    taker: Keypair,
    mint_a: Pubkey,
    mint_b: Pubkey,
    maker_ata_a: Pubkey,
    taker_ata_b: Pubkey,
    escrow: Pubkey,
    vault: Pubkey,
    bump: u8,
}

// Setup: Build a fresh chain with your program loaded and a funded user
// This function will be used at the start of each test, providing it with a 
// fresh chain and a funded user.

const SEED: u64 = 42;

pub fn setup() -> TestContext {
    let mut svm = LiteSVM::new();

    let program_id = escrow::ID;

    // Get the program bytes
    // Add the program to the SVM

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/escrow.so"
    ));
    svm.add_program(program_id, bytes).unwrap();

    // Create the users

    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 2_000_000_000).unwrap();

    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 2_000_000_000).unwrap();

    // Create the token mints
    // This doesn't create tokens, it creates the mint account and the authority
    // which then has the ability to mint tokens.

    let mint_a = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    // Create the associated token accounts

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
        .owner(&maker.pubkey())
        .send()
        .unwrap();

    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    // Mint the tokens to the associated token accounts

    MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, 1_000_000_000)
        .send()
        .unwrap();

    MintTo::new(&mut svm, &maker, &mint_b, &taker_ata_b, 1_000_000_000)
        .send()
        .unwrap();

    // Set up Escrow

    let (escrow, bump) = Pubkey::find_program_address(
        &[ESCROW_SEED, maker.pubkey().as_ref(), &SEED.to_le_bytes()],
        &escrow::ID,
    );

    // Set up vault

    let vault = get_associated_token_address(&escrow, &mint_a);

    TestContext {
        svm,
        maker,
        taker,
        mint_a,
        mint_b,
        maker_ata_a,
        taker_ata_b,
        escrow,
        vault,
        bump,
    }
}

// Build a standard instruction send

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

// Make instruction: builds the `make` instruction from the test context.
// The maker locks `amount_a` of token A in the vault and asks for `receive` of token B.
// `expiration` is optional: None means the escrow never expires.

fn make_instruction(ctx: &TestContext, receive: u64, amount_a: u64, expiration: Option<i64>) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::Make {
            seed: SEED,
            receive,
            amount_a,
            expiration,
        }
        .data(),
        escrow::accounts::Make {
            maker: ctx.maker.pubkey(),
            mint_a: ctx.mint_a,
            mint_b: ctx.mint_b,
            maker_ata_a: ctx.maker_ata_a,
            escrow: ctx.escrow,
            vault: ctx.vault,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

// Take instruction: the taker pays token B to the maker and receives token A from the vault.
// maker_ata_b and taker_ata_a may not exist yet (init_if_needed in the program),
// so their addresses are worked out here rather than stored in the test context.

fn take_instruction(ctx: &TestContext) -> Instruction {
    let maker_ata_b = get_associated_token_address(&ctx.maker.pubkey(), &ctx.mint_b);
    let taker_ata_a = get_associated_token_address(&ctx.taker.pubkey(), &ctx.mint_a);

    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::Take {}.data(),
        escrow::accounts::Take {
            maker: ctx.maker.pubkey(),
            taker: ctx.taker.pubkey(),
            mint_a: ctx.mint_a,
            mint_b: ctx.mint_b,
            taker_ata_b: ctx.taker_ata_b,
            maker_ata_b,
            taker_ata_a,
            escrow: ctx.escrow,
            vault: ctx.vault,
            token_program: anchor_spl::token::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

// Refund instruction: the maker cancels, gets all token A back, and the vault and escrow are closed.

fn refund_instruction(ctx: &TestContext) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::Refund {}.data(),
        escrow::accounts::Refund {
            maker: ctx.maker.pubkey(),
            mint_a: ctx.mint_a,
            maker_ata_a: ctx.maker_ata_a,
            escrow: ctx.escrow,
            vault: ctx.vault,
            token_program: anchor_spl::token::ID,
        }
        .to_account_metas(None),
    )
}

// Update instruction: the maker changes the price (receive) and the optional expiration.

fn update_instruction(ctx: &TestContext, receive: u64, expiration: Option<i64>) -> Instruction {
    Instruction::new_with_bytes(
        escrow::ID,
        &escrow::instruction::Update { receive, expiration }.data(),
        escrow::accounts::Update {
            maker: ctx.maker.pubkey(),
            escrow: ctx.escrow,
        }
        .to_account_metas(None),
    )
}

// ------------------------------------------------------------
// TEST HELPERS
// ------------------------------------------------------------

// Every test uses the same amounts (6 decimals, so 1 token = 1_000_000)
const STARTING_BALANCE: u64 = 1_000_000_000; // 1,000 tokens, minted in setup()
const AMOUNT_A: u64 = 10_000_000; // 10 token A locked by the maker
const RECEIVE: u64 = 50_000_000; // 50 token B wanted in return

// Reads how many tokens a token account holds
fn token_balance(ctx: &TestContext, token_account: &Pubkey) -> u64 {
    let account = ctx.svm.get_account(token_account).unwrap();
    TokenAccount::try_deserialize(&mut account.data.as_ref()).unwrap().amount
}

// Reads the escrow account back into the Escrow struct
fn read_escrow(ctx: &TestContext) -> escrow::state::Escrow {
    let account = ctx.svm.get_account(&ctx.escrow).unwrap();
    escrow::state::Escrow::try_deserialize(&mut account.data.as_ref()).unwrap()
}

// The current time on the test chain
fn now(ctx: &TestContext) -> i64 {
    ctx.svm.get_sysvar::<Clock>().unix_timestamp
}

// Moves the test chain's clock forward, so expiry can be tested
fn warp_time(ctx: &mut TestContext, seconds: i64) {
    let mut clock = ctx.svm.get_sysvar::<Clock>();
    clock.unix_timestamp += seconds;
    ctx.svm.set_sysvar::<Clock>(&clock);
}

// ------------------------------------------------------------
// HAPPY PATH TESTS
// ------------------------------------------------------------

#[test]
fn test_make_and_refund() {
    let mut ctx = setup();

    // Make: the maker locks 10 token A
    let ix = make_instruction(&ctx, RECEIVE, AMOUNT_A, None);
    send_instruction(&mut ctx.svm, &ctx.maker, ix).unwrap();

    assert_eq!(token_balance(&ctx, &ctx.vault), AMOUNT_A, "Vault should hold the locked token A");
    assert_eq!(token_balance(&ctx, &ctx.maker_ata_a), STARTING_BALANCE - AMOUNT_A, "Maker should have sent token A");

    let escrow = read_escrow(&ctx);
    assert_eq!(escrow.seed, SEED);
    assert_eq!(escrow.maker, ctx.maker.pubkey());
    assert_eq!(escrow.mint_a, ctx.mint_a);
    assert_eq!(escrow.mint_b, ctx.mint_b);
    assert_eq!(escrow.receive, RECEIVE);
    assert_eq!(escrow.expiration, None);

    // Refund: the maker cancels and gets everything back
    let ix = refund_instruction(&ctx);
    send_instruction(&mut ctx.svm, &ctx.maker, ix).unwrap();

    assert_eq!(token_balance(&ctx, &ctx.maker_ata_a), STARTING_BALANCE, "Maker should have all token A back");
    assert!(ctx.svm.get_account(&ctx.vault).is_none(), "Vault should be closed");
    assert!(ctx.svm.get_account(&ctx.escrow).is_none(), "Escrow should be closed");
}

#[test]
fn test_make_and_take() {
    let mut ctx = setup();

    // Make: the maker offers 10 token A for 50 token B
    let ix = make_instruction(&ctx, RECEIVE, AMOUNT_A, None);
    send_instruction(&mut ctx.svm, &ctx.maker, ix).unwrap();

    // Take: the taker accepts
    let ix = take_instruction(&ctx);
    send_instruction(&mut ctx.svm, &ctx.taker, ix).unwrap();

    // These accounts were created by take (init_if_needed)
    let maker_ata_b = get_associated_token_address(&ctx.maker.pubkey(), &ctx.mint_b);
    let taker_ata_a = get_associated_token_address(&ctx.taker.pubkey(), &ctx.mint_a);

    assert_eq!(token_balance(&ctx, &maker_ata_b), RECEIVE, "Maker should receive token B");
    assert_eq!(token_balance(&ctx, &taker_ata_a), AMOUNT_A, "Taker should receive token A");
    assert_eq!(token_balance(&ctx, &ctx.taker_ata_b), STARTING_BALANCE - RECEIVE, "Taker should have paid token B");
    assert!(ctx.svm.get_account(&ctx.vault).is_none(), "Vault should be closed");
    assert!(ctx.svm.get_account(&ctx.escrow).is_none(), "Escrow should be closed");
}

#[test]
fn test_make_and_update() {
    let mut ctx = setup();

    let ix = make_instruction(&ctx, RECEIVE, AMOUNT_A, None);
    send_instruction(&mut ctx.svm, &ctx.maker, ix).unwrap();

    // Update: lower the price and add a deadline one day from now
    let new_receive = 40_000_000;
    let new_expiration = Some(now(&ctx) + 86_400);

    let ix = update_instruction(&ctx, new_receive, new_expiration);
    send_instruction(&mut ctx.svm, &ctx.maker, ix).unwrap();

    let escrow = read_escrow(&ctx);
    assert_eq!(escrow.receive, new_receive, "Price should be updated");
    assert_eq!(escrow.expiration, new_expiration, "Expiration should be updated");
}

// ------------------------------------------------------------
// FAILURE TESTS
// ------------------------------------------------------------

#[test]
fn test_take_after_expiry_fails() {
    let mut ctx = setup();

    // Make an escrow that expires in 100 seconds
    let expiration = Some(now(&ctx) + 100);
    let ix = make_instruction(&ctx, RECEIVE, AMOUNT_A, expiration);
    send_instruction(&mut ctx.svm, &ctx.maker, ix).unwrap();

    // Move the clock past the deadline
    warp_time(&mut ctx, 200);

    let ix = take_instruction(&ctx);
    let result = send_instruction(&mut ctx.svm, &ctx.taker, ix);

    assert!(result.is_err(), "Take should fail after expiry");
    let logs = result.unwrap_err().meta.logs.join("\n");
    assert!(logs.contains("EscrowExpired"), "Should fail with EscrowExpired");

    // The maker's tokens are still safe in the vault
    assert_eq!(token_balance(&ctx, &ctx.vault), AMOUNT_A, "Vault should still hold token A");
}

#[test]
fn test_make_with_past_expiry_fails() {
    let mut ctx = setup();

    // An expiry in the past makes no sense
    let expiration = Some(now(&ctx) - 1);
    let ix = make_instruction(&ctx, RECEIVE, AMOUNT_A, expiration);
    let result = send_instruction(&mut ctx.svm, &ctx.maker, ix);

    assert!(result.is_err(), "Make should fail with a past expiry");
    let logs = result.unwrap_err().meta.logs.join("\n");
    assert!(logs.contains("InvalidExpiration"), "Should fail with InvalidExpiration");
}
