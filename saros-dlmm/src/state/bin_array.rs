use anchor_lang::prelude::ProgramError;
use anyhow::Result;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const BIN_ARRAY_SIZE: u32 = 256;
pub const BIN_ARRAY_SIZE_USIZE: usize = 256;

use crate::{errors::ErrorCode, state::bin::Bin};

#[derive(Clone, Copy)]
pub struct BinArray {
    pub pair: Pubkey,
    pub bins: [Bin; BIN_ARRAY_SIZE_USIZE],
    pub index: u32,
    pub _space: [u8; 12],
}

impl BinArray {
    pub fn initialize(&mut self, pair: Pubkey, index: u32) {
        self.pair = pair;
        self.index = index;
    }

    pub fn get_index_from_bin_id(bin_id: u32) -> u32 {
        bin_id / BIN_ARRAY_SIZE
    }

    pub fn contains(&self, bin_id: u32) -> bool {
        bin_id / BIN_ARRAY_SIZE == self.index
    }

    pub fn get_bin(&self, bin_id: u32) -> Result<&Bin> {
        if !self.contains(bin_id) {
            return Err(ErrorCode::BinNotFound.into());
        }

        Ok(&self.bins[(bin_id % BIN_ARRAY_SIZE) as usize])
    }

    pub fn get_bin_mut(&mut self, bin_id: u32) -> Result<&mut Bin> {
        if !self.contains(bin_id) {
            return Err(ErrorCode::BinNotFound.into());
        }

        Ok(&mut self.bins[(bin_id % BIN_ARRAY_SIZE) as usize])
    }
}

impl Default for BinArray {
    fn default() -> Self {
        Self {
            pair: Pubkey::default(),
            bins: [Bin::default(); BIN_ARRAY_SIZE_USIZE],
            index: 0,
            _space: [0; 12],
        }
    }
}

impl IsInitialized for BinArray {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Sealed for BinArray {}

impl Pack for BinArray {
    const LEN: usize = 32 + BIN_ARRAY_SIZE_USIZE * 32 + 4 + 12;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 8240];
        let (pair_dst, bins_dst, index_dst, _space_dst) = mut_array_refs![output, 32, 8192, 4, 12];

        pair_dst.copy_from_slice(self.pair.as_ref());
        for (i, bin) in self.bins.iter().enumerate() {
            bin.pack_into_slice(&mut bins_dst[i * Bin::LEN..]);
        }
        index_dst.copy_from_slice(&self.index.to_le_bytes());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, 8240];
        let (pair_src, bins_src, index_src, _space_src) =
            array_refs![input, 32, BIN_ARRAY_SIZE_USIZE * 32, 4, 12];

        let mut bins = [Bin::default(); BIN_ARRAY_SIZE_USIZE];
        for (i, bin) in bins.iter_mut().enumerate() {
            *bin = Bin::unpack_from_slice(&bins_src[i * Bin::LEN..])?;
        }

        Ok(Self {
            pair: Pubkey::new_from_array(*pair_src),
            bins,
            index: u32::from_le_bytes(*index_src),
            _space: [0; 12],
        })
    }
}

pub struct BinArrayPair {
    pub bin_array_lower: BinArray,
    pub bin_array_upper: BinArray,
}

impl BinArrayPair {
    pub fn merge(bin_array_lower: BinArray, bin_array_upper: BinArray) -> Result<Self> {
        if bin_array_upper.index != bin_array_lower.index + 1 {
            return Err(ErrorCode::BinArrayIndexMismatch.into());
        }
        Ok(Self {
            bin_array_lower,
            bin_array_upper,
        })
    }

    pub fn get_bin(&self, bin_id: u32) -> Result<&Bin> {
        self.bin_array_lower
            .get_bin(bin_id)
            .or_else(|_| self.bin_array_upper.get_bin(bin_id))
    }

    pub fn get_bin_mut(&mut self, bin_id: u32) -> Result<&mut Bin> {
        self.bin_array_lower
            .get_bin_mut(bin_id)
            .or_else(|_| self.bin_array_upper.get_bin_mut(bin_id))
    }
}
