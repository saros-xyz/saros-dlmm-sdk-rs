use itertools::Itertools;

use anchor_lang::{InstructionData, prelude::*};
use anyhow::{Ok, Result};
use solana_sdk::instruction::Instruction;

use jupiter_amm_interface::SwapMode;
use saros::{self};

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
    amount: u64,
    other_amount_threshold: u64,
    swap_for_y: bool,
    swap_mode: SwapMode,
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
        SwapMode::ExactIn => saros::instruction::Swap {
            _amount: amount,
            _other_amount_threshold: other_amount_threshold,
            _swap_for_y: swap_for_y,
            _swap_type: saros::SwapType::ExactInput,
        }
        .data(),
        SwapMode::ExactOut => saros::instruction::Swap {
            _amount: amount,
            _other_amount_threshold: other_amount_threshold,
            _swap_for_y: swap_for_y,
            _swap_type: saros::SwapType::ExactOutput,
        }
        .data(),
    })
}

pub struct BuildSwapAccountsParams<'a> {
    pub swap_program_id: &'a Pubkey,
    pub source_token_account: &'a Pubkey,
    pub destination_token_account: &'a Pubkey,
    pub pair_account: &'a Pubkey,
    pub bin_array_lower: &'a Pubkey,
    pub bin_array_upper: &'a Pubkey,
    pub token_vault_x: &'a Pubkey,
    pub token_vault_y: &'a Pubkey,
    pub user_vault_x: &'a Pubkey,
    pub user_vault_y: &'a Pubkey,
    pub user: &'a Pubkey,
    pub token_program_x: &'a Pubkey,
    pub token_program_y: &'a Pubkey,
    pub memo_program: &'a Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
}

pub fn build_swap_accounts(
    BuildSwapAccountsParams {
        swap_program_id,
        source_token_account,
        destination_token_account,
        pair_account,
        bin_array_lower,
        bin_array_upper,
        token_vault_x,
        token_vault_y,
        user_vault_x,
        user_vault_y,
        user,
        token_program_x,
        token_program_y,
        memo_program,
        event_authority: _,
        program: _,
    }: BuildSwapAccountsParams<'_>,
) -> Vec<AccountMeta> {
    let (event_authority, _) =
        Pubkey::find_program_address(&[b"__event_authority"], swap_program_id);
    saros::accounts::Swap {
        pair: *pair_account,
        token_mint_x: *source_token_account,
        token_mint_y: *destination_token_account,
        bin_array_lower: *bin_array_lower,
        bin_array_upper: *bin_array_upper,
        token_vault_x: *token_vault_x,
        token_vault_y: *token_vault_y,
        user_vault_x: *user_vault_x,
        user_vault_y: *user_vault_y,
        user: *user,
        token_program_x: *token_program_x,
        token_program_y: *token_program_y,
        memo_program: *memo_program,
        event_authority, // This is not used in the swap instruction
        program: *swap_program_id,
    }
    .to_account_metas(None)
}
