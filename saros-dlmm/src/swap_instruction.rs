use itertools::Itertools;

use anchor_lang::{InstructionData, prelude::*};
use anyhow::{Ok, Result};
use solana_sdk::instruction::Instruction;

use jupiter_amm_interface::SwapMode;

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
    pub swap_mode: SwapMode,
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
        SwapMode::ExactIn => liquidity_book::instruction::Swap {
            _amount: amount,
            _other_amount_threshold: other_amount_threshold,
            _swap_for_y: swap_for_y,
            _swap_type: liquidity_book::SwapType::ExactInput,
        }
        .data(),
        SwapMode::ExactOut => liquidity_book::instruction::Swap {
            _amount: amount,
            _other_amount_threshold: other_amount_threshold,
            _swap_for_y: swap_for_y,
            _swap_type: liquidity_book::SwapType::ExactOutput,
        }
        .data(),
    })
}
