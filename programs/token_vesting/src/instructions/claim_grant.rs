use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::errors::VestingErrors;
use crate::state::{Grant, GrantShecdule};
use chrono::prelude::*;

#[derive(Accounts)]
pub struct ClaimGrant<'info> {
    #[account(mut)]
    pub employer: SystemAccount<'info>,

    #[account(mut)]
    pub employee: Signer<'info>,

    pub grant_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = employee,
        associated_token::mint = grant_mint,
        associated_token::authority = employee
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"grant",employer.key().as_ref(),employee.key().as_ref()],
        bump = grant_account.grant_bump,
        close = employer
    )]
    pub grant_account: Account<'info, Grant>,

    #[account(
        mut,
        seeds = [b"grant-schedule",employer.key().as_ref(),employee.key().as_ref()],
        bump = grant_account.grant_bump,
        close = employer
    )]
    pub grant_schedule_account: Account<'info, GrantShecdule>,

    #[account(
        mut,
        associated_token::mint = grant_mint,
        associated_token::authority = grant_account
    )]
    pub grant_vault_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimGrant<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        let unix_time = clock.unix_timestamp;

        let total_amount = self.grant_account.total_amount_locked;
        let total_period = self.grant_schedule_account.total_period as u64;

        let cpi_program = self.token_program.to_account_info();
        let transfer_accounts = TransferChecked {
            authority: self.grant_account.to_account_info(),
            from: self.grant_vault_account.to_account_info(),
            to: self.employee_token_account.to_account_info(),
            mint: self.grant_mint.to_account_info(),
        };

        let seeds = &[
            b"grant",
            self.employer.to_account_info().key.as_ref(),
            self.employee.to_account_info().key.as_ref(),
        ];
        let signer_seeds = &[&seeds[..]];

        let start_date =
            NaiveDateTime::from_timestamp_opt(self.grant_schedule_account.start_date as i64, 0)
                .unwrap();

        let current_date = NaiveDateTime::from_timestamp_opt(unix_time, 0).unwrap();
        let prev_recive_date = NaiveDateTime::from_timestamp_opt(
            self.grant_schedule_account.no_of_months_completed as i64,
            0,
        )
        .unwrap();

        let cliff_date =
            NaiveDateTime::from_timestamp_opt(self.grant_schedule_account.cliff_date as i64, 0)
                .unwrap();

        let total_months_completed = ((current_date.year() - start_date.year()) as u32) * 12
            + (current_date.month() - start_date.month());

        let recent_month_taken = ((prev_recive_date.year() - start_date.year()) as u32) * 12
            + (prev_recive_date.month() - start_date.month());

        let cliff_period = ((cliff_date.year() - start_date.year()) as u32) * 12
            + (cliff_date.month() - start_date.month());

        let shares =
            (total_amount / total_period) * ((total_months_completed - cliff_period) as u64);

        // checks
        require!(
            self.grant_account.total_amount_locked != 0,
            VestingErrors::EmptyVault
        );

        require!(
            cliff_period < total_months_completed,
            VestingErrors::ClaimBeforeCliff
        );

        require!(
            recent_month_taken < total_months_completed,
            VestingErrors::AlreadyTakenCurrentMonthShares
        );

        // Transfer token
        let cpi_context = CpiContext::new_with_signer(cpi_program, transfer_accounts, signer_seeds);

        transfer_checked(cpi_context, shares, self.grant_mint.decimals)?;

        // Changing the state
        self.grant_account.total_amount_locked -= shares;
        self.grant_account.amount_unlocked += shares;
        self.grant_schedule_account.no_of_months_completed = unix_time as u64;

        Ok(())
    }
}

// - Eqaution for calculating Vesting Amount:-
// (TotalAmount/TotalPeriod)*(CurrentTotalMonths - Cliff)

// - Equation for calculating months:-
// (YearCurrent - YearFrom)*12 + (MonthCurrent - MonthFrom) = Total no.of Months
