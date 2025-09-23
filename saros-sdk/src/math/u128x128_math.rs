use std::cmp::min;

use ruint::aliases::U256;

// Round up, down
#[derive(PartialEq)]
pub enum Rounding {
    Up,
    Down,
}

/// (x * y) / denominator
pub fn mul_div(x: u128, y: u128, denominator: u128, rounding: Rounding) -> Option<u128> {
    if denominator == 0 {
        return None;
    }

    let x = U256::from(x);
    let y = U256::from(y);
    let denominator = U256::from(denominator);

    let prod = x.checked_mul(y)?;

    match rounding {
        Rounding::Up => prod.div_ceil(denominator).try_into().ok(),
        Rounding::Down => {
            let (quotient, _) = prod.div_rem(denominator);
            quotient.try_into().ok()
        }
    }
}

/// (x * y) >> offset
#[inline]
pub fn mul_shr(x: u128, y: u128, offset: u8, rounding: Rounding) -> Option<u128> {
    let denominator = 1u128.checked_shl(offset.into())?;
    mul_div(x, y, denominator, rounding)
}

/// (x << offset) / y
#[inline]
pub fn shl_div(x: u128, y: u128, offset: u8, rounding: Rounding) -> Option<u128> {
    let scale = 1u128.checked_shl(offset.into())?;
    mul_div(x, scale, y, rounding)
}

pub fn sqrt(x: u128) -> u128 {
    if x == 0 {
        return 0;
    };

    let msb = most_significant_bit(x);

    let mut sqrt_x = 1u128 << (msb >> 1);

    sqrt_x = (sqrt_x + x / sqrt_x) >> 1;
    sqrt_x = (sqrt_x + x / sqrt_x) >> 1;
    sqrt_x = (sqrt_x + x / sqrt_x) >> 1;
    sqrt_x = (sqrt_x + x / sqrt_x) >> 1;
    sqrt_x = (sqrt_x + x / sqrt_x) >> 1;
    sqrt_x = (sqrt_x + x / sqrt_x) >> 1;

    min(sqrt_x, x / sqrt_x)
}

fn most_significant_bit(x: u128) -> u8 {
    let mut x_mut = x;
    let mut most_significant_bit: u8 = 0;

    if x_mut > 0xffffffffffffffff {
        x_mut >>= 64;
        most_significant_bit = 64;
    }
    if x_mut > 0xffffffff {
        x_mut >>= 32;
        most_significant_bit += 32;
    }
    if x_mut > 0xffff {
        x_mut >>= 16;
        most_significant_bit += 16;
    }
    if x_mut > 0xff {
        x_mut >>= 8;
        most_significant_bit += 8;
    }
    if x_mut > 0xf {
        x_mut >>= 4;
        most_significant_bit += 4;
    }
    if x_mut > 0x3 {
        x_mut >>= 2;
        most_significant_bit += 2;
    }
    if x_mut > 0x1 {
        most_significant_bit += 1;
    }

    most_significant_bit
}

#[cfg(test)]
mod fuzz_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100_000))]

        #[test]
        fn test_sqrt(x: u128) {
            let sqrt_x = sqrt(x);

            assert!(sqrt_x * sqrt_x <= x);

            let sqrt_x_plus_1 = sqrt_x + 1;
            let sqrt_x_plus_1_squared = sqrt_x_plus_1 * sqrt_x_plus_1;

            if sqrt_x_plus_1_squared / sqrt_x_plus_1 == sqrt_x_plus_1 {
                assert!(sqrt_x_plus_1_squared > x);
            }
        }

        #[test]
        fn test_most_significant_bit(x: u128) {
            let msb = most_significant_bit(x);

            assert!(1 << msb < x);

            if msb < 127 {
                assert!(1 << (msb + 1) > x);
            }
        }
    }
}
