use solana_sdk::pubkey::Pubkey;

pub fn get_bin_array_lower(
    bin_array_index: u32,
    pair: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    let (bin_array_lower_pubkey, bump) = Pubkey::find_program_address(
        &[b"bin_array", pair.as_ref(), &bin_array_index.to_le_bytes()],
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
            b"bin_array",
            pair.as_ref(),
            &(bin_array_index + 1).to_le_bytes(),
        ],
        program_id,
    );

    (bin_array_upper_pubkey, bump)
}
