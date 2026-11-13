use anchor_lang::{self, declare_program};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM, AssertionHelpers, Pubkey};
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