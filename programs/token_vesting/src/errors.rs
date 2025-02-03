use anchor_lang::prelude::*;

#[error_code]
pub enum VestingErrors {
    #[msg("You have already taken current month shares")]
    AlreadyTakenCurrentMonthShares,

    #[msg("You are trying to take money before cliff period")]
    ClaimBeforeCliff,

    #[msg("Your vault is empty")]
    EmptyVault,
}
