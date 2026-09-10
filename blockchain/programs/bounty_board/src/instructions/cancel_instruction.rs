use crate::error::ErrorCode;
use crate::state::{BountyState, BountyStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct CancelTask<'info> {
    #[account(mut)]
    pub poster: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        constraint = bounty_state.poster == poster.key() @ ErrorCode::Unauthorized,
        constraint = bounty_state.status == BountyStatus::Open @ ErrorCode::BountyNotOpen,
    )]
    pub bounty_state: Account<'info, BountyState>,

    pub system_program: Program<'info, System>,
}
