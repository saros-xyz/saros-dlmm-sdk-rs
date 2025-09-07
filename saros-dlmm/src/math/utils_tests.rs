#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::BASIS_POINT_MAX;

    #[test]
    fn test_get_protocol_fee_overflow() {
        // Test case that would cause overflow
        let fee = u64::MAX;
        let protocol_share = BASIS_POINT_MAX;
        
        // Should return error, not panic
        let result = get_protocol_fee(fee, protocol_share);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_protocol_fee_normal() {
        // Test normal case
        let fee = 1000;
        let protocol_share = 100; // 1%
        
        let result = get_protocol_fee(fee, protocol_share);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10); // 1000 * 100 / 10000 = 10
    }

    #[test]
    fn test_get_protocol_fee_zero() {
        // Test with zero fee
        let fee = 0;
        let protocol_share = 100;
        
        let result = get_protocol_fee(fee, protocol_share);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_fee_amount_zero_amount() {
        // Test with zero amount
        let amount = 0;
        let fee = 1000;
        
        let result = get_fee_amount(amount, fee);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_fee_amount_overflow() {
        // Test case that would cause overflow
        let amount = u64::MAX;
        let fee = u64::MAX;
        
        let result = get_fee_amount(amount, fee);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_fee_for_amount_zero_amount() {
        // Test with zero amount
        let amount = 0;
        let fee = 1000;
        
        let result = get_fee_for_amount(amount, fee);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_fee_for_amount_fee_too_large() {
        // Test with fee >= PRECISION (should cause division by zero)
        let amount = 1000;
        let fee = PRECISION; // This should cause denominator to be 0
        
        let result = get_fee_for_amount(amount, fee);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_fee_for_amount_overflow() {
        // Test case that would cause overflow
        let amount = u64::MAX;
        let fee = u64::MAX;
        
        let result = get_fee_for_amount(amount, fee);
        assert!(result.is_err());
    }
}
