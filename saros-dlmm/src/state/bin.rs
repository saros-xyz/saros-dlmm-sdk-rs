use crate::{
    errors::ErrorCode,
    math::utils::{convert_math_result, get_fee_amount, get_protocol_fee},
};
use anyhow::Result;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};

use crate::math::bin_math::get_price_from_id;
use crate::math::u64x64_math::SCALE_OFFSET;
use crate::math::u128x128_math::{Rounding, mul_shr, shl_div};
use crate::math::utils::get_fee_for_amount;

pub const BIN_ARRAY_SIZE: u32 = 256;
pub const BIN_ARRAY_SIZE_USIZE: usize = 256;

#[derive(Default, Clone, Copy)]
pub struct Bin {
    pub total_supply: u128,

    pub reserve_x: u64,
    pub reserve_y: u64,
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for Bin {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Sealed for Bin {}

impl Pack for Bin {
    const LEN: usize = 16 + 8 + 8;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Bin::LEN];
        let (total_supply_dst, reserve_x_dst, reserve_y_dst) = mut_array_refs![output, 16, 8, 8];

        total_supply_dst.copy_from_slice(&self.total_supply.to_le_bytes());
        reserve_x_dst.copy_from_slice(&self.reserve_x.to_le_bytes());
        reserve_y_dst.copy_from_slice(&self.reserve_y.to_le_bytes());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Bin::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (total_supply_src, reserve_x_src, reserve_y_src) = array_refs![input, 16, 8, 8];
        Ok(Self {
            total_supply: u128::from_le_bytes(*total_supply_src),
            reserve_x: u64::from_le_bytes(*reserve_x_src),
            reserve_y: u64::from_le_bytes(*reserve_y_src),
        })
    }
}

impl Bin {
    pub fn swap_exact_in(
        &mut self,
        bin_step: u8,
        bin_id: u32,
        amount_in_left: u64,
        fee: u64,
        protocol_share: u64,
        swap_for_y: bool,
    ) -> Result<(u64, u64, u64, u64)> {
        let price = get_price_from_id(bin_step, bin_id);

        let bin_reserve_out = if swap_for_y {
            self.reserve_y
        } else {
            self.reserve_x
        };

        if bin_reserve_out == 0 {
            return Ok((0, 0, 0, 0));
        }

        let mut max_amount_in = if swap_for_y {
            convert_math_result(
                shl_div(
                    u128::from(bin_reserve_out),
                    price,
                    SCALE_OFFSET,
                    Rounding::Up,
                ),
                ErrorCode::ShlDivMathError,
            )?
        } else {
            convert_math_result(
                mul_shr(
                    u128::from(bin_reserve_out),
                    price,
                    SCALE_OFFSET,
                    Rounding::Up,
                ),
                ErrorCode::MulShrMathError,
            )?
        };

        let max_fee_amount = get_fee_for_amount(max_amount_in, fee)?;

        max_amount_in = max_amount_in
            .checked_add(max_fee_amount)
            .ok_or(ErrorCode::AmountOverflow)?;

        let mut amount_out: u64;
        let amount_in: u64;
        let fee_amount: u64;

        if amount_in_left >= max_amount_in {
            fee_amount = max_fee_amount;

            amount_in = max_amount_in
                .checked_sub(fee_amount)
                .ok_or(ErrorCode::AmountUnderflow)?;

            amount_out = bin_reserve_out;
        } else {
            fee_amount = get_fee_amount(amount_in_left, fee)?;

            amount_in = amount_in_left
                .checked_sub(fee_amount)
                .ok_or(ErrorCode::AmountUnderflow)?;

            amount_out = if swap_for_y {
                convert_math_result(
                    mul_shr(u128::from(amount_in), price, SCALE_OFFSET, Rounding::Down),
                    ErrorCode::MulShrMathError,
                )?
            } else {
                convert_math_result(
                    shl_div(u128::from(amount_in), price, SCALE_OFFSET, Rounding::Down),
                    ErrorCode::ShlDivMathError,
                )?
            };

            if amount_out > bin_reserve_out {
                amount_out = bin_reserve_out;
            }
        }

        let mut protocol_fee_amount = 0;

        if protocol_share > 0 {
            protocol_fee_amount = get_protocol_fee(fee_amount, protocol_share);
        }

        let amount_in_with_fees = amount_in
            .checked_add(fee_amount)
            .ok_or(ErrorCode::AmountOverflow)?;

        if swap_for_y {
            self.reserve_x = self
                .reserve_x
                .checked_add(amount_in_with_fees)
                .ok_or(ErrorCode::AmountOverflow)?
                .checked_sub(protocol_fee_amount)
                .ok_or(ErrorCode::AmountUnderflow)?;

            self.reserve_y = self
                .reserve_y
                .checked_sub(amount_out)
                .ok_or(ErrorCode::AmountUnderflow)?;
        } else {
            self.reserve_x = self
                .reserve_x
                .checked_sub(amount_out)
                .ok_or(ErrorCode::AmountUnderflow)?;

            self.reserve_y = self
                .reserve_y
                .checked_add(amount_in_with_fees)
                .ok_or(ErrorCode::AmountOverflow)?
                .checked_sub(protocol_fee_amount)
                .ok_or(ErrorCode::AmountUnderflow)?;
        }

        Ok((
            amount_in_with_fees,
            amount_out,
            fee_amount,
            protocol_fee_amount,
        ))
    }

