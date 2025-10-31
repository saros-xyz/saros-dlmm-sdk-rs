use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use jupiter_amm_interface::Amm;
use saros_sdk::instruction::{CreatePositionParams, ModifierPositionParams};

pub trait SarosPositionManagement: Amm {
    fn get_create_position_account_metas(
        &self,
        create_position_params: CreatePositionParams,
    ) -> Result<Vec<AccountMeta>>;

    fn get_modifier_position_account_metas(
        &self,
        modifier_position_params: ModifierPositionParams,
    ) -> Result<Vec<AccountMeta>>;
}
