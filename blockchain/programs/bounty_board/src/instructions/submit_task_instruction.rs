use anchor_lang::prelude::*;

use crate::error::ErrorCode;
use crate::state::{BountyState, BountyStatus};

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct SubmitTask<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,

    /// CHECK: Used only as a PDA seed; validated against `bounty_state.poster`.
    pub poster: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        constraint = bounty_state.poster == poster.key() @ ErrorCode::Unauthorized,
        constraint = bounty_state.status == BountyStatus::Open @ ErrorCode::BountyNotOpen,
        constraint = submitter.key() != bounty_state.poster @ ErrorCode::CannotSubmitOwnBounty,
    )]
    pub bounty_state: Account<'info, BountyState>,
}
