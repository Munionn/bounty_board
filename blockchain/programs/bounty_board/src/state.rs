use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BountyState {
    pub id: u64,
    pub poster: Pubkey,
    pub claimer: Pubkey,
    pub amount: u64,
    pub status: BountyStatus,
    #[max_len(200)]
    pub submisstion_uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq, InitSpace)]
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

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub authority: Pubkey,
    pub bump: u8,
}
