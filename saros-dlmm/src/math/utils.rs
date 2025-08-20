use crate::constants::{BASIS_POINT_MAX, PRECISION};
use crate::errors::ErrorCode;
use anyhow::Result;

pub fn get_protocol_fee(fee: u64, protocol_share: u64) -> u64 {
    u64::try_from(u128::from(fee) * u128::from(protocol_share) / u128::from(BASIS_POINT_MAX))
        .unwrap()
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

    // Add validation to prevent underflow when fee >= PRECISION
    if fee >= PRECISION {
        return Err(ErrorCode::AmountOverflow.into());
    }

    let denominator = u128::from(PRECISION) - fee;

    let fee_for_amount = amount
        .checked_mul(fee)
        .ok_or(ErrorCode::AmountOverflow)?
        .checked_add(denominator)
        .ok_or(ErrorCode::AmountOverflow)?
        .checked_sub(1)
        .ok_or(ErrorCode::AmountUnderflow)?
        / denominator;

    let fee_for_amount = u64::try_from(fee_for_amount).map_err(|_| ErrorCode::AmountOverflow)?;

    Ok(fee_for_amount)
}

pub fn convert_math_result(opt: Option<u128>, err: ErrorCode) -> Result<u64, ErrorCode> {
    let value = opt.ok_or(err)?;
    u64::try_from(value).map_err(|_| ErrorCode::U64ConversionOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fee_for_amount_normal_case() {
        let amount = 1000;
        let fee = 100_000; // 0.1% fee
        let result = get_fee_for_amount(amount, fee).unwrap();
        assert!(result > 0);
    }

    #[test]
    fn test_get_fee_for_amount_zero_amount() {
        let amount = 0;
        let fee = 100_000;
        let result = get_fee_for_amount(amount, fee).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_fee_for_amount_zero_fee() {
        let amount = 1000;
        let fee = 0;
        let result = get_fee_for_amount(amount, fee).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_get_fee_for_amount_critical_overflow_bug() {
        // This test reproduces the critical bug where fee >= PRECISION
        let amount = 1000;
        let fee = PRECISION; // This should cause underflow in the original code
        let result = get_fee_for_amount(amount, fee);
        
        // With the fix, this should return an error instead of panicking
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().downcast_ref::<ErrorCode>(), Some(ErrorCode::AmountOverflow)));
    }

    #[test]
    fn test_get_fee_for_amount_above_precision() {
        // Test with fee values above PRECISION
        let amount = 1000;
        let fee = PRECISION + 1;
        let result = get_fee_for_amount(amount, fee);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().downcast_ref::<ErrorCode>(), Some(ErrorCode::AmountOverflow)));
    }

    #[test]
    fn test_get_fee_for_amount_boundary_case() {
        // Test with fee just below PRECISION (should work)
        let amount = 1000;
        let fee = PRECISION - 1;
        let result = get_fee_for_amount(amount, fee);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_fee_amount_normal_case() {
        let amount = 1000;
        let fee = 100_000; // 0.1% fee
        let result = get_fee_amount(amount, fee).unwrap();
        assert!(result > 0);
    }

    #[test]
    fn test_get_protocol_fee() {
        let fee = 1000;
        let protocol_share = 5000; // 50%
        let result = get_protocol_fee(fee, protocol_share);
        assert_eq!(result, 500); // 50% of 1000
    }
}
