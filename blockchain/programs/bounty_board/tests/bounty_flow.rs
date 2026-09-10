use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::system_program,
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    bounty_board::{
        accounts, constants::GLOBAL_SEED, instruction, state::BountyState, state::BountyStatus,
        state::GlobalState, state::UserProfile, ID as PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_pubkey::Pubkey as SolPubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/bounty_board.so"
    ));
    svm.add_program(to_sol_pubkey(PROGRAM_ID), bytes).unwrap();
    svm
}

fn airdrop(svm: &mut LiteSVM, user: &Keypair, lamports: u64) {
    svm.airdrop(&user.pubkey(), lamports).unwrap();
}

fn send_ix(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction, signers: &[&Keypair]) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let mut all_signers: Vec<&Keypair> = vec![payer];
    for signer in signers {
        if signer.pubkey() != payer.pubkey() {
            all_signers.push(signer);
        }
    }
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all_signers).unwrap();
    svm.send_transaction(tx).expect("transaction should succeed");
}

fn send_ix_expect_err(svm: &mut LiteSVM, payer: &Keypair, ix: Instruction, signers: &[&Keypair]) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let mut all_signers: Vec<&Keypair> = vec![payer];
    for signer in signers {
        if signer.pubkey() != payer.pubkey() {
            all_signers.push(signer);
        }
    }
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &all_signers).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "transaction should fail"
    );
}

fn to_pubkey(pk: SolPubkey) -> Pubkey {
    Pubkey::new_from_array(pk.to_bytes())
}

fn to_sol_pubkey(pk: Pubkey) -> SolPubkey {
    SolPubkey::new_from_array(pk.to_bytes())
}

fn global_pda() -> Pubkey {
    Pubkey::find_program_address(&[GLOBAL_SEED], &PROGRAM_ID).0
}

fn user_pda(authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"user", authority.as_ref()], &PROGRAM_ID).0
}

fn bounty_pda(poster: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"bounty", poster.as_ref(), &id.to_le_bytes()], &PROGRAM_ID).0
}

