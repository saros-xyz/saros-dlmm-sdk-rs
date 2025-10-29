use anyhow::Result;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::{constants::MAX_BIN_PER_POSITION, errors::ErrorCode};

const POSITION_DISCRIMINATOR: [u8; 8] = [170, 188, 143, 228, 122, 64, 247, 208];

#[derive(Clone, Debug)]
pub struct Position {
    _discriminator: [u8; 8],
    pub pair: Pubkey,
    pub position_mint: Pubkey,
    pub liquidity_shares: [u128; MAX_BIN_PER_POSITION as usize],
    pub lower_bin_id: u32,
    pub upper_bin_id: u32,
    _space: [u8; 8],
}

impl Default for Position {
    fn default() -> Self {
        Self {
            _discriminator: POSITION_DISCRIMINATOR,
            pair: Pubkey::default(),
            position_mint: Pubkey::default(),
            liquidity_shares: [0u128; MAX_BIN_PER_POSITION as usize],
            lower_bin_id: 0,
            upper_bin_id: 0,
            _space: [0u8; 8],
        }
    }
}

impl IsInitialized for Position {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Sealed for Position {}

impl Pack for Position {
    const LEN: usize = 8 + 32 + 32 + 16 * MAX_BIN_PER_POSITION as usize + 4 + 4 + 8;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Position::LEN];

        let (
            discriminator_dst,
            pair_dst,
            position_mint_dst,
            liquidity_shares_dst,
            lower_bin_id_dst,
            upper_bin_id_dst,
            _space_dst,
        ) = mut_array_refs![
            output,
            8,
            32,
            32,
            16 * MAX_BIN_PER_POSITION as usize,
            4,
            4,
            8
        ];

        println!("packing discriminator_dst: {:?}", discriminator_dst);

        discriminator_dst.copy_from_slice(&POSITION_DISCRIMINATOR);
        pair_dst.copy_from_slice(self.pair.as_ref());
        position_mint_dst.copy_from_slice(self.position_mint.as_ref());
        for i in 0..MAX_BIN_PER_POSITION as usize {
            let start = i * 16;
            let end = start + 16;
            liquidity_shares_dst[start..end]
                .copy_from_slice(&self.liquidity_shares[i].to_le_bytes());
        }
        lower_bin_id_dst.copy_from_slice(&self.lower_bin_id.to_le_bytes());
        upper_bin_id_dst.copy_from_slice(&self.upper_bin_id.to_le_bytes());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Position::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            discriminator_src,
            pair_src,
            position_mint_src,
            liquidity_shares_src,
            lower_bin_id_src,
            upper_bin_id_src,
            _space_src,
        ) = array_refs![
            input,
            8,
            32,
            32,
            16 * MAX_BIN_PER_POSITION as usize,
            4,
            4,
            8
        ];

        let mut liquidity_shares = [0u128; MAX_BIN_PER_POSITION as usize];
        for i in 0..MAX_BIN_PER_POSITION as usize {
            let start = i * 16;
            let end = start + 16;
            liquidity_shares[i] = u128::from_le_bytes(
                liquidity_shares_src[start..end]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            );
        }

        Ok(Position {
            _discriminator: *discriminator_src,
            pair: Pubkey::new_from_array(*pair_src),
            position_mint: Pubkey::new_from_array(*position_mint_src),
            liquidity_shares,
            lower_bin_id: u32::from_le_bytes(*lower_bin_id_src),
            upper_bin_id: u32::from_le_bytes(*upper_bin_id_src),
            _space: [_space_src[0]; 8],
        })
    }
}

impl Position {
    pub fn get_share_mut(&mut self, bin_id: u32) -> Result<&mut u128> {
        if bin_id < self.lower_bin_id || bin_id > self.upper_bin_id {
            return Err(ErrorCode::BinNotFound)?;
        }

        Ok(&mut self.liquidity_shares[(bin_id - self.lower_bin_id) as usize])
    }
}
