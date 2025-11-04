use anyhow::Result;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

const HOOK_DISCRIMINATOR: [u8; 8] = [125, 61, 76, 173, 200, 161, 92, 217];
#[derive(Clone, Debug)]
pub struct Hook {
    _discriminator: [u8; 8],
    pub bump: [u8; 1],
    pub authority: Pubkey,
    pub pair: Pubkey,
    pub reward_token_mint: Pubkey,
    pub hook_reserve: Pubkey,

    pub rewards_per_second: u64,
    pub end_time: i64,
    pub last_update: i64,

    pub delta_bin_a: i32,
    pub delta_bin_b: i32,

    pub total_unclaimed_rewards: u64,
}

impl Default for Hook {
    fn default() -> Self {
        Self {
            _discriminator: HOOK_DISCRIMINATOR,
            bump: [0u8; 1],
            authority: Pubkey::default(),
            pair: Pubkey::default(),
            reward_token_mint: Pubkey::default(),
            hook_reserve: Pubkey::default(),
            rewards_per_second: 0,
            end_time: 0,
            last_update: 0,
            delta_bin_a: 0,
            delta_bin_b: 0,
            total_unclaimed_rewards: 0,
        }
    }
}

impl IsInitialized for Hook {
    fn is_initialized(&self) -> bool {
        self.pair != Pubkey::default()
    }
}

impl Sealed for Hook {}

impl Pack for Hook {
    const LEN: usize = 8 + 1 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 4 + 4 + 8;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Hook::LEN];

        let (
            discriminator_dst,
            bump_dst,
            authority_dst,
            pair_dst,
            reward_token_mint_dst,
            hook_reserve_dst,
            rewards_per_second_dst,
            end_time_dst,
            last_update_dst,
            delta_bin_a_dst,
            delta_bin_b_dst,
            total_unclaimed_rewards_dst,
        ) = mut_array_refs![output, 8, 1, 32, 32, 32, 32, 8, 8, 8, 4, 4, 8];

        println!(
            "Packing Hook struct into slice... {:?}",
            discriminator_dst.as_ref()
        );

        discriminator_dst.copy_from_slice(&HOOK_DISCRIMINATOR);
        bump_dst.copy_from_slice(&self.bump);
        authority_dst.copy_from_slice(self.authority.as_ref());
        pair_dst.copy_from_slice(self.pair.as_ref());
        reward_token_mint_dst.copy_from_slice(self.reward_token_mint.as_ref());
        hook_reserve_dst.copy_from_slice(self.hook_reserve.as_ref());
        *rewards_per_second_dst = self.rewards_per_second.to_le_bytes();
        *end_time_dst = self.end_time.to_le_bytes();
        *last_update_dst = self.last_update.to_le_bytes();
        *delta_bin_a_dst = self.delta_bin_a.to_le_bytes();
        *delta_bin_b_dst = self.delta_bin_b.to_le_bytes();
        *total_unclaimed_rewards_dst = self.total_unclaimed_rewards.to_le_bytes();
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Hook::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            discriminator_src,
            bump_src,
            authority_src,
            pair_src,
            reward_token_mint_src,
            hook_reserve_src,
            rewards_per_second_src,
            end_time_src,
            last_update_src,
            delta_bin_a_src,
            delta_bin_b_src,
            total_unclaimed_rewards_src,
        ) = array_refs![input, 8, 1, 32, 32, 32, 32, 8, 8, 8, 4, 4, 8];
        Ok(Self {
            _discriminator: *discriminator_src,
            bump: *bump_src,
            authority: Pubkey::new_from_array(*authority_src),
            pair: Pubkey::new_from_array(*pair_src),
            reward_token_mint: Pubkey::new_from_array(*reward_token_mint_src),
            hook_reserve: Pubkey::new_from_array(*hook_reserve_src),
            rewards_per_second: u64::from_le_bytes(*rewards_per_second_src),
            end_time: i64::from_le_bytes(*end_time_src),
            last_update: i64::from_le_bytes(*last_update_src),
            delta_bin_a: i32::from_le_bytes(*delta_bin_a_src),
            delta_bin_b: i32::from_le_bytes(*delta_bin_b_src),
            total_unclaimed_rewards: u64::from_le_bytes(*total_unclaimed_rewards_src),
        })
    }
}
