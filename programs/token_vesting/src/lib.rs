use anchor_lang::prelude::*;

declare_id!("F5Ev8fXiWDFm4f2mQw8MiJ3tePp6bSaGQUZRXzhXuy4p");

pub mod constant;
pub mod errors;
pub mod helper;
pub mod instructions;
pub mod state;

use crate::instructions::*;

#[program]
pub mod token_vesting {
    use super::*;

    pub fn initialize(ctx: Context<InitGrant>, arg: InitGrantArg) -> Result<()> {
        ctx.accounts
            .initialize_grant(arg.cliff_date, arg.start_date, arg.end_date, ctx.bumps)?;

        ctx.accounts.deposite_grant(arg.grant_deposited)?;

        Ok(())
    }

    pub fn revoke_grant(ctx: Context<WithdrawGrant>) -> Result<()> {
        ctx.accounts.withdraw_amount()?;

        ctx.accounts.close_accounts()?;

        Ok(())
    }

    pub fn claim_grant(ctx: Context<ClaimGrant>) -> Result<()> {
        ctx.accounts.claim()?;

        Ok(())
    }
}

// ++++++++++++++++++++ Token Vesting ++++++++++++++++++++
// - Initailize Grant(Init and deposit token)
// - Withdraw Grant(close the account)
// - Claim Grant(Based of some condition we will impl Escrow)
