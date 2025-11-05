use anchor_lang::{prelude::AccountMeta, system_program, InstructionData};
use liquidity_book::liquidity_book::client::args::InitializeBinArray as InitializeBinArrayArgs;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::{
    constants::{HOOK_PROGRAM_ID, LIQUIDITY_BOOK_PROGRAM_ID},
    utils::helper::find_event_authority,
};

pub fn get_initialize_bin_array_instruction(
    pair: Pubkey,
    bin_array_index: u32,
    payer: Pubkey,
    bin_array_account: Pubkey,
) -> Instruction {
    let event_authority = find_event_authority(LIQUIDITY_BOOK_PROGRAM_ID);

    let accounts = vec![
        AccountMeta::new_readonly(pair, false),
        AccountMeta::new(bin_array_account, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(LIQUIDITY_BOOK_PROGRAM_ID, false),
    ];

    Instruction {
        program_id: LIQUIDITY_BOOK_PROGRAM_ID,
        accounts,
        data: InitializeBinArrayArgs {
            id: bin_array_index,
        }
        .data(),
    }
}

pub fn get_initialize_hook_bin_array_instruction(
    hook: Pubkey,
    bin_array_index: u32,
    payer: Pubkey,
    bin_array_account: Pubkey,
) -> Instruction {
    let event_authority = find_event_authority(HOOK_PROGRAM_ID);

    let accounts = vec![
        AccountMeta::new_readonly(hook, false),
        AccountMeta::new(bin_array_account, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(HOOK_PROGRAM_ID, false),
    ];

    Instruction {
        program_id: HOOK_PROGRAM_ID,
        accounts,
        data: InitializeBinArrayArgs {
            id: bin_array_index,
        }
        .data(),
    }
}
