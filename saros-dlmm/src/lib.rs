pub mod amms;
pub mod constants;
pub mod errors;
pub mod math;
pub mod route;
pub mod state;
pub mod utils;

use crate::math::swap_manager::get_swap_result;
use crate::{
    math::fees::TokenTransferFee,
    state::{
        bin_array::{BinArray, BinArrayPair},
        pair::Pair,
    },
    utils::helper::{get_bin_array_lower, get_bin_array_upper},
};
pub use amms::amm;
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams, SwapAndAccountMetas, SwapMode,
    SwapParams, try_get_account_data, try_get_account_data_and_owner,
};
use solana_sdk::pubkey;
use solana_sdk::{clock::Clock, program_pack::Pack, pubkey::Pubkey, sysvar::Sysvar};

#[derive(Clone)]
pub struct SarosDlmm {
    pub program_id: Pubkey,
    pub key: Pubkey,
    pub label: String,
    pub pair: Pair,

    pub token_transfer_fee: TokenTransferFee,

    pub bin_array_lower: BinArray,
    pub bin_array_upper: BinArray,
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

    fn from_keyed_account(keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self>
    where
        Self: Sized,
    {
        let key = keyed_account.key;
        let label = "SarosDlmm"[..].to_string();
        let pair = Pair::unpack(&keyed_account.account.data[..])?;

        let bin_array_lower = BinArray::default();
        let bin_array_upper = BinArray::default();

        Ok(Self {
            program_id: keyed_account.account.owner,
            key,
            label,
            pair,
            token_transfer_fee: TokenTransferFee::default(),
            bin_array_lower,
            bin_array_upper,
        })
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.pair.token_mint_x, self.pair.token_mint_y]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        let bin_array_index = self.pair.bin_array_index();
        let (bin_array_lower, _) =
            get_bin_array_lower(bin_array_index, &self.key, &self.program_id());
        let (bin_array_upper, _) =
            get_bin_array_upper(bin_array_index, &self.key, &self.program_id());
        return vec![
            bin_array_lower,
            bin_array_upper,
            self.pair.token_mint_x,
            self.pair.token_mint_y,
        ];
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        println!("start update:");

        let bin_array_index = self.pair.bin_array_index();
        let (bin_array_lower, _) =
            get_bin_array_lower(bin_array_index, &self.key, &self.program_id());
        let (bin_array_upper, _) =
            get_bin_array_upper(bin_array_index, &self.key, &self.program_id());

        let bin_array_lower_data = try_get_account_data(account_map, &bin_array_lower)?;
        let bin_array_lower = &BinArray::unpack(&bin_array_lower_data)?;

        let bin_array_upper_data = try_get_account_data(account_map, &bin_array_upper)?;
        let bin_array_upper = &BinArray::unpack(&bin_array_upper_data)?;

        let (mint_x_data, mint_x_owner) =
            try_get_account_data_and_owner(account_map, &self.pair.token_mint_x)?;
        let (mint_y_data, mint_y_owner) =
            try_get_account_data_and_owner(account_map, &self.pair.token_mint_y)?;

        self.token_transfer_fee = TokenTransferFee::new(
            &mut self.token_transfer_fee,
            mint_x_data,
            &mint_x_owner,
            mint_y_data,
            &mint_y_owner,
        )?;

        self.bin_array_lower = bin_array_lower.clone();
        self.bin_array_upper = bin_array_upper.clone();

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let from_amount = quote_params.amount;

        let mut current_pair = self.pair.clone();

        let mut bin_array =
            BinArrayPair::merge(self.bin_array_lower.clone(), self.bin_array_upper.clone())?;

        let clock = Clock::get()?;
        let block_timestamp = clock.unix_timestamp as u64;
        let swap_for_y = self.pair.resolve_mints(quote_params.input_mint)?;

        let (amount_in, amount_out, fee_amount) = match quote_params.swap_mode {
            SwapMode::ExactIn => {
                let (amount_in_after_transfer_fee, _) = self
                    .token_transfer_fee
                    .compute_transfer_fee_amount(swap_for_y, from_amount)
                    .unwrap();

                let (amount_out, fee_amount) = get_swap_result(
                    &mut current_pair,
                    &mut bin_array,
                    amount_in_after_transfer_fee,
                    swap_for_y,
                    quote_params.swap_mode,
                    block_timestamp,
                )?;

                (from_amount, amount_out, fee_amount)
            }
            SwapMode::ExactOut => {
                let (amount_out_after_transfer_fee, _) = self
                    .token_transfer_fee
                    .compute_transfer_fee_amount(swap_for_y, from_amount)
                    .unwrap();

                let (amount_in, fee_amount) = get_swap_result(
                    &mut current_pair,
                    &mut bin_array,
                    amount_out_after_transfer_fee,
                    swap_for_y,
                    quote_params.swap_mode,
                    block_timestamp,
                )?;

                let (amount_in_before_transfer_fee, _) = self
                    .token_transfer_fee
                    .compute_transfer_amount_for_expected_output(swap_for_y, amount_in)?;

                (amount_in_before_transfer_fee, from_amount, fee_amount)
            }
        };

        Ok(Quote {
            in_amount: amount_in,
            out_amount: amount_out,
            fee_amount,
            fee_mint: if swap_for_y {
                self.pair.token_mint_y
            } else {
                self.pair.token_mint_x
            },
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

        let (user_vault_x, user_vault_y) = if *source_mint == self.pair.token_mint_x {
            (source_token_account, destination_token_account)
        } else {
            (destination_token_account, source_token_account)
        };

        let pair = self.key();
        let token_mint_x = self.pair.token_mint_x;
        let token_mint_y = self.pair.token_mint_y;
        let bin_array_lower: Pubkey =
            get_bin_array_lower(self.pair.bin_array_index(), &pair, &self.program_id()).0;
        let bin_array_upper: Pubkey =
            get_bin_array_upper(self.pair.bin_array_index(), &pair, &self.program_id()).0;
        let token_vault_x = Pubkey::default(); // Placeholder, replace with actual vault key
        let token_vault_y = Pubkey::default(); // Placeholder, replace with actual vault key

        let token_program_x = Pubkey::default(); // Placeholder, replace with actual token program key
        let token_program_y = Pubkey::default(); // Placeholder, replace with actual token program key
        let memo_program = Pubkey::default(); // Placeholder

        let account_metas = vec![
            AccountMeta::new(pair, false),
            AccountMeta::new(token_mint_x, false),
            AccountMeta::new(token_mint_y, false),
            AccountMeta::new(bin_array_lower, false),
            AccountMeta::new(bin_array_upper, false),
            AccountMeta::new(token_vault_x, false),
            AccountMeta::new(token_vault_y, false),
            AccountMeta::new(*user_vault_x, false),
            AccountMeta::new(*user_vault_y, false),
            AccountMeta::new(*token_transfer_authority, true),
            AccountMeta::new(token_program_x, false),
            AccountMeta::new(token_program_y, false),
            AccountMeta::new(memo_program, false),
        ];

        todo!("Implement Swap::SarosDlmm for SarosSwap");

        // Ok({
        //     SwapAndAccountMetas {
        //         swap: Swap::SarosDlmm,
        //         account_metas,
        //     }
        // });
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}