    pub fn swap_exact_out(
        &mut self,
        bin_step: u8,
        bin_id: u32,
        amount_out_left: u64,
        fee: u64,
        protocol_share: u64,
        swap_for_y: bool,
    ) -> Result<(u64, u64, u64, u64)> {
        let price = get_price_from_id(bin_step, bin_id);

        let bin_reserve_out = if swap_for_y {
            self.reserve_y
        } else {
            self.reserve_x
        };

        if bin_reserve_out == 0 {
            return Ok((0, 0, 0, 0));
        }

        let amount_out = if amount_out_left > bin_reserve_out {
            bin_reserve_out
        } else {
            amount_out_left
        };

        let amount_in_without_fee = if swap_for_y {
            convert_math_result(
                shl_div(amount_out as u128, price, SCALE_OFFSET, Rounding::Up),
                ErrorCode::ShlDivMathError,
            )?
        } else {
            convert_math_result(
                mul_shr(amount_out as u128, price, SCALE_OFFSET, Rounding::Up),
                ErrorCode::MulShrMathError,
            )?
        };

        let fee_amount = get_fee_for_amount(amount_in_without_fee, fee)?;

        let amount_in = amount_in_without_fee
            .checked_add(fee_amount)
            .ok_or(ErrorCode::AmountOverflow)?;

        let protocol_fee_amount = get_protocol_fee(fee_amount, protocol_share);

        if swap_for_y {
            self.reserve_x = self
                .reserve_x
                .checked_add(amount_in)
                .ok_or(ErrorCode::AmountOverflow)?
                .checked_sub(protocol_fee_amount)
                .ok_or(ErrorCode::AmountUnderflow)?;

            self.reserve_y = self
                .reserve_y
                .checked_sub(amount_out)
                .ok_or(ErrorCode::AmountUnderflow)?;
        } else {
            self.reserve_x = self
                .reserve_x
                .checked_sub(amount_out)
                .ok_or(ErrorCode::AmountUnderflow)?;

            self.reserve_y = self
                .reserve_y
                .checked_add(amount_in)
                .ok_or(ErrorCode::AmountOverflow)?
                .checked_sub(protocol_fee_amount)
                .ok_or(ErrorCode::AmountUnderflow)?;
        }

        Ok((amount_in, amount_out, fee_amount, protocol_fee_amount))
    }
}