fn initialize_global(svm: &mut LiteSVM, payer: &Keypair) {
    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::Initialize {
            payer: to_pubkey(payer.pubkey()),
            global_state: global_pda(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::Initialize {}.data(),
    };
    send_ix(svm, payer, ix, &[]);
}

fn initialize_user(svm: &mut LiteSVM, payer: &Keypair) {
    let authority = to_pubkey(payer.pubkey());
    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::InitializeUser {
            payer: authority,
            user_profile: user_pda(&authority),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::InitializeUser {}.data(),
    };
    send_ix(svm, payer, ix, &[]);
}

fn post_bounty(svm: &mut LiteSVM, poster: &Keypair, id: u64, amount: u64) {
    let poster_pk = to_pubkey(poster.pubkey());
    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::PostBounty {
            payer: poster_pk,
            bounty_state: bounty_pda(&poster_pk, id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::PostBounty { id, amount }.data(),
    };
    send_ix(svm, poster, ix, &[]);
}

fn submit_task(
    svm: &mut LiteSVM,
    submitter: &Keypair,
    poster: &Pubkey,
    id: u64,
    submission_uri: String,
) {
    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::SubmitTask {
            submitter: to_pubkey(submitter.pubkey()),
            poster: *poster,
            bounty_state: bounty_pda(poster, id),
        }
        .to_account_metas(None),
        data: instruction::SubmitTask {
            id,
            submission_uri,
        }
        .data(),
    };
    send_ix(svm, submitter, ix, &[]);
}

fn approve_task(svm: &mut LiteSVM, poster: &Keypair, claimer: &Pubkey, id: u64) {
    let poster_pk = to_pubkey(poster.pubkey());
    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::ApproveTask {
            poster: poster_pk,
            claimer: *claimer,
            bounty_state: bounty_pda(&poster_pk, id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::ApproveTask { id }.data(),
    };
    send_ix(svm, poster, ix, &[]);
}

fn cancel_task(svm: &mut LiteSVM, poster: &Keypair, id: u64) {
    let poster_pk = to_pubkey(poster.pubkey());
    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::CancelTask {
            poster: poster_pk,
            bounty_state: bounty_pda(&poster_pk, id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CancelTask { id }.data(),
    };
    send_ix(svm, poster, ix, &[]);
}

fn fetch_bounty(svm: &LiteSVM, poster: &Pubkey, id: u64) -> BountyState {
    let account = svm
        .get_account(&to_sol_pubkey(bounty_pda(poster, id)))
        .expect("bounty account exists");
    BountyState::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn lamports_of(svm: &LiteSVM, pubkey: &Pubkey) -> u64 {
    svm.get_account(&to_sol_pubkey(*pubkey))
        .map(|a| a.lamports)
        .unwrap_or(0)
}

#[test]
fn test_initialize_global_state() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    airdrop(&mut svm, &payer, 10_000_000_000);

    initialize_global(&mut svm, &payer);

    let account = svm
        .get_account(&to_sol_pubkey(global_pda()))
        .expect("global state exists");
    let state = GlobalState::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(state.authority, to_pubkey(payer.pubkey()));
}

#[test]
fn test_initialize_and_update_user() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    airdrop(&mut svm, &payer, 10_000_000_000);

    initialize_user(&mut svm, &payer);

    let authority = to_pubkey(payer.pubkey());
    let account = svm
        .get_account(&to_sol_pubkey(user_pda(&authority)))
        .expect("user profile exists");
    let profile = UserProfile::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(profile.authority, authority);
    assert_eq!(profile.reputation, 0);

    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::UpdateUser {
            payer: authority,
            user_profile: user_pda(&authority),
        }
        .to_account_metas(None),
        data: instruction::UpdateUser { data: 42 }.data(),
    };
    send_ix(&mut svm, &payer, ix, &[]);

    let account = svm
        .get_account(&to_sol_pubkey(user_pda(&authority)))
        .unwrap();
    let profile = UserProfile::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(profile.reputation, 42);
}

#[test]
fn test_post_bounty_creates_open_escrow() {
    let mut svm = setup_svm();
    let poster = Keypair::new();
    airdrop(&mut svm, &poster, 10_000_000_000);

    let id = 1u64;
    let amount = 1_000_000u64;
    let poster_pk = to_pubkey(poster.pubkey());
    let bounty = bounty_pda(&poster_pk, id);
    let before = lamports_of(&svm, &poster_pk);

    post_bounty(&mut svm, &poster, id, amount);

    let state = fetch_bounty(&svm, &poster_pk, id);
    assert_eq!(state.id, id);
    assert_eq!(state.poster, poster_pk);
    assert_eq!(state.claimer, Pubkey::default());
    assert_eq!(state.amount, amount);
    assert_eq!(state.status, BountyStatus::Open);
    assert!(state.submisstion_uri.is_empty());

    assert!(lamports_of(&svm, &bounty) >= amount);
    assert!(lamports_of(&svm, &poster_pk) < before);
}

#[test]
fn test_full_bounty_flow_submit_and_approve() {
    let mut svm = setup_svm();
    let poster = Keypair::new();
    let developer = Keypair::new();
    airdrop(&mut svm, &poster, 10_000_000_000);
    airdrop(&mut svm, &developer, 10_000_000_000);

    let id = 7u64;
    let amount = 2_000_000u64;
    let poster_pk = to_pubkey(poster.pubkey());
    let developer_pk = to_pubkey(developer.pubkey());

    post_bounty(&mut svm, &poster, id, amount);

    let developer_before = lamports_of(&svm, &developer_pk);
    submit_task(
        &mut svm,
        &developer,
        &poster_pk,
        id,
        "ipfs://bafy-test-proof".to_string(),
    );

    let claimed = fetch_bounty(&svm, &poster_pk, id);
    assert_eq!(claimed.status, BountyStatus::Claimed);
    assert_eq!(claimed.claimer, developer_pk);
    assert_eq!(claimed.submisstion_uri, "ipfs://bafy-test-proof");

    approve_task(&mut svm, &poster, &developer_pk, id);

    let closed = fetch_bounty(&svm, &poster_pk, id);
    assert_eq!(closed.status, BountyStatus::Closed);
    assert_eq!(closed.amount, 0);
    assert!(lamports_of(&svm, &developer_pk) >= developer_before + amount - 10_000);
}

#[test]
fn test_cancel_open_bounty_refunds_poster() {
    let mut svm = setup_svm();
    let poster = Keypair::new();
    airdrop(&mut svm, &poster, 10_000_000_000);

    let id = 3u64;
    let amount = 1_500_000u64;
    let poster_pk = to_pubkey(poster.pubkey());

    post_bounty(&mut svm, &poster, id, amount);
    let after_post = lamports_of(&svm, &poster_pk);

    cancel_task(&mut svm, &poster, id);

    let cancelled = fetch_bounty(&svm, &poster_pk, id);
    assert_eq!(cancelled.status, BountyStatus::Closed);
    assert_eq!(cancelled.amount, 0);
    assert!(lamports_of(&svm, &poster_pk) > after_post);
}

#[test]
fn test_submit_own_bounty_fails() {
    let mut svm = setup_svm();
    let poster = Keypair::new();
    airdrop(&mut svm, &poster, 10_000_000_000);

    let id = 9u64;
    let poster_pk = to_pubkey(poster.pubkey());
    post_bounty(&mut svm, &poster, id, 1_000_000);

    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::SubmitTask {
            submitter: poster_pk,
            poster: poster_pk,
            bounty_state: bounty_pda(&poster_pk, id),
        }
        .to_account_metas(None),
        data: instruction::SubmitTask {
            id,
            submission_uri: "http://evil".to_string(),
        }
        .data(),
    };
    send_ix_expect_err(&mut svm, &poster, ix, &[]);
}

#[test]
fn test_cancel_claimed_bounty_fails() {
    let mut svm = setup_svm();
    let poster = Keypair::new();
    let developer = Keypair::new();
    airdrop(&mut svm, &poster, 10_000_000_000);
    airdrop(&mut svm, &developer, 10_000_000_000);

    let id = 11u64;
    let poster_pk = to_pubkey(poster.pubkey());
    post_bounty(&mut svm, &poster, id, 1_000_000);
    submit_task(
        &mut svm,
        &developer,
        &poster_pk,
        id,
        "ipfs://claimed".to_string(),
    );

    let ix = Instruction {
        program_id: to_sol_pubkey(PROGRAM_ID),
        accounts: accounts::CancelTask {
            poster: poster_pk,
            bounty_state: bounty_pda(&poster_pk, id),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CancelTask { id }.data(),
    };
    send_ix_expect_err(&mut svm, &poster, ix, &[]);
}
