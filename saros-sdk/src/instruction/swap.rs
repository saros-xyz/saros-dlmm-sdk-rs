use itertools::Itertools;

use anchor_lang::{prelude::*, InstructionData};
use anyhow::{Ok, Result};
use solana_sdk::instruction::Instruction;

use crate::math::swap_manager::SwapType;

use liquidity_book::liquidity_book::{
    client::args::Swap as SwapArgs, types::SwapType as LbSwapType,
};

/// All necessary parts to build a `VersionedTransaction`
#[derive(Clone)]
pub struct SwapInstructions {
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    pub token_ledger_instruction: Option<Instruction>,
    /// Instruction performing the action of swapping
    pub swap_instruction: Instruction,
    pub cleanup_instruction: Option<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
}

impl From<SwapInstructions> for Vec<Instruction> {
    fn from(
        SwapInstructions {
            compute_budget_instructions,
            setup_instructions,
            token_ledger_instruction: _,
            swap_instruction,
            cleanup_instruction,
            address_lookup_table_addresses: _,
        }: SwapInstructions,
    ) -> Vec<Instruction> {
        // We don't use `token_ledger_instruction` to build the transaction. `token_ledger_instruction` is
        // only available in instructions mode.
        compute_budget_instructions
            .into_iter()
            .chain(setup_instructions)
            .chain([swap_instruction])
            .chain(cleanup_instruction)
            .collect_vec()
    }
}

pub struct BuildSwapInstructionDataParams {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub swap_for_y: bool,
    pub swap_mode: SwapType,
}

pub fn build_swap_instruction_data(
    BuildSwapInstructionDataParams {
        amount,
        other_amount_threshold,
        swap_for_y,
        swap_mode,
    }: BuildSwapInstructionDataParams,
) -> Result<Vec<u8>> {
    Ok(match swap_mode {
        SwapType::ExactIn => SwapArgs {
            amount,
            other_amount_threshold,
            swap_for_y,
            swap_type: LbSwapType::ExactInput,
        }
        .data(),
        SwapType::ExactOut => SwapArgs {
            amount,
            other_amount_threshold,
            swap_for_y,
            swap_type: LbSwapType::ExactOutput,
        }
        .data(),
    })
}
