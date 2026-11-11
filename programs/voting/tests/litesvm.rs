use anchor_lang::{self, declare_program};
use anchor_litesvm::{AnchorContext, AnchorLiteSVM};

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

