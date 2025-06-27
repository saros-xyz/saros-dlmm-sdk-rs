use crate::{
    errors::ErrorCode,
    state::{bin_array::BinArrayPair, pair::Pair},
};
use anyhow::Result;
use jupiter_amm_interface::SwapMode;

pub fn get_swap_result(
    pair: &mut Pair,
    bin_array: &mut BinArrayPair,
    amount: u64,
    swap_for_y: bool,
    swap_type: SwapMode,
    block_timestamp: u64,
) -> Result<u64> {
    pair.update_references(block_timestamp)?;

    match swap_type {
        SwapMode::ExactIn => {
            let mut amount_in_left: u64 = amount;
            let mut amount_out: u64 = 0;
            let mut total_protocol_fee: u64 = 0;

            while amount_in_left > 0 {
                pair.update_volatility_accumulator()?;

                let bin = bin_array.get_bin_mut(pair.active_id)?;

                let fee = pair.get_total_fee()?;

                let (amount_in_with_fees, amount_out_of_bin, fee_amount, protocol_fee) = bin
                    .swap_exact_in(
                        pair.bin_step,
                        pair.active_id,
                        amount_in_left,
                        fee,
                        pair.get_protocol_share(),
                        swap_for_y,
                    )?;

                amount_out = amount_out
                    .checked_add(amount_out_of_bin)
                    .ok_or(ErrorCode::AmountOverflow)?;

                amount_in_left = amount_in_left
                    .checked_sub(amount_in_with_fees)
                    .ok_or(ErrorCode::AmountUnderflow)?;

                total_protocol_fee = total_protocol_fee
                    .checked_add(protocol_fee)
                    .ok_or(ErrorCode::AmountOverflow)?;

                if amount_in_left == 0 {
                    break;
                } else {
                    pair.move_active_id(swap_for_y)?;
                }
            }

            if swap_for_y {
                pair.protocol_fees_x = pair
                    .protocol_fees_x
                    .checked_add(total_protocol_fee)
                    .ok_or(ErrorCode::AmountOverflow)?;
            } else {
                pair.protocol_fees_y = pair
                    .protocol_fees_y
                    .checked_add(total_protocol_fee)
                    .ok_or(ErrorCode::AmountOverflow)?;
            }

            Ok(amount_out)
        }

        SwapMode::ExactOut => {
            let mut amount_out_left: u64 = amount;
            let mut amount_in: u64 = 0;
            let mut total_protocol_fee: u64 = 0;

            while amount_out_left > 0 {
                pair.update_volatility_accumulator()?;

                let bin = bin_array.get_bin_mut(pair.active_id)?;

                let fee = pair.get_total_fee()?;

                let (amount_in_with_fees, amount_out_of_bin, fee_amount, protocol_fee) = bin
                    .swap_exact_out(
                        pair.bin_step,
                        pair.active_id,
                        amount_out_left,
                        fee,
                        pair.get_protocol_share(),
                        swap_for_y,
                    )?;

                amount_in = amount_in
                    .checked_add(amount_in_with_fees)
                    .ok_or(ErrorCode::AmountOverflow)?;

                amount_out_left = amount_out_left
                    .checked_sub(amount_out_of_bin)
                    .ok_or(ErrorCode::AmountUnderflow)?;

                total_protocol_fee = total_protocol_fee
                    .checked_add(protocol_fee)
                    .ok_or(ErrorCode::AmountOverflow)?;

                if amount_out_left == 0 {
                    break;
                } else {
                    pair.move_active_id(swap_for_y)?;
                }
            }

            if swap_for_y {
                pair.protocol_fees_x = pair
                    .protocol_fees_x
                    .checked_add(total_protocol_fee)
                    .ok_or(ErrorCode::AmountOverflow)?;
            } else {
                pair.protocol_fees_y = pair
                    .protocol_fees_y
                    .checked_add(total_protocol_fee)
                    .ok_or(ErrorCode::AmountOverflow)?;
            }

            Ok(amount_in)
        }
    }
}
