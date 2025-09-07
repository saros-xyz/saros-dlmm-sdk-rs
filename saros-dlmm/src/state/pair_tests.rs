#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{BASIS_POINT_MAX, VARIABLE_FEE_PRECISION};

    #[test]
    fn test_get_variable_fee_overflow() {
        // Create a pair with maximum volatility accumulator
        let mut pair = create_test_pair();
        pair.dynamic_fee_parameters.volatility_accumulator = u32::MAX;
        pair.bin_step = 255; // Maximum bin_step
        
        let result = pair.get_variable_fee();
        // Should return error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_get_variable_fee_normal() {
        // Test normal case
        let mut pair = create_test_pair();
        pair.dynamic_fee_parameters.volatility_accumulator = 1000;
        pair.bin_step = 1;
        pair.static_fee_parameters.variable_fee_control = 1000;
        
        let result = pair.get_variable_fee();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_variable_fee_zero_control() {
        // Test with zero variable fee control
        let mut pair = create_test_pair();
        pair.static_fee_parameters.variable_fee_control = 0;
        
        let result = pair.get_variable_fee();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_composition_fee_overflow() {
        // Test with maximum amount
        let mut pair = create_test_pair();
        let amount = u64::MAX;
        
        let result = pair.get_composition_fee(amount);
        // Should return error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_get_composition_fee_normal() {
        // Test normal case
        let mut pair = create_test_pair();
        let amount = 1000;
        
        let result = pair.get_composition_fee(amount);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_volatility_accumulator_overflow() {
        // Test with values that would cause overflow
        let mut pair = create_test_pair();
        pair.active_id = u32::MAX;
        pair.dynamic_fee_parameters.id_reference = 0;
        pair.static_fee_parameters.max_volatility_accumulator = 1000;
        
        let result = pair.update_volatility_accumulator();
        // Should return error, not panic
        assert!(result.is_err());
    }

    fn create_test_pair() -> Pair {
        Pair {
            _discriminator: [0; 8],
            bump: [0],
            liquidity_book_config: Pubkey::default(),
            bin_step: 1,
            bin_step_seed: [0],
            token_mint_x: Pubkey::default(),
            token_mint_y: Pubkey::default(),
            static_fee_parameters: StaticFeeParameters {
                base_factor: 1000,
                filter_period: 3600,
                decay_period: 3600,
                reduction_factor: 5000,
                variable_fee_control: 1000,
                protocol_share: 1000,
                max_volatility_accumulator: 100000,
            },
            active_id: 8388608, // MIDDLE_BIN_ID
            dynamic_fee_parameters: DynamicFeeParameters {
                volatility_accumulator: 0,
                volatility_reference: 0,
                id_reference: 8388608,
                time_last_updated: 0,
            },
            protocol_fees_x: 0,
            protocol_fees_y: 0,
            hook: None,
        }
    }
}
