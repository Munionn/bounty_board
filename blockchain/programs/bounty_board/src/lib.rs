pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Hw8AqRxBe2BxAS5AByHThA5A8xNWAifPx514B9FDKfq9");

#[program]
pub mod bounty_board {
    use super::*;

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
        bounty.amount = amount;
        bounty.status = BountyStatus::Open;

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
    pub fn post_bounty_task(ctx: Context<BountyState>) -> Result<()> {
        Ok(())
    }
}
