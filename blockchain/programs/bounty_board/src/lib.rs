pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use anchor_lang::system_program::Transfer;
pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Hw8AqRxBe2BxAS5AByHThA5A8xNWAifPx514B9FDKfq9");

#[program]
pub mod bounty_board {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.authority = *ctx.accounts.payer.key;
        global_state.bump = ctx.bumps.global_state;
        msg!("global state initialized!");
        Ok(())
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        let user_profile = &mut ctx.accounts.user_profile;
        user_profile.authority = *ctx.accounts.payer.key;
        user_profile.reputation = 0;
        msg!("user profile initialized!");
        Ok(())
    }
    pub fn update_user(ctx: Context<UpdateUser>, data: u64) -> Result<()> {
        let user_profile = &mut ctx.accounts.user_profile;
        user_profile.reputation = data;
        Ok(())
    }

    pub fn post_bounty(ctx: Context<PostBounty>, id: u64, amount: u64) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty_state;
        bounty.id = id;
        bounty.poster = *ctx.accounts.payer.key;
        bounty.claimer = Pubkey::default();
        bounty.amount = amount;
        bounty.status = BountyStatus::Open;
        bounty.submisstion_uri = String::new();

        let cpi_accounts = Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.bounty_state.to_account_info(),
        };
        let cpi_context = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);
        anchor_lang::system_program::transfer(cpi_context, amount)?;
        Ok(())
    }

    pub fn update_bounty(
        ctx: Context<UpdateBounty>,
        new_amount: Option<u64>,
        new_status: Option<BountyStatus>,
    ) -> Result<()> {
        let bounty = &mut ctx.accounts.bounty_state;

        require!(
            bounty.poster == *ctx.accounts.payer.key,
            crate::error::ErrorCode::Unauthorized
        );

        if let Some(amount) = new_amount {
            bounty.amount = amount;
        }
        if let Some(status) = new_status {
            bounty.status = status;
        }
        Ok(())
    }
    pub fn submit_task(ctx: Context<SubmitTask>, id: u64, submission_uri: String) -> Result<()> {
        require!(submission_uri.len() <= 200, error::ErrorCode::UriTooLong);
        let bounty = &mut ctx.accounts.bounty_state;
        bounty.claimer = *ctx.accounts.submitter.key;
        bounty.submisstion_uri = submission_uri;
        bounty.status = BountyStatus::Claimed;
        msg!("task submitted for bounty {}", id);
        Ok(())
    }
    pub fn approve_task(ctx: Context<ApproveTask>, id: u64) -> Result<()> {
        let amount = ctx.accounts.bounty_state.amount;

        {
            let bounty_info = ctx.accounts.bounty_state.to_account_info();
            let claimer_info = ctx.accounts.claimer.to_account_info();
            **bounty_info.try_borrow_mut_lamports()? -= amount;
            **claimer_info.try_borrow_mut_lamports()? += amount;
        }

        let bounty = &mut ctx.accounts.bounty_state;
        bounty.amount = 0;
        bounty.status = BountyStatus::Closed;
        msg!("task approved for bounty {}, paid {}", id, amount);
        Ok(())
    }
    pub fn cancel_task(ctx: Context<CancelTask>, id: u64) -> Result<()> {
        let amount = ctx.accounts.bounty_state.amount;

        {
            let bounty_info = ctx.accounts.bounty_state.to_account_info();
            let poster_info = ctx.accounts.poster.to_account_info();
            **bounty_info.try_borrow_mut_lamports()? -= amount;
            **poster_info.try_borrow_mut_lamports()? += amount;
        }

        let bounty = &mut ctx.accounts.bounty_state;
        bounty.amount = 0;
        bounty.status = BountyStatus::Closed;
        msg!("bounty {} cancelled, refunded {}", id, amount);
        Ok(())
    }
}
