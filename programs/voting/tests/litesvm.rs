use std::assert_eq;
use anchor_lang::{self, declare_program};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers, Pubkey, TestHelpers};
use solana_signer::Signer;

declare_program!(voting);

use self::voting::accounts::{CandidateAccount,PollAccount};
use self::voting::client::{accounts,args};

const PROGRAM_BYTES:&[u8] = include_bytes!("../../../target/deploy/voting.so");

fn setup() -> AnchorContext{
    use anchor_lang::solana_program::clock::Clock;

    let mut ctx = AnchorLiteSVM::build_with_program(self::voting::ID, PROGRAM_BYTES);

    let clock = Clock{
        slot:1000,
        epoch_start_timestamp:0,
        epoch:1,
        leader_schedule_epoch:1,
        unix_timestamp:1000,
    };
    ctx.svm.set_sysvar(&clock);
    ctx
}

fn get_poll_pda(poll_id:u64) -> Pubkey{
    Pubkey::find_program_address(&[b"poll",&poll_id.to_le_bytes()], &self::voting::ID).0
}

fn get_candidate_pda(poll_id:u64,candidate:&str) -> Pubkey{
    Pubkey::find_program_address(&[&poll_id.to_le_bytes(),candidate.as_bytes()], &self::voting::ID).0
}

fn initialize_poll(
    ctx:&mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    poll_id:u64,
    start_time:u64,
    end_time:u64,
    name:&str,
    description: &str
){
    let poll_pda = get_poll_pda(poll_id);

    let ix = ctx.program().accounts(accounts::InitPoll{
        signer: signer.pubkey(),
        poll_account: poll_pda,
        system_program: anchor_lang::system_program::ID,
    })
    .args(args::InitPoll{
        _poll_id: poll_id,
        start: start_time,
        end: end_time,
        name: name.to_string(),
        description: description.to_string(),
    })
    .instruction()
    .unwrap();

    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
    ctx.svm.assert_account_exists(&poll_pda);
}

fn intialize_candidate(
    ctx:&mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    poll_id:u64,
    candidate:&str
){
    let ix = ctx.program().accounts(accounts::InitializeCandidate{
        signer: signer.pubkey(),
        poll_account: get_poll_pda(poll_id),
        candidate_account: get_candidate_pda(poll_id, candidate),
        system_program: anchor_lang::system_program::ID,
    }).args(args::InitializeCandidate{
        _poll_id: poll_id,
        candidate: candidate.to_string(),
    }).instruction().unwrap();

    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
}

#[allow(dead_code)]
fn cast_vote(
    ctx:&mut AnchorContext,
    signer: &anchor_litesvm::Keypair,
    poll_id:u64,
    candidate:&str
)-> anchor_litesvm::TransactionResult{
    let ix = ctx.program().accounts(accounts::Voting{
        signer: signer.pubkey(),
        poll_account: get_poll_pda(poll_id),
        candidate_account: get_candidate_pda(poll_id, candidate),
    })
    .args(args::Voting{
        _poll_id: poll_id,
        _candidate: candidate.to_string(),
    }).instruction().unwrap();

    ctx.execute_instruction(ix, &[signer]).unwrap()
}

#[test]
fn test_init_poll(){
    let mut ctx =  setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id =  1;
    initialize_poll(&mut ctx, &user, poll_id, 0,u64::MAX, "Test Poll","Test Poll for Voting");

    intialize_candidate(&mut ctx, &user, poll_id, "Alice");
    intialize_candidate(&mut ctx, &user, poll_id, "Bob");

    let poll_account:PollAccount = ctx.get_account(&get_poll_pda(poll_id)).unwrap();
    let aliceaccount:CandidateAccount = ctx.get_account(&get_candidate_pda(poll_id, "Alice")).unwrap();
    let bobaccount:CandidateAccount = ctx.get_account(&get_candidate_pda(poll_id, "Bob")).unwrap();

    assert_eq!(poll_account.poll_option_index,2);
    assert_eq!(aliceaccount.candidate_name,"Alice");
    assert_eq!(aliceaccount.candidate_votes,0);
    assert_eq!(bobaccount.candidate_name,"Bob");
    assert_eq!(bobaccount.candidate_votes,0);
}

#[test]
fn test_init_candidates() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    initialize_poll(&mut ctx, &user, poll_id, 0, u64::MAX, "Test Poll", "A test poll for voting");
    intialize_candidate(&mut ctx, &user, poll_id, "Alice");
    intialize_candidate(&mut ctx, &user, poll_id, "Bob");

    let poll_account:PollAccount = ctx.get_account(&get_poll_pda(poll_id)).unwrap();
    let aliceaccount:CandidateAccount = ctx.get_account(&get_candidate_pda(poll_id, "Alice")).unwrap();
    let bobaccount:CandidateAccount = ctx.get_account(&get_candidate_pda(poll_id, "Bob")).unwrap();

    assert_eq!(poll_account.poll_option_index,2);
    assert_eq!(aliceaccount.candidate_name,"Alice");
    assert_eq!(aliceaccount.candidate_votes,0);
    assert_eq!(bobaccount.candidate_name,"Bob");
    assert_eq!(bobaccount.candidate_votes,0);
}

#[test]
fn test_vote(){
    let mut ctx =  setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    initialize_poll(&mut ctx, &authority, poll_id, 0, 2_000, "Test Poll", "It is a Test Poll Voting");
    intialize_candidate(&mut ctx, &authority, poll_id, "Alice");
    let result = cast_vote(&mut ctx, &voter, poll_id, "Alice");
    result.assert_success();

    let alice_account:CandidateAccount = ctx.get_account(&get_candidate_pda(poll_id, "Alice")).unwrap();
    assert_eq!(alice_account.candidate_votes,1);
}

#[test]
fn test_vote_before_start_fails(){
    let mut ctx = setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    initialize_poll(&mut ctx, &authority, poll_id, 1_500, 2_000, "Future Poll", "Voting has not yet started");
    intialize_candidate(&mut ctx, &authority, poll_id, "Alice");

    cast_vote(&mut ctx, &voter, poll_id, "Alice").assert_failure().assert_anchor_error("VotingNotStarted");
}

#[test]
fn test_vote_after_end_fails(){
    let mut ctx = setup();
    let authority = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    initialize_poll(&mut ctx, &authority, poll_id, 0, 500, "Closed Poll", "Voting has ended");
    intialize_candidate(&mut ctx, &authority, poll_id, "Alice");

    cast_vote(&mut ctx, &voter, poll_id, "Alice").assert_failure().assert_anchor_error("VotingEnded");
}