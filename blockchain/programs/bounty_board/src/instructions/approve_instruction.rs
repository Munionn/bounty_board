use crate::error::ErrorCode;
use crate::state::BountyState;
use crate::state::BountyStatus;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct ApproveTask<'info> {
    pub poster: Signer<'info>,
    #[account(mut)]
    pub claimer: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"bounty", poster.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
        constraint = bounty_state.poster == poster.key() @ ErrorCode::Unauthorized,
        constraint = bounty_state.claimer == claimer.key() @ ErrorCode::Unauthorized,
        constraint = bounty_state.status == BountyStatus::Claimed @ ErrorCode::BountyNotClaimed,
    )]
    pub bounty_state: Account<'info, BountyState>,
    pub system_program: Program<'info, System>,
}
