use anchor_lang::prelude::*;

use crate::BountyState;

#[derive(Accounts)]
pub struct InitializeBounty<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + BountyState::INIT_SPACE,
        seeds = [b"bounty", payer.key().as_ref()],
        bump
    )]
    pub bounty_state: Account<'info, BountyState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBounty<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"bounty", payer.key().as_ref()],
        bump
    )]
    pub bounty_state: Account<'info, BountyState>,
}

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct PostBounty<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + BountyState::INIT_SPACE,
        seeds = [b"bounty", payer.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub bounty_state: Account<'info, BountyState>,
    pub system_program: Program<'info, System>,
}
