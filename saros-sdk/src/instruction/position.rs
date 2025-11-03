use anchor_lang::InstructionData;
use anyhow::{Ok, Result};
use solana_sdk::pubkey::Pubkey;

use liquidity_book::liquidity_book::{
    client::args::{
        ClosePosition as ClosePositionArgs, CreatePosition as CreatePositionArgs,
        DecreasePosition as DecreasePositionArgs, IncreasePosition as IncreasePositionArgs,
    },
    types::BinLiquidityDistribution,
};

use crate::constants::BASIS_POINT_MAX;

#[derive(Clone)]
pub struct ModifierPositionParams {
    pub user: Pubkey,
    pub position_key: Pubkey,
    pub position_token_account: Pubkey,
    pub position_mint: Pubkey,
    pub user_vault_x: Pubkey,
    pub user_vault_y: Pubkey,

    // Bin arrays at position
    pub bin_array_position_lower: Pubkey,
    pub bin_array_position_upper: Pubkey,

    // Remaining accounts of the LB program call, need to be checked
    pub position_hook_bin_array_lower: Pubkey,
    pub position_hook_bin_array_upper: Pubkey,
}
#[derive(Clone)]
pub struct CreatePositionParams {
    pub relative_bin_id_left: i32,
    pub relative_bin_id_right: i32,
    pub user: Pubkey,
    pub source_position: Pubkey,
    pub position_mint: Pubkey,
}

#[derive(Clone, Debug)]
pub struct IncreasePositionParams {
    pub amount_x: u64,
    pub amount_y: u64,
    pub liquidity_distribution: Vec<LiquidityDistribution>,
}

#[derive(Clone)]
pub struct DecreasePositionParams {
    pub shares: Vec<u128>,
}

#[derive(Clone, Debug)]
pub struct LiquidityDistribution {
    pub relative_bin_id: i32,
    pub distribution_x: u16,
    pub distribution_y: u16,
}

impl LiquidityDistribution {
    pub fn into_bin_liquidity(self) -> BinLiquidityDistribution {
        BinLiquidityDistribution {
            relative_bin_id: self.relative_bin_id,
            distribution_x: self.distribution_x,
            distribution_y: self.distribution_y,
        }
    }
}

pub fn build_create_position_instruction_data(
    CreatePositionParams {
        relative_bin_id_left,
        relative_bin_id_right,
        ..
    }: CreatePositionParams,
) -> Result<Vec<u8>> {
    Ok(CreatePositionArgs {
        relative_bin_id_left: relative_bin_id_left,
        relative_bin_in_right: relative_bin_id_right,
    }
    .data())
}

pub fn build_increase_position_instruction_data(
    IncreasePositionParams {
        amount_x,
        amount_y,
        liquidity_distribution,
        ..
    }: IncreasePositionParams,
) -> Result<Vec<u8>> {
    Ok(IncreasePositionArgs {
        amount_x,
        amount_y,
        liquidity_distribution: liquidity_distribution
            .into_iter()
            .map(|ld| ld.into_bin_liquidity())
            .collect(),
    }
    .data())
}

pub fn build_decrease_position_instruction_data(
    DecreasePositionParams { shares, .. }: DecreasePositionParams,
) -> Result<Vec<u8>> {
    Ok(DecreasePositionArgs { shares }.data())
}

pub fn build_close_position_instruction_data() -> Result<Vec<u8>> {
    Ok(ClosePositionArgs {}.data())
}

/// Create a uniform liquidity distribution
pub fn create_uniform_distribution(number_of_bins_each_side: u64) -> Vec<LiquidityDistribution> {
    let total_len = number_of_bins_each_side * 2 + 1;
    let step_value = BASIS_POINT_MAX / (number_of_bins_each_side + 1);

    (0..total_len)
        .map(|i| {
            let i_i32 = i as i32;
            let center = number_of_bins_each_side as i32;

            let distribution_x = if i_i32 < center { 0 } else { step_value as u16 };
            let distribution_y = if i_i32 > center { 0 } else { step_value as u16 };
            let relative_bin_id = i_i32 - center;

            LiquidityDistribution {
                relative_bin_id,
                distribution_x,
                distribution_y,
            }
        })
        .collect()
}
