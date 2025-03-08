use std::u64;

use chrono::{Datelike, NaiveDateTime};

use crate::errors::VestingErrors;
use anchor_lang::prelude::*;

pub struct ShareCalculateArg {
    pub current_unix_time: i64,
    pub start_date_unix_time: i64,
    pub last_claim_unix_time: i64,
    pub cliff_date_unix_time: i64,
    pub vault_total_amount: u64,
    pub total_period: u64,
}

pub struct CalulateRes {
    pub cliff_period: u32,
    pub total_months_completed: u32,
    pub recent_month_taken: u32,
    pub shares: u64,
}

pub fn calculate_shares(arg: ShareCalculateArg) -> Result<CalulateRes> {
    #[allow(deprecated)]
    let start_date = NaiveDateTime::from_timestamp_opt(arg.start_date_unix_time, 0)
        .ok_or(VestingErrors::InvalidTimeStamp)?;

    #[allow(deprecated)]
    let current_date = NaiveDateTime::from_timestamp_opt(arg.current_unix_time, 0)
        .ok_or(VestingErrors::InvalidTimeStamp)?;

    #[allow(deprecated)]
    let prev_recive_date = NaiveDateTime::from_timestamp_opt(arg.last_claim_unix_time, 0)
        .ok_or(VestingErrors::InvalidTimeStamp)?;

    #[allow(deprecated)]
    let cliff_date = NaiveDateTime::from_timestamp_opt(arg.cliff_date_unix_time, 0).unwrap();

    let total_months_completed = ((current_date.year() - start_date.year()) as u32) * 12
        + (current_date.month() - start_date.month()); // total no.of months done since start

    let recent_month_taken = match arg.last_claim_unix_time == 0 {
        true => 0,
        false => {
            ((prev_recive_date.year() - start_date.year()) as u32) * 12
                + (prev_recive_date.month() - start_date.month())
        }
    };

    let cliff_period = ((cliff_date.year() - start_date.year()) as u32) * 12
        + (cliff_date.month() - start_date.month()); // cliff

    // Calculate shares only if the cliff period has passed
    let shares = if total_months_completed > cliff_period {
        (arg.vault_total_amount / arg.total_period)
            * ((total_months_completed - cliff_period) as u64)
    } else {
        0
    };

    Ok(CalulateRes {
        cliff_period,
        total_months_completed,
        recent_month_taken,
        shares,
    })
}
