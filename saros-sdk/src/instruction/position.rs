use anchor_lang::InstructionData;
use anyhow::{Ok, Result};
pub struct CreatePositionParams {
    pub relative_bin_id_left: i32,
    pub relative_bin_id_right: i32,
}

pub fn build_create_position_instruction_data(
    CreatePositionParams {
        relative_bin_id_left,
        relative_bin_id_right,
    }: CreatePositionParams,
) -> Result<Vec<u8>> {
    Ok(liquidity_book::instruction::CreatePosition {
        _relative_bin_id_left: relative_bin_id_left,
        _relative_bin_in_right: relative_bin_id_right,
    }
    .data())
}

pub struct DecreasePositionParams {
    pub shares: Vec<u128>,
}

pub fn build_decrease_position_instruction_data(
    DecreasePositionParams { shares }: DecreasePositionParams,
) -> Result<Vec<u8>> {
    Ok(liquidity_book::instruction::DecreasePosition { _shares: shares }.data())
}

pub struct LiquidityDistribution {
    pub relative_bin_id: i32,
    pub distribution_x: u16,
    pub distribution_y: u16,
}

impl LiquidityDistribution {
    pub fn into_bin_liquidity(self) -> liquidity_book::BinLiquidityDistribution {
        liquidity_book::BinLiquidityDistribution {
            relative_bin_id: self.relative_bin_id,
            distribution_x: self.distribution_x,
            distribution_y: self.distribution_y,
        }
    }
}

pub struct IncreasePositionParams {
    pub amount_x: u64,
    pub amount_y: u64,
    pub liquidity_distribution: Vec<LiquidityDistribution>,
}

pub fn build_increase_position_instruction_data(
    IncreasePositionParams {
        amount_x,
        amount_y,
        liquidity_distribution,
    }: IncreasePositionParams,
) -> Result<Vec<u8>> {
    Ok(liquidity_book::instruction::IncreasePosition {
        _amount_x: amount_x,
        _amount_y: amount_y,
        _liquidity_distribution: liquidity_distribution
            .into_iter()
            .map(|ld| ld.into_bin_liquidity())
            .collect(),
    }
    .data())
}
