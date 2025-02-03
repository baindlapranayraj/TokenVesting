use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Grant {
    pub grant_mint: Pubkey,
    pub employee: Pubkey,
    pub total_amount_locked: u64, // changable state
    pub amount_unlocked: u64,     // changable state
    pub grant_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct GrantShecdule {
    pub cliff_date: u64,             // no change
    pub start_date: u64,             // no change
    pub end_date: u64,               // no change
    pub no_of_months_completed: u64, // changable state
    pub total_period: u32,
    pub shecdule_bump: u8, // no change
}
// all dates are in unix_timestamp
