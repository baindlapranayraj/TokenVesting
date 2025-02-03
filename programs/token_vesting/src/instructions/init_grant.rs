use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use chrono::prelude::*;

use crate::state::{Grant, GrantShecdule};

#[derive(Accounts)]
pub struct InitGrant<'info> {
    #[account(mut)]
    pub employer: Signer<'info>,

    #[account(mut)]
    pub employee: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = grant_mint,
        associated_token::authority = employer
    )]
    pub employer_token: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = employer,
        associated_token::mint = grant_mint,
        associated_token::authority = grant
    )]
    pub grant_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = employer,
        space = Grant::INIT_SPACE,
        seeds = [b"grant",employer.key().as_ref(),employee.key().as_ref()],
        bump
    )]
    pub grant: Account<'info, Grant>,

    #[account(
        init,
        payer = employer,
        space = GrantShecdule::INIT_SPACE,
        seeds = [b"grant-schedule",employer.key().as_ref(),employee.key().as_ref()],
        bump
    )]
    pub grant_shecdule: Account<'info, GrantShecdule>,

    pub grant_mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitGrant<'info> {
    pub fn initialize_grant(
        &mut self,
        cliff_date: u64,
        start_date: u64,
        end_date: u64,
        bump: InitGrantBumps,
    ) -> Result<()> {
        self.grant.set_inner(Grant {
            total_amount_locked: 0,
            amount_unlocked: 0,
            employee: self.employee.key(),
            grant_mint: self.grant_mint.key(),
            grant_bump: bump.grant,
        });

        let start_date_chrono = NaiveDateTime::from_timestamp(start_date as i64, 0);
        let end_date_chrono = NaiveDateTime::from_timestamp(end_date as i64, 0);

        // Will give you total no.of periods
        let total_period = ((end_date_chrono.year() - start_date_chrono.year()) as u32) * 12
            + (end_date_chrono.month() - start_date_chrono.month());

        self.grant_shecdule.set_inner(GrantShecdule {
            cliff_date,
            start_date,
            end_date,
            total_period,
            no_of_months_completed: 0,
            shecdule_bump: bump.grant_shecdule,
        });
        Ok(())
    }

    pub fn deposite_grant(&mut self, grant_deposit: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let transfer_accounts = TransferChecked {
            authority: self.employer.to_account_info(),
            mint: self.grant_mint.to_account_info(),
            from: self.employer_token.to_account_info(),
            to: self.grant_vault.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, transfer_accounts);
        transfer_checked(cpi_context, grant_deposit, self.grant_mint.decimals)?;

        // Changing the state
        self.grant.total_amount_locked = grant_deposit;

        Ok(())
    }
}

// - Equation for calculating months:-
// (YearCurrent - YearFrom)*12 + (MonthCurrent - MonthFrom) = Total no.of Months
//
//
// - Eqaution for calculating Vesting Amount:-
// (TotalAmount/TotalPeriod)*(CurrentTotal months - Cliff)
