use anchor_lang::prelude::*;

#[error_code]
pub enum VestingErrors {
    #[msg("The vault is empty.")]
    EmptyVault,

    #[msg("Cannot claim before the cliff period.")]
    ClaimBeforeCliff,

    #[msg("Shares for the current month have already been taken.")]
    AlreadyTakenCurrentMonthShares,

    #[msg("No shares available for claiming.")]
    NoSharesAvailable,

    #[msg("Insufficient funds in the vault.")]
    InsufficientFunds,

    #[msg("The Given Timestamp is Invalid")]
    InvalidTimeStamp,

    #[msg("Overflow Error Occured")]
    OverflowError,
}
