pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

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
}
