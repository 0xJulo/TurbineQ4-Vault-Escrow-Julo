use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    }, 
    anchor_spl::associated_token::get_associated_token_address, 
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
