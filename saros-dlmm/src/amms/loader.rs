use std::collections::HashSet;

use anyhow::{Result, anyhow};
use jupiter_amm_interface::{Amm, AmmContext, KeyedAccount};
use lazy_static::lazy_static;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

use crate::SarosDlmm;

mod spl_token_swap_programs {

    use super::*;
    pub const SAROS_DLMM: Pubkey = saros::ID;
}

lazy_static! {
    pub static ref SPL_TOKEN_SWAP_PROGRAMS: HashMap<Pubkey, String> = {
        let mut m = HashMap::new();
        m.insert(spl_token_swap_programs::SAROS_DLMM, "Saros DLMM".into());

        m
    };
}

pub fn amm_factory(
    keyed_account: &KeyedAccount,
    amm_context: &AmmContext,
    _saber_wrapper_mints: &mut HashSet<Pubkey>,
) -> Result<Box<dyn Amm + Send + Sync>> {
    let owner = keyed_account.account.owner;

    let saros_dlmm = SarosDlmm::from_keyed_account(keyed_account, amm_context)?;

    // Add your AMM here
    if SPL_TOKEN_SWAP_PROGRAMS.contains_key(&owner) {
        Ok(Box::new(saros_dlmm))
    } else {
        Err(anyhow!(
            "Unsupported pool {}, from owner {}",
            keyed_account.key,
            keyed_account.account.owner
        ))
    }
}
