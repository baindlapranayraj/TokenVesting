use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    constant::VAULT_SEED,
    errors::VestingErrors,
    helper::{calculate_shares, ShareCalculateArg},
};
use crate::{
    constant::{GRANT, GRANT_SCHEDULE},
    state::{Grant, GrantShecdule},
};
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
        associated_token::authority = employee,
        associated_token::token_program = token_program
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [GRANT, employer.key().to_bytes().as_ref(), employee.key().to_bytes().as_ref()],
        bump = grant_account.grant_bump,
        has_one = employee,
        has_one = grant_mint
    )]
    pub grant_account: Account<'info, Grant>,

    #[account(
        mut,
        seeds = [GRANT_SCHEDULE, employer.key().to_bytes().as_ref(), employee.key().to_bytes().as_ref()],
        bump = grant_account.grant_bump,
    )]
    pub grant_schedule_account: Account<'info, GrantShecdule>,

    #[account(
        mut,
        token::mint = grant_mint,
        token::authority = grant_account,
        seeds = [VAULT_SEED, grant_account.key().to_bytes().as_ref()],
        bump = grant_account.vault_bump
    )]
    pub grant_vault_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> ClaimGrant<'info> {
    pub fn claim(&mut self) -> Result<()> {
        let unix_time = Clock::get()?.unix_timestamp;

        let cpi_program = self.token_program.to_account_info();
        let transfer_accounts = TransferChecked {
            authority: self.grant_account.to_account_info(),
            from: self.grant_vault_account.to_account_info(),
            to: self.employee_token_account.to_account_info(),
            mint: self.grant_mint.to_account_info(),
        };

        let employer_seed = self.employer.key().to_bytes();
        let employee_seed = self.employee.key().to_bytes();

        let seeds = &[
            GRANT,
            employer_seed.as_ref(),
            employee_seed.as_ref(),
            &[self.grant_account.grant_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let calculate_res = calculate_shares(ShareCalculateArg {
            current_unix_time: unix_time,
            start_date_unix_time: self.grant_schedule_account.start_date,
            last_claim_unix_time: self.grant_schedule_account.no_of_months_completed,
            cliff_date_unix_time: self.grant_schedule_account.cliff_date,
            vault_total_amount: self.grant_account.total_amount_locked,
            total_period: self.grant_schedule_account.total_period as u64,
        })?;

        // Checks
        require!(
            self.grant_account.total_amount_locked != 0,
            VestingErrors::EmptyVault
        );

        require!(
            calculate_res.cliff_period < calculate_res.total_months_completed,
            VestingErrors::ClaimBeforeCliff
        );

        require!(
            calculate_res.recent_month_taken < calculate_res.total_months_completed,
            VestingErrors::AlreadyTakenCurrentMonthShares
        );

        require!(calculate_res.shares > 0, VestingErrors::NoSharesAvailable);

        require!(
            calculate_res.shares <= self.grant_account.total_amount_locked,
            VestingErrors::InsufficientFunds
        );

        // Transfer tokens
        let cpi_context = CpiContext::new_with_signer(cpi_program, transfer_accounts, signer_seeds);
        transfer_checked(cpi_context, calculate_res.shares, self.grant_mint.decimals)?;

        // Update state
        self.grant_account.total_amount_locked = self
            .grant_account
            .total_amount_locked
            .checked_sub(calculate_res.shares)
            .ok_or(VestingErrors::OverflowError)?;

        self.grant_account.amount_unlocked = self
            .grant_account
            .amount_unlocked
            .checked_add(calculate_res.shares)
            .ok_or(VestingErrors::OverflowError)?;

        self.grant_schedule_account.no_of_months_completed = unix_time;

        Ok(())
    }
}
