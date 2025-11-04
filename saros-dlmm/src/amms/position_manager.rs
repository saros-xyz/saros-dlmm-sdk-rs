use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use jupiter_amm_interface::Amm;
use saros_sdk::{
    instruction::{ClaimParams, CreatePositionParams, ModifierPositionParams},
    state::hook::Hook,
};
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug, Default)]
pub struct HookInfo {
    pub reward_token_program: Pubkey,
    pub hook: Hook,
}

pub trait SarosPositionManagement: Amm {
    fn has_hook(&self) -> bool;
    fn get_hook(&self) -> Option<Pubkey>;
    fn get_create_position_account_metas(
        &self,
        create_position_params: CreatePositionParams,
    ) -> Result<Vec<AccountMeta>>;

    fn get_modifier_position_account_metas(
        &self,
        modifier_position_params: ModifierPositionParams,
    ) -> Result<Vec<AccountMeta>>;

    fn get_claim_account_metas(
        &self,
        claim_params: ClaimParams,
    ) -> Result<Vec<AccountMeta>>;
}
