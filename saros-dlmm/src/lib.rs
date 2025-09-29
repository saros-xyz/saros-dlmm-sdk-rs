pub mod amms;
pub mod route;
pub mod swap_instruction;

pub use amms::amm;
use anchor_lang::prelude::AccountMeta;
use anyhow::{Context, Result};
use bincode::deserialize;
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas,
    SwapMode, SwapParams, try_get_account_data, try_get_account_data_and_owner,
};
use saros_sdk::{
    math::{
        fees::{
            TokenTransferFee, compute_transfer_amount_for_expected_output, compute_transfer_fee,
        },
        swap_manager::get_swap_result,
    },
    state::{
        bin_array::{BinArray, BinArrayPair},
        pair::Pair,
    },
    utils::helper::{
        find_event_authority, get_bin_array_lower, get_bin_array_upper, get_hook_bin_array,
        is_swap_for_y,
    },
};
use solana_sdk::{
    program_pack::Pack,
    pubkey,
    pubkey::Pubkey,
    sysvar::{clock, clock::Clock},
};
use std::sync::{
    Arc,
    atomic::{AtomicI64, AtomicU64, Ordering},
};

#[derive(Clone)]
pub struct SarosDlmm {
    pub program_id: Pubkey,
    pub key: Pubkey,
    pub label: String,
    pub pair: Pair,
    pub token_transfer_fee: TokenTransferFee,
    pub bin_array_lower: BinArray,
    pub bin_array_upper: BinArray,
    pub bin_array_key: [Pubkey; 2],
    pub hook_bin_array_key: [Pubkey; 2],
    pub token_vault: [Pubkey; 2],
    pub token_program: [Pubkey; 2],
    pub event_authority: Pubkey,
    pub epoch: Arc<AtomicU64>,
    pub timestamp: Arc<AtomicI64>,
}

impl SarosDlmm {
    const ASSOCIATED_TOKEN_PROGRAM_ADDRESS: Pubkey =
        pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
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

        let (bin_array_lower_key, _) = get_bin_array_lower(
            bin_array_index,
            &keyed_account.key,
            &keyed_account.account.owner,
        );
        let (bin_array_upper_key, _) = get_bin_array_upper(
            bin_array_index,
            &keyed_account.key,
            &keyed_account.account.owner,
        );

        let (hook_bin_array_lower_key, hook_bin_array_upper_key) =
            get_hook_bin_array(bin_array_index, &keyed_account.key);

        let event_authority = find_event_authority(keyed_account.account.owner);

        Ok(Self {
            program_id: keyed_account.account.owner,
            key: keyed_account.key,
            label: "saros_dlmm".into(),
            pair,
            token_transfer_fee: TokenTransferFee::default(),
            bin_array_lower: BinArray::default(),
            bin_array_upper: BinArray::default(),
            bin_array_key: [bin_array_lower_key, bin_array_upper_key],
            hook_bin_array_key: [hook_bin_array_lower_key, hook_bin_array_upper_key],
            token_vault: [Pubkey::default(), Pubkey::default()],
            token_program: [Pubkey::default(), Pubkey::default()],
            event_authority,
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

        let (bin_array_lower_key, _) =
            get_bin_array_lower(bin_array_index, &self.key, &self.program_id);

        let (bin_array_upper_key, _) =
            get_bin_array_upper(bin_array_index, &self.key, &self.program_id);

        if self.bin_array_key[0] != bin_array_lower_key
            || self.bin_array_key[1] != bin_array_upper_key
        {
            self.bin_array_key = [bin_array_lower_key, bin_array_upper_key];
            let (hook_bin_array_lower_key, hook_bin_array_upper_key) =
                get_hook_bin_array(bin_array_index, &self.key);
            self.hook_bin_array_key = [hook_bin_array_lower_key, hook_bin_array_upper_key];
        } else {
            let bin_array_lower_data = try_get_account_data(account_map, &bin_array_lower_key)
                .with_context(|| {
                    format!(
                        "Bin array lower account does not exist or has not been initialized: {}",
                        self.bin_array_key[0]
                    )
                })?;

            self.bin_array_lower = BinArray::unpack(bin_array_lower_data)?;

            let bin_array_upper_data = try_get_account_data(account_map, &bin_array_upper_key)
                .with_context(|| {
                    format!(
                        "Bin array upper account does not exist or has not been initialized: {}",
                        self.bin_array_key[1]
                    )
                })?;

            self.bin_array_upper = BinArray::unpack(bin_array_upper_data)?;
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

        let bin_array = BinArrayPair::merge(self.bin_array_lower, self.bin_array_upper)?;

        let block_timestamp = u64::try_from(self.timestamp.load(Ordering::Relaxed))?;
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
                    swap_mode,
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
                    swap_mode,
                    block_timestamp,
                )?;

                let (amount_in_before_transfer_fee, _) =
                    compute_transfer_amount_for_expected_output(epoch_transfer_fee_in, amount_in)?;

                let (amount_out_after_transfer_fee, _) =
                    compute_transfer_fee(epoch_transfer_fee_out, amount)?;

                (
                    amount_in_before_transfer_fee,
                    amount_out_after_transfer_fee,
                    fee_amount,
                )
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

        let swap_for_y = is_swap_for_y(*source_mint, self.pair.token_mint_x);

        let (user_vault_x, user_vault_y) = if swap_for_y {
            (source_token_account, destination_token_account)
        } else {
            (destination_token_account, source_token_account)
        };

        let user = *token_transfer_authority;

        let _account_metas = vec![
            AccountMeta::new(self.key, false),
            AccountMeta::new_readonly(self.pair.token_mint_x, false),
            AccountMeta::new_readonly(self.pair.token_mint_y, false),
            AccountMeta::new(self.bin_array_key[0], false),
            AccountMeta::new(self.bin_array_key[1], false),
            AccountMeta::new(self.token_vault[0], false),
            AccountMeta::new(self.token_vault[1], false),
            AccountMeta::new(*user_vault_x, false),
            AccountMeta::new(*user_vault_y, false),
            AccountMeta::new_readonly(user, true),
            AccountMeta::new_readonly(self.token_program[0], false),
            AccountMeta::new_readonly(self.token_program[1], false),
            AccountMeta::new_readonly(spl_memo::ID, false),
            AccountMeta::new_readonly(self.event_authority, false),
            AccountMeta::new_readonly(self.program_id, false),
            AccountMeta::new(self.hook_bin_array_key[0], false),
            AccountMeta::new(self.hook_bin_array_key[1], false),
        ];

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
