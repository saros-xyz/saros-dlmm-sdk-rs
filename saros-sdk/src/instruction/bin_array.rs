use anchor_lang::{prelude::AccountMeta, system_program, InstructionData};
use anyhow::{Ok, Result};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::utils::helper::find_event_authority;

pub fn build_initialize_bin_array_data(id: u32) -> Result<Vec<u8>> {
    Ok(liquidity_book::instruction::InitializeBinArray { _id: id }.data())
}

pub fn get_initialize_bin_array_instruction(
    pair: Pubkey,
    bin_array_index: u32,
    payer: Pubkey,
    bin_array_account: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(pair, false),
        AccountMeta::new(bin_array_account, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(find_event_authority(liquidity_book::ID), false),
        AccountMeta::new_readonly(liquidity_book::ID, false),
    ];

    let data = build_initialize_bin_array_data(bin_array_index).unwrap();

    Instruction {
        program_id: liquidity_book::ID,
        accounts,
        data,
    }
}
