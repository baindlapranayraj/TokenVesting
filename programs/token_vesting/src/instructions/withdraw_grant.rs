use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::{
    constant::{GRANT, GRANT_SCHEDULE, VAULT_SEED},
    state::{Grant, GrantShecdule},
};

#[derive(Accounts)]
pub struct WithdrawGrant<'info> {
    #[account(mut)]
    pub employer: Signer<'info>,

    #[account(mut)]
    pub employee: SystemAccount<'info>,

    pub grant_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = grant_mint,
        associated_token::authority = employer
    )]
    pub employer_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [GRANT,employer.key().to_bytes().as_ref(),employee.key().to_bytes().as_ref()],
        bump = grant_account.grant_bump,
        close = employer
    )]
    pub grant_account: Account<'info, Grant>,

    #[account(
        mut,
        seeds = [GRANT_SCHEDULE,employer.key().to_bytes().as_ref(),employee.key().to_bytes().as_ref()],
        bump = grant_account.grant_bump,
        close = employer
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

impl<'info> WithdrawGrant<'info> {
    pub fn withdraw_amount(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let accounts = TransferChecked {
            mint: self.grant_mint.to_account_info(),
            authority: self.grant_account.to_account_info(),
            from: self.grant_vault_account.to_account_info(),
            to: self.employer_token_account.to_account_info(),
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

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, accounts, signer_seeds);

        transfer_checked(
            cpi_ctx,
            self.grant_vault_account.amount,
            self.grant_mint.decimals,
        )?;

        Ok(())
    }

    pub fn close_accounts(&mut self) -> Result<()> {
        let seeds = &[
            GRANT,
            self.employer.to_account_info().key.as_ref(),
            self.employee.to_account_info().key.as_ref(),
            &[self.grant_account.grant_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.grant_vault_account.to_account_info(),
                destination: self.employer.to_account_info(),
                authority: self.grant_account.to_account_info(),
            },
            signer_seeds,
        );

        close_account(ctx)?;

        Ok(())
    }
}
