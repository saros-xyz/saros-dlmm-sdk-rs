use crate::constants::MIDDLE_BIN_ID;

use super::u64x64_math::{get_base, pow};

pub fn get_price_from_id(bin_step: u8, id: u32) -> u128 {
    let base = get_base(bin_step).unwrap();
    let exponent = i32::try_from(id)
        .unwrap()
        .checked_sub(MIDDLE_BIN_ID)
        .unwrap();

    pow(base, exponent).unwrap()
}
