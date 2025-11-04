use anchor_lang::InstructionData;
use anyhow::{Ok, Result};
use rewarder_hook::rewarder_hook::client::args::Claim as ClaimArgs;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct ClaimParams {
    pub lb_position: Pubkey,
    pub position_mint: Pubkey,
    pub user: Pubkey,
    pub position_hook_bin_array_lower: Pubkey,
    pub position_hook_bin_array_upper: Pubkey,
    pub user_reserve: Pubkey,
}

pub fn build_claim_instruction_data() -> Result<Vec<u8>> {
    Ok(ClaimArgs {}.data())
}
