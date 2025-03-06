use anchor_lang::{prelude::*, Discriminator};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use chrono::prelude::*;

use crate::{
    constant::*,
    errors::VestingErrors,
    state::{Grant, GrantShecdule},
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitGrantArg {
    pub cliff_date: i64,
    pub start_date: i64,
    pub end_date: i64,
    pub grant_deposited: u64,
}

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
        token::mint = grant_mint, // 1)
        token::authority = grant,
        seeds = [VAULT_SEED,grant.key().to_bytes().as_ref()],
        bump
    )]
    pub grant_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = employer,
        space = Grant::INIT_SPACE + Grant::DISCRIMINATOR.len(),
        seeds = [GRANT,employer.key().to_bytes().as_ref(),employee.key().to_bytes().as_ref()],
        bump
    )]
    pub grant: Account<'info, Grant>,

    #[account(
        init,
        payer = employer,
        space = GrantShecdule::INIT_SPACE + GrantShecdule::DISCRIMINATOR.len(),
        seeds = [GRANT_SCHEDULE,employer.key().to_bytes().as_ref(),employee.key().to_bytes().as_ref()],
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
        cliff_date: i64,
        start_date: i64,
        end_date: i64,
        bump: InitGrantBumps,
    ) -> Result<()> {
        self.grant.set_inner(Grant {
            total_amount_locked: 0,
            amount_unlocked: 0,
            employee: self.employee.key(),
            grant_mint: self.grant_mint.key(),
            grant_bump: bump.grant,
            vault_bump: bump.grant_vault,
        });

        require_gt!(start_date, 100, VestingErrors::InvalidTimeStamp);

        let start_date_chrono = NaiveDateTime::from_timestamp(start_date, 0);
        let end_date_chrono = NaiveDateTime::from_timestamp(end_date, 0);

        // Will give you total no.of periods in months
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

// +++++++++++++++++++ Equations +++++++++++++++++++

// - Equation for calculating total number of months:-
// (YearCurrent - YearFrom)*12 + (MonthCurrent - MonthFrom) = Total no.of Months
//
//
// - Eqaution for calculating Vesting Amount:-
// (TotalAmount/TotalPeriod)*(CurrentTotal months - Cliff)
//

// +++++++++++++++++++ Learnigs +++++++++++++++++++
// 1) We are using token rather then associated_token bcoz this vault account should be specific to
//    this employee and employer
//
//
