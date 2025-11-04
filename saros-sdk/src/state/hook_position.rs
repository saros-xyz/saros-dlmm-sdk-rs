use anyhow::Result;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use crate::constants::MAX_BIN_PER_POSITION;

const HOOK_POSITION_DISCRIMINATOR: [u8; 8] = [125, 149, 132, 62, 52, 71, 211, 143];

#[derive(Clone, Debug)]
pub struct HookPosition {
    _discriminator: [u8; 8],
    pub user_accrued_rewards_per_share: [u128; MAX_BIN_PER_POSITION as usize],
    pub pending_rewards: u64,
    pub bump: u8,
    pub user: Pubkey,
    _space: [u8; 7],
}

impl HookPosition {
    pub fn default() -> Self {
        Self {
            _discriminator: HOOK_POSITION_DISCRIMINATOR,
            user_accrued_rewards_per_share: [0u128; MAX_BIN_PER_POSITION as usize],
            pending_rewards: 0,
            bump: 0,
            user: Pubkey::default(),
            _space: [0u8; 7],
        }
    }
}

impl IsInitialized for HookPosition {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Sealed for HookPosition {}

impl Pack for HookPosition {
    const LEN: usize = 8 + 16 * MAX_BIN_PER_POSITION as usize + 8 + 1 + 32 + 7;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, HookPosition::LEN];

        let (
            discriminator_dst,
            user_accrued_rewards_per_share_dst,
            pending_rewards_dst,
            bump_dst,
            user_dst,
            _space_dst,
        ) = mut_array_refs![output, 8, 16 * MAX_BIN_PER_POSITION as usize, 8, 1, 32, 7];

        discriminator_dst.copy_from_slice(&HOOK_POSITION_DISCRIMINATOR);
        for i in 0..MAX_BIN_PER_POSITION as usize {
            let start = i * 16;
            let end = start + 16;
            user_accrued_rewards_per_share_dst[start..end]
                .copy_from_slice(&self.user_accrued_rewards_per_share[i].to_le_bytes());
        }
        pending_rewards_dst.copy_from_slice(&self.pending_rewards.to_le_bytes());
        bump_dst[0] = self.bump;
        user_dst.copy_from_slice(self.user.as_ref());
        _space_dst.copy_from_slice(&self._space);
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, HookPosition::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            discriminator_src,
            user_accrued_rewards_per_share_src,
            pending_rewards_src,
            bump_src,
            user_src,
            _space_src,
        ) = array_refs![input, 8, 16 * MAX_BIN_PER_POSITION as usize, 8, 1, 32, 7];

        let mut user_accrued_rewards_per_share = [0u128; MAX_BIN_PER_POSITION as usize];
        for i in 0..MAX_BIN_PER_POSITION as usize {
            let start = i * 16;
            let end = start + 16;
            user_accrued_rewards_per_share[i] = u128::from_le_bytes(
                user_accrued_rewards_per_share_src[start..end]
                    .try_into()
                    .unwrap(),
            );
        }

        Ok(Self {
            _discriminator: *discriminator_src,
            user_accrued_rewards_per_share,
            pending_rewards: u64::from_le_bytes(*pending_rewards_src),
            bump: bump_src[0],
            user: Pubkey::new_from_array(*user_src),
            _space: [0; 7],
        })
    }
}
