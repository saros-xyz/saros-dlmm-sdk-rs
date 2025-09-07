use crate::constants::MIDDLE_BIN_ID;
use crate::errors::ErrorCode;
use anyhow::Result;

use super::u64x64_math::{get_base, pow};

pub fn get_price_from_id(bin_step: u8, id: u32) -> Result<u128> {
    if bin_step == 0 {
        return Err(ErrorCode::DivideByZero.into());
    }
    
    let base = get_base(bin_step).ok_or(ErrorCode::NumberCastError)?;
    let exponent = i32::try_from(id)
        .map_err(|_| ErrorCode::NumberCastError)?
        .checked_sub(MIDDLE_BIN_ID)
        .ok_or(ErrorCode::NumberCastError)?;

    pow(base, exponent).ok_or(ErrorCode::NumberCastError.into())
}
