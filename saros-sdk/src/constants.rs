use solana_sdk::{pubkey, pubkey::Pubkey};

pub const MAX_PROTOCOL_SHARE: u16 = 2_500;
pub const BASIS_POINT_MAX: u64 = 10_000;
pub const VARIABLE_FEE_PRECISION: u128 = 100_000_000_000;
pub const PRECISION: u64 = 1_000_000_000;
pub const SQUARED_PRECISION: u128 = 1_000_000_000_000_000_000;
pub const MAX_ACTIVE_ID: u32 = 16_777_215; // 2^24 - 1
pub const MIDDLE_BIN_ID: i32 = 8_388_608; // 2^23
pub const MAX_BIN_CROSSING: u32 = 30; // Maximum number of bins that can be crossed in a swap

pub const HOOK_PROGRAM_ID: Pubkey = pubkey!("BXpvgXGKDKax82p9JK2E8jRoKLify6eq3mMaW1wHVhuK");
