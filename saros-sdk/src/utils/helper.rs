use solana_sdk::pubkey::Pubkey;

use crate::constants::{HOOK_CONFIG, HOOK_PROGRAM_ID};

pub fn find_event_authority(program_id: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &program_id).0
}

pub fn get_bin_array_lower(
    bin_array_index: u32,
    pair: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let (bin_array_lower_pubkey, bump) = Pubkey::find_program_address(
        &[
            b"bin_array".as_ref(),
            pair.as_ref(),
            bin_array_index.to_le_bytes().as_ref(),
        ],
        program_id,
    );
    (bin_array_lower_pubkey, bump)
}

pub fn get_bin_array_upper(
    bin_array_index: u32,
    pair: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let (bin_array_upper_pubkey, bump) = Pubkey::find_program_address(
        &[
            b"bin_array".as_ref(),
            pair.as_ref(),
            (bin_array_index + 1).to_le_bytes().as_ref(),
        ],
        program_id,
    );

    (bin_array_upper_pubkey, bump)
}

pub fn get_hook_bin_array(bin_array_index: u32, pair: &Pubkey) -> (Pubkey, Pubkey) {
    let hook = Pubkey::find_program_address(
        &[b"hook".as_ref(), HOOK_CONFIG.as_ref(), pair.as_ref()],
        &HOOK_PROGRAM_ID,
    )
    .0;

    let (hook_bin_array_lower, _) = Pubkey::find_program_address(
        &[
            b"bin_array".as_ref(),
            hook.as_ref(),
            (bin_array_index).to_le_bytes().as_ref(),
        ],
        &HOOK_PROGRAM_ID,
    );
    let (hook_bin_array_upper, _) = Pubkey::find_program_address(
        &[
            b"bin_array".as_ref(),
            hook.as_ref(),
            (bin_array_index + 1).to_le_bytes().as_ref(),
        ],
        &HOOK_PROGRAM_ID,
    );

    (hook_bin_array_lower, hook_bin_array_upper)
}

pub fn is_swap_for_y(source_mint: Pubkey, token_x: Pubkey) -> bool {
    source_mint == token_x
}
