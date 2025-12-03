pub mod amms;
pub mod route;

use crate::amms::position_manager::SarosPositionManagement;
pub use amms::amm;
use anchor_lang::prelude::AccountMeta;
use anyhow::{Context, Result};
use bincode::deserialize;
use jupiter_amm_interface::{
    try_get_account_data, try_get_account_data_and_owner, AccountMap, Amm, AmmContext,
    KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas, SwapMode, SwapParams,
};
use saros_sdk::utils::helper::{get_hook_bin_array, get_pair_bin_array, get_swap_pair_bin_array};
use saros_sdk::{
    instruction::{CreatePositionParams, ModifierPositionParams},
    math::{
        fees::{
            compute_transfer_amount_for_expected_output, compute_transfer_fee, TokenTransferFee,
        },
        swap_manager::{get_swap_result, SwapType},
    },
    state::{
        bin_array::{BinArray, BinArrayPair},
        pair::Pair,
    },
    utils::helper::{
        find_event_authority, find_hook_position, find_position, get_swap_hook_bin_array,
        is_swap_for_y,
    },
};
use solana_sdk::{
    program_pack::IsInitialized,
    program_pack::Pack,
    pubkey,
    pubkey::Pubkey,
    sysvar::{clock, clock::Clock},
};
use std::sync::{
    atomic::{AtomicI64, AtomicU64, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct SarosDlmm {
    pub program_id: Pubkey,
    pub key: Pubkey,
    pub label: String,
    pub pair: Pair,
    pub token_transfer_fee: TokenTransferFee,
    pub bin_array_lower: BinArray,
    pub bin_array_middle: BinArray,
    pub bin_array_upper: BinArray,
    pub bin_array_key: [Pubkey; 3],
    pub active_bin_array_key: [Pubkey; 2],
    pub token_vault: [Pubkey; 2],
    pub token_program: [Pubkey; 2],
    pub event_authority: Pubkey,
    pub hook: Pubkey,
    // // Remaining accounts of the LB program cpi call to hooks, will be checked at hook program.
    pub hook_bin_array_key: [Pubkey; 3],
    pub active_hook_bin_array_key: [Pubkey; 2],
    pub epoch: Arc<AtomicU64>,
    pub timestamp: Arc<AtomicI64>,
}

pub struct BinForSwap {
    pub bin_arrays: [BinArray; 2],
    pub bin_array_keys: [Pubkey; 2],
    pub hook_bin_array_keys: [Pubkey; 2],
}

impl SarosDlmm {
    pub const ASSOCIATED_TOKEN_PROGRAM_ADDRESS: Pubkey =
        pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

    pub fn compute_bin_array_swap(&self) -> Result<BinForSwap> {
        // unpack fixed order
        let [lower_key, middle_key, upper_key] = self.bin_array_key;
        let [hook_lower_key, hook_middle_key, hook_upper_key] = self.hook_bin_array_key;

        let lower_init = self.bin_array_lower.is_initialized();
        let middle_init = self.bin_array_middle.is_initialized();
        let upper_init = self.bin_array_upper.is_initialized();

        // case 1: all initialized → normal swap
        if lower_init && middle_init && upper_init {
            return Ok(BinForSwap {
                bin_arrays: [self.bin_array_middle.clone(), self.bin_array_upper.clone()],
                bin_array_keys: [middle_key, upper_key],
                hook_bin_array_keys: [hook_middle_key, hook_upper_key],
            });
        }
        // case 2: some bin not init → handle partial
        match (lower_init, middle_init, upper_init) {
            // lower + middle
            (true, true, false) => Ok(BinForSwap {
                bin_arrays: [self.bin_array_lower.clone(), self.bin_array_middle.clone()],
                bin_array_keys: [lower_key, middle_key],
                hook_bin_array_keys: [hook_lower_key, hook_middle_key],
            }),
            // middle + upper
            (false, true, true) => Ok(BinForSwap {
                bin_arrays: [self.bin_array_middle.clone(), self.bin_array_upper.clone()],
                bin_array_keys: [middle_key, upper_key],
                hook_bin_array_keys: [hook_middle_key, hook_upper_key],
            }),
            // middle not init
            (_, false, _) => Err(anyhow::anyhow!(
                "Require bin array is not initialized {middle_key}"
            )),
            // only 1 or 0 initialized
            _ => Err(anyhow::anyhow!(
                "Require at least 2 bin arrays to be initialized"
            )),
        }
    }
}

impl Amm for SarosDlmm {
    fn key(&self) -> Pubkey {
        self.key
    }

    fn label(&self) -> String {
        self.label.clone()
    }

    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn from_keyed_account(keyed_account: &KeyedAccount, amm_context: &AmmContext) -> Result<Self>
    where
        Self: Sized,
    {
        let account_data = &keyed_account.account.data[..];
        let pair = Pair::unpack(account_data)?;

        let bin_array_index = pair.bin_array_index();

        let (bin_array_lower_key, bin_array_middle_key, bin_array_upper_key) =
            get_swap_pair_bin_array(
                bin_array_index,
                &keyed_account.key,
                &keyed_account.account.owner,
            );

        let mut hook_key = keyed_account.key; // Dummy key if no hook
        let mut hook_bin_array_key = [Pubkey::default(); 3];

        if let Some(pair_hook_key) = pair.hook {
            let (hook_bin_array_lower_key, hook_bin_array_middle_key, hook_bin_array_upper_key) =
                get_swap_hook_bin_array(bin_array_index, pair_hook_key);
            hook_bin_array_key = [
                hook_bin_array_lower_key,
                hook_bin_array_middle_key,
                hook_bin_array_upper_key,
            ];
            hook_key = pair_hook_key;
        }

        let event_authority = find_event_authority(keyed_account.account.owner);

        Ok(Self {
            program_id: keyed_account.account.owner,
            key: keyed_account.key,
            label: "saros_dlmm".into(),
            pair: pair.clone(),
            token_transfer_fee: TokenTransferFee::default(),
            bin_array_lower: BinArray::default(),
            bin_array_middle: BinArray::default(),
            bin_array_upper: BinArray::default(),
            bin_array_key: [
                bin_array_lower_key,
                bin_array_middle_key,
                bin_array_upper_key,
            ],
            active_bin_array_key: [Pubkey::default(), Pubkey::default()],
            token_vault: [Pubkey::default(), Pubkey::default()],
            token_program: [Pubkey::default(), Pubkey::default()],
            event_authority,
            hook: hook_key,
            hook_bin_array_key,
            active_hook_bin_array_key: [Pubkey::default(), Pubkey::default()],
            epoch: amm_context.clock_ref.epoch.clone(),
            timestamp: amm_context.clock_ref.unix_timestamp.clone(),
        })
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.pair.token_mint_x, self.pair.token_mint_y]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![
            self.key,
            self.bin_array_key[0],
            self.bin_array_key[1],
            self.bin_array_key[2],
            self.pair.token_mint_x,
            self.pair.token_mint_y,
            clock::ID,
        ]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let pair_data = try_get_account_data(account_map, &self.key).with_context(|| {
            format!(
                "Pair account does not exist or has not been initialized: {}",
                self.key
            )
        })?;

        self.pair = Pair::unpack(pair_data)?;
        let bin_array_index = self.pair.bin_array_index();

        let (bin_array_lower_key, bin_array_middle_key, bin_array_upper_key) =
            get_swap_pair_bin_array(bin_array_index, &self.key, &self.program_id);

        if self.bin_array_key[0] != bin_array_lower_key
            || self.bin_array_key[1] != bin_array_middle_key
            || self.bin_array_key[2] != bin_array_upper_key
        {
            self.bin_array_key = [
                bin_array_lower_key,
                bin_array_middle_key,
                bin_array_upper_key,
            ];

            if let Some(hook_key) = self.pair.hook {
                let (hook_bin_array_lower_key, hook_bin_array_middle_key, hook_bin_array_upper_key) =
                    get_swap_hook_bin_array(bin_array_index, hook_key);

                self.hook_bin_array_key = [
                    hook_bin_array_lower_key,
                    hook_bin_array_middle_key,
                    hook_bin_array_upper_key,
                ];

                self.hook = hook_key;
            }
        } else {
            let _ = match try_get_account_data(&account_map, &bin_array_lower_key) {
                Ok(data) => {
                    self.bin_array_lower = BinArray::unpack(data)?;
                }
                Err(_) => {}
            };

            let _ = match try_get_account_data(&account_map, &bin_array_middle_key) {
                Ok(data) => {
                    self.bin_array_middle = BinArray::unpack(data)?;
                }
                Err(_) => {}
            };

            let _ = match try_get_account_data(&account_map, &bin_array_upper_key) {
                Ok(data) => {
                    self.bin_array_upper = BinArray::unpack(data)?;
                }
                Err(_) => {}
            };

            let active_bin_array_keys =
                get_pair_bin_array(bin_array_index, &self.key, &self.program_id);

            let active_hook_bin_array_keys = get_hook_bin_array(bin_array_index, self.hook);

            self.active_bin_array_key = [active_bin_array_keys.0, active_bin_array_keys.1];
            self.active_hook_bin_array_key =
                [active_hook_bin_array_keys.0, active_hook_bin_array_keys.1];
        }

        let (mint_x_data, mint_x_owner) =
            try_get_account_data_and_owner(account_map, &self.pair.token_mint_x).with_context(
                || {
                    format!(
                        "Token mint X not found or invalid: {}",
                        self.pair.token_mint_x
                    )
                },
            )?;
        let (mint_y_data, mint_y_owner) =
            try_get_account_data_and_owner(account_map, &self.pair.token_mint_y).with_context(
                || {
                    format!(
                        "Token mint Y not found or invalid: {}",
                        self.pair.token_mint_y
                    )
                },
            )?;

        let clock_data = try_get_account_data(account_map, &clock::ID)
            .with_context(|| format!("Sysvar Clock account does not exist : {}", clock::ID))?;

        let clock: Clock =
            deserialize(clock_data).with_context(|| "Failed to deserialize Clock")?;

        self.epoch = Arc::new(AtomicU64::new(clock.epoch));
        self.timestamp = Arc::new(AtomicI64::new(clock.unix_timestamp));

        self.token_transfer_fee = TokenTransferFee::new(
            &mut self.token_transfer_fee,
            mint_x_data,
            mint_x_owner,
            mint_y_data,
            mint_y_owner,
            self.epoch.load(Ordering::Relaxed),
        )?;

        (self.token_vault[0], _) = Pubkey::find_program_address(
            &[
                &self.key.to_bytes(),
                &mint_x_owner.to_bytes(),
                &self.pair.token_mint_x.to_bytes(),
            ],
            &SarosDlmm::ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
        );

        (self.token_vault[1], _) = Pubkey::find_program_address(
            &[
                &self.key.to_bytes(),
                &mint_y_owner.to_bytes(),
                &self.pair.token_mint_y.to_bytes(),
            ],
            &SarosDlmm::ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
        );

        self.token_program = [*mint_x_owner, *mint_y_owner];

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let QuoteParams {
            amount,
            swap_mode,
            input_mint,
            ..
        } = *quote_params;
        let mut pair = self.pair.clone();

        let block_timestamp = u64::try_from(self.timestamp.load(Ordering::Relaxed))?;

        let bin_for_swap = self.compute_bin_array_swap()?;

        let bin_array =
            BinArrayPair::merge(bin_for_swap.bin_arrays[0], bin_for_swap.bin_arrays[1])?;

        let swap_for_y = is_swap_for_y(input_mint, self.pair.token_mint_x);
        let (mint_in, epoch_transfer_fee_in, epoch_transfer_fee_out) = if swap_for_y {
            (
                self.pair.token_mint_x,
                self.token_transfer_fee.epoch_transfer_fee_x,
                self.token_transfer_fee.epoch_transfer_fee_y,
            )
        } else {
            (
                self.pair.token_mint_y,
                self.token_transfer_fee.epoch_transfer_fee_y,
                self.token_transfer_fee.epoch_transfer_fee_x,
            )
        };

        let (amount_in, amount_out, fee_amount) = match swap_mode {
            SwapMode::ExactIn => {
                let (amount_in_after_transfer_fee, _) =
                    compute_transfer_fee(epoch_transfer_fee_in, amount)?;

                let (amount_out, fee_amount) = get_swap_result(
                    &mut pair,
                    bin_array,
                    amount_in_after_transfer_fee,
                    swap_for_y,
                    SwapType::ExactIn,
                    block_timestamp,
                )?;

                let (amount_out_after_transfer_fee, _) =
                    compute_transfer_fee(epoch_transfer_fee_out, amount_out)?;

                (amount, amount_out_after_transfer_fee, fee_amount)
            }
            SwapMode::ExactOut => {
                let (amount_out_before_transfer_fee, _) =
                    compute_transfer_amount_for_expected_output(epoch_transfer_fee_out, amount)?;

                let (amount_in, fee_amount) = get_swap_result(
                    &mut pair,
                    bin_array,
                    amount_out_before_transfer_fee,
                    swap_for_y,
                    SwapType::ExactOut,
                    block_timestamp,
                )?;

                let (amount_in_before_transfer_fee, _) =
                    compute_transfer_amount_for_expected_output(epoch_transfer_fee_in, amount_in)?;

                (amount_in_before_transfer_fee, amount, fee_amount)
            }
        };

        Ok(Quote {
            in_amount: amount_in,
            out_amount: amount_out,
            fee_amount,
            fee_mint: mint_in,
            ..Default::default()
        })
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            token_transfer_authority,
            source_token_account,
            destination_token_account,
            source_mint,
            ..
        } = swap_params;

        let bin_for_swap = self.compute_bin_array_swap()?;
        let swap_for_y = is_swap_for_y(*source_mint, self.pair.token_mint_x);

        let (user_vault_x, user_vault_y) = if swap_for_y {
            (source_token_account, destination_token_account)
        } else {
            (destination_token_account, source_token_account)
        };

        let user = *token_transfer_authority;
        let mut account_metas = Vec::new();

        {
            account_metas.push(AccountMeta::new(self.key, false));
            account_metas.push(AccountMeta::new_readonly(self.pair.token_mint_x, false));
            account_metas.push(AccountMeta::new_readonly(self.pair.token_mint_y, false));
            account_metas.push(AccountMeta::new(bin_for_swap.bin_array_keys[0], false));
            account_metas.push(AccountMeta::new(bin_for_swap.bin_array_keys[1], false));
            account_metas.push(AccountMeta::new(self.token_vault[0], false));
            account_metas.push(AccountMeta::new(self.token_vault[1], false));
            account_metas.push(AccountMeta::new(*user_vault_x, false));
            account_metas.push(AccountMeta::new(*user_vault_y, false));
            account_metas.push(AccountMeta::new_readonly(user, true));
            account_metas.push(AccountMeta::new_readonly(self.token_program[0], false));
            account_metas.push(AccountMeta::new_readonly(self.token_program[1], false));
            account_metas.push(AccountMeta::new_readonly(spl_memo::ID, false));
        }

        // If pair does not have hook, hook should be pair key (dummy)
        account_metas.push(AccountMeta::new(self.hook, false));
        account_metas.push(AccountMeta::new_readonly(rewarder_hook::ID, false));
        // This expect as the last of swap instruction
        account_metas.push(AccountMeta::new_readonly(self.event_authority, false));
        account_metas.push(AccountMeta::new_readonly(self.program_id, false));

        // Remaining accounts for hook CPI call
        if self.hook != self.key {
            account_metas.push(AccountMeta::new(bin_for_swap.hook_bin_array_keys[0], false));
            account_metas.push(AccountMeta::new(bin_for_swap.hook_bin_array_keys[1], false));
        }

        unimplemented!();

        // Ok(SwapAndAccountMetas {
        //     swap: Swap::SarosDlmm, // TODO : Add SarosDlmm to Swap enum
        //     account_metas,
        // })
    }

    fn supports_exact_out(&self) -> bool {
        true
    }

    fn is_active(&self) -> bool {
        true
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

impl SarosPositionManagement for SarosDlmm {
    fn has_hook(&self) -> bool {
        self.pair.hook.is_some()
    }

    fn get_hook(&self) -> Option<Pubkey> {
        self.pair.hook
    }

    fn get_create_position_account_metas(
        &self,
        create_position_params: CreatePositionParams,
    ) -> Result<Vec<AccountMeta>> {
        let CreatePositionParams {
            source_position,
            position_mint,
            user,
            ..
        } = create_position_params;

        let mut account_metas = Vec::new();

        let position_key = find_position(position_mint);

        {
            account_metas.push(AccountMeta::new(self.key(), false));
            account_metas.push(AccountMeta::new(position_key, false));
            account_metas.push(AccountMeta::new(position_mint, true));
            account_metas.push(AccountMeta::new(source_position, false));
            account_metas.push(AccountMeta::new(user, true));

            account_metas.push(AccountMeta::new_readonly(
                anchor_lang::system_program::ID,
                false,
            ));
            account_metas.push(AccountMeta::new_readonly(spl_token_2022::ID, false));
            account_metas.push(AccountMeta::new_readonly(
                SarosDlmm::ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
                false,
            ));

            account_metas.push(AccountMeta::new_readonly(self.event_authority, false));
            account_metas.push(AccountMeta::new_readonly(self.program_id(), false));
        }

        Ok(account_metas)
    }

    fn get_modifier_position_account_metas(
        &self,
        modifier_position_params: ModifierPositionParams,
    ) -> Result<Vec<AccountMeta>> {
        let ModifierPositionParams {
            position_token_account,
            position_key,
            position_mint,
            user_vault_x,
            user_vault_y,
            user,
            bin_array_position_lower,
            bin_array_position_upper,
            position_hook_bin_array_lower,
            position_hook_bin_array_upper,
            ..
        } = modifier_position_params;

        let mut account_metas = Vec::new();

        {
            account_metas.push(AccountMeta::new(self.key, false));
            account_metas.push(AccountMeta::new(position_key, false));
            account_metas.push(AccountMeta::new(position_mint, true));
            account_metas.push(AccountMeta::new(position_token_account, false));
            account_metas.push(AccountMeta::new(bin_array_position_lower, false));
            account_metas.push(AccountMeta::new(bin_array_position_upper, false));
            account_metas.push(AccountMeta::new_readonly(self.pair.token_mint_x, false));
            account_metas.push(AccountMeta::new_readonly(self.pair.token_mint_y, false));
            account_metas.push(AccountMeta::new(self.token_vault[0], false));
            account_metas.push(AccountMeta::new(self.token_vault[1], false));
            account_metas.push(AccountMeta::new(user_vault_x, false));
            account_metas.push(AccountMeta::new(user_vault_y, false));
            account_metas.push(AccountMeta::new(user, true));
            account_metas.push(AccountMeta::new_readonly(self.token_program[0], false));
            account_metas.push(AccountMeta::new_readonly(self.token_program[1], false));
            account_metas.push(AccountMeta::new_readonly(spl_token_2022::ID, false));
            account_metas.push(AccountMeta::new_readonly(
                anchor_lang::system_program::ID,
                false,
            ));
            account_metas.push(AccountMeta::new_readonly(spl_memo::ID, false));
            // If pair does not have hook, hook should be pair key (dummy)
            account_metas.push(AccountMeta::new(self.hook, false));
            account_metas.push(AccountMeta::new_readonly(rewarder_hook::ID, false));
            account_metas.push(AccountMeta::new_readonly(self.event_authority, false));
            account_metas.push(AccountMeta::new_readonly(self.program_id, false));

            if let Some(hook_key) = self.pair.hook {
                let hook_position = find_hook_position(position_key, hook_key);
                account_metas.push(AccountMeta::new(self.active_hook_bin_array_key[0], false));
                account_metas.push(AccountMeta::new(self.active_hook_bin_array_key[1], false));
                account_metas.push(AccountMeta::new(hook_position, false));
                account_metas.push(AccountMeta::new(position_hook_bin_array_lower, false));
                account_metas.push(AccountMeta::new(position_hook_bin_array_upper, false));
                account_metas.push(AccountMeta::new(self.active_bin_array_key[0], false));
                account_metas.push(AccountMeta::new(self.active_bin_array_key[1], false));
            }
        }

        Ok(account_metas)
    }
}
