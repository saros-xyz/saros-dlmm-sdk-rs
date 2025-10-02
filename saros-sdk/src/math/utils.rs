use crate::constants::{BASIS_POINT_MAX, PRECISION};
use crate::errors::ErrorCode;
use anyhow::Result;

pub fn get_protocol_fee(fee: u64, protocol_share: u64) -> Result<u64> {
    let value = u128::from(fee) * u128::from(protocol_share) / u128::from(BASIS_POINT_MAX);
    Ok(u64::try_from(value)?)
}

/// Calculate the fee amount taken from the input amount
pub fn get_fee_amount(amount: u64, fee: u64) -> Result<u64> {
    let amount = u128::from(amount);
    let fee = u128::from(fee);

    let fee_amount = amount
        .checked_mul(fee)
        .ok_or(ErrorCode::AmountOverflow)?
        .checked_add(PRECISION.into())
        .ok_or(ErrorCode::AmountOverflow)?
        .checked_sub(1)
        .ok_or(ErrorCode::AmountUnderflow)?
        / u128::from(PRECISION);

    let fee_amount = u64::try_from(fee_amount).map_err(|_| ErrorCode::AmountOverflow)?;

    Ok(fee_amount)
}

/// Calculate the fee to be paid in order to receive the desired amount
pub fn get_fee_for_amount(amount: u64, fee: u64) -> Result<u64> {
    let amount = u128::from(amount);
    let fee = u128::from(fee);

    let denominator = u128::from(PRECISION)
        .checked_sub(fee)
        .ok_or(ErrorCode::AmountUnderflow)?;

    let fee_for_amount = amount
        .checked_mul(fee)
        .ok_or(ErrorCode::AmountOverflow)?
        .checked_add(denominator)
        .ok_or(ErrorCode::AmountOverflow)?
        .checked_sub(1)
        .ok_or(ErrorCode::AmountUnderflow)?
        .checked_div(denominator)
        .ok_or(ErrorCode::DivideByZero)?;

    let fee_for_amount = u64::try_from(fee_for_amount).map_err(|_| ErrorCode::AmountOverflow)?;

    Ok(fee_for_amount)
}

pub fn convert_math_result(opt: Option<u128>, err: ErrorCode) -> Result<u64, ErrorCode> {
    let value = opt.ok_or(err)?;
    u64::try_from(value).map_err(|_| ErrorCode::U64ConversionOverflow)
}
