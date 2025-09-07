#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::MIDDLE_BIN_ID;

    #[test]
    fn test_get_price_from_id_zero_bin_step() {
        // Test with zero bin_step (should return error)
        let bin_step = 0;
        let id = 1000;
        
        let result = get_price_from_id(bin_step, id);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_price_from_id_normal() {
        // Test normal case
        let bin_step = 1; // 0.01%
        let id = MIDDLE_BIN_ID as u32;
        
        let result = get_price_from_id(bin_step, id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_price_from_id_large_id() {
        // Test with large id that might cause overflow
        let bin_step = 1;
        let id = u32::MAX;
        
        let result = get_price_from_id(bin_step, id);
        // This might succeed or fail depending on the math, but should not panic
        // We just want to ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_get_price_from_id_small_id() {
        // Test with small id
        let bin_step = 1;
        let id = 0;
        
        let result = get_price_from_id(bin_step, id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_price_from_id_max_bin_step() {
        // Test with maximum bin_step
        let bin_step = 255;
        let id = MIDDLE_BIN_ID as u32;
        
        let result = get_price_from_id(bin_step, id);
        // This might succeed or fail depending on the math, but should not panic
        let _ = result;
    }
}
