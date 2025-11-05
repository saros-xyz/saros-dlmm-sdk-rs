use anchor_lang::{prelude::AccountMeta, system_program, InstructionData};
use rewarder_hook::rewarder_hook::client::args::InitializePosition as InitializePositionArgs;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::{
    constants::HOOK_PROGRAM_ID,
    utils::helper::{find_event_authority, find_hook_position},
};

pub fn get_initialize_hook_position_instruction(
    hook: Pubkey,
    lb_position: Pubkey,
    payer: Pubkey,
) -> Instruction {
    let hook_position = find_hook_position(lb_position, hook);

    let event_authority = find_event_authority(HOOK_PROGRAM_ID);

    let accounts = vec![
        AccountMeta::new_readonly(hook, false),
        AccountMeta::new(lb_position, false),
        AccountMeta::new(hook_position, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(HOOK_PROGRAM_ID, false),
    ];

    Instruction {
        program_id: HOOK_PROGRAM_ID,
        accounts,
        data: InitializePositionArgs {}.data(),
    }
}
