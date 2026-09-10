use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the bounty poster can update this bounty")]
    Unauthorized,
    #[msg("Counter has reached the maximum value")]
    CounterOverflow,
    #[msg("Bounty is not open for submission")]
    BountyNotOpen,
    #[msg("Cannot submit work on your own bounty")]
    CannotSubmitOwnBounty,
    #[msg("Bounty is not claimed")]
    BountyNotClaimed,
    #[msg("Bounty is not closed")]
    BountyNotClosed,
    #[msg("Uri is do long")]
    UriTooLong,
}
