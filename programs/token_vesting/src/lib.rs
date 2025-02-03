use anchor_lang::prelude::*;

declare_id!("F5Ev8fXiWDFm4f2mQw8MiJ3tePp6bSaGQUZRXzhXuy4p");

pub mod errors;
pub mod instructions;
pub mod state;

use crate::{instructions::*, state::*};

#[program]
pub mod token_vesting {
    use super::*;

    pub fn initialize(
        ctx: Context<InitGrant>,
        cliff_date: u64,
        start_date: u64,
        end_date: u64,
        grant_deposited: u64,
    ) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);

        ctx.accounts
            .initialize_grant(cliff_date, start_date, end_date, ctx.bumps)?;

        ctx.accounts.deposite_grant(grant_deposited)?;

        Ok(())
    }

    pub fn revoke_grant(ctx: Context<WithdrawGrant>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);

        ctx.accounts.withdraw_amount()?;

        ctx.accounts.close_accounts()?;

        Ok(())
    }

    pub fn claim_grant(ctx: Context<ClaimGrant>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);

        ctx.accounts.claim()?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

// ++++++++++++++++++++ Token Vesting ++++++++++++++++++++
// - Initailize Grant(Init and deposit token)
// - Withdraw Grant(close the account)
// - Claim Grant(Based of some condition we will impl Escrow)
