use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BountyState {
    pub id: u64,
    pub poster: Pubkey,
    pub amount: u64,
    pub status: BountyStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum BountyStatus {
    Open,
    Claimed,
    Closed,
}

#[account]
#[derive(InitSpace)]
pub struct UserProfile {
    pub authority: Pubkey,
    pub reputation: u64,
}
