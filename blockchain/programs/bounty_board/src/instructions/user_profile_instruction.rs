use crate::state::UserProfile;
use anchor_lang::prelude::*;
#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + UserProfile::INIT_SPACE,
        seeds = [b"user", payer.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = payer.key() == user_profile.authority,
        seeds = [b"user", payer.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,
}
