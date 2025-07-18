use anchor_lang::prelude::AccountMeta;
use anyhow::{Context, Result};
use assert_matches::assert_matches;
use async_trait::async_trait;
use glob::glob;
use jupiter_amm_interface::{
    AccountMap, AmmContext, ClockRef, KeyedUiAccount, QuoteParams, SwapAndAccountMetas, SwapMode,
    SwapParams,
};
use lazy_static::lazy_static;

use serde_json::{Value, json};
use solana_account_decoder::{UiAccount, UiAccountEncoding, encode_ui_account};
use solana_client::{
    nonblocking,
    rpc_client::{RpcClient, RpcClientConfig},
    rpc_request::RpcRequest,
    rpc_response::{Response, RpcKeyedAccount, RpcResponseContext},
    rpc_sender::{RpcSender, RpcTransportStats},
};
use solana_program_test::{BanksClient, BanksClientError, ProgramTestContext};
use solana_sdk::{
    account::Account, clock::Clock, compute_budget::ComputeBudgetInstruction,
    instruction::Instruction, program_option::COption, program_pack::Pack, pubkey::Pubkey,
    signature::Keypair, signer::Signer, sysvar, transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::extension::StateWithExtensions;
// use stakedex_sdk::test_utils::spl_stake_pool;
use super::amm::{Amm, KeyedAccount};
use crate::{
    amms::loader::amm_factory,
    route::get_token_mints_permutations,
    swap_instruction::{BuildSwapInstructionDataParams, build_swap_instruction_data},
    utils::helper::is_swap_for_y,
};
use ahash::RandomState;
use solana_sdk::pubkey;
use std::fs::{create_dir_all, remove_dir_all};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::Path,
};
use std::{hint::black_box, str::FromStr, time::Instant};
pub const SAROS_MINT: Pubkey = pubkey!("SarosY6Vscao718M4A778z4CGtvcwcGef5M9MEH1LGL");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDT_MINT: Pubkey = pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const MASHA_MINT: Pubkey = pubkey!("mae8vJGf8Wju8Ron1oDTQVaTGGBpcpWDwoRQJALMMf2");

lazy_static! {
    // For SwapMode::ExactIn
    pub static ref TOKEN_MINT_AND_IN_AMOUNT: [(Pubkey, u64); 5] = [
        (spl_token::native_mint::ID, 2_500_000_000),
        (SAROS_MINT, 1_000_000_000),
        (MASHA_MINT, 1_000_000_000),
        (USDC_MINT, 1_110_000_000),
        (USDT_MINT, 1_110_000_000),
    ];

    // For SwapMode::ExactOut
    pub static ref TOKEN_MINT_AND_OUT_AMOUNT: [(Pubkey, u64); 5] = [
        (spl_token::native_mint::ID, 100_000_000),
        (SAROS_MINT, 500_000_000),
        (MASHA_MINT, 500_000_000),
        (USDC_MINT, 50_000_000),
        (USDT_MINT, 50_000_000),
    ];
    pub static ref TOKEN2022_MINT_AND_IN_AMOUNT: [(Pubkey, u64); 0] = [];
    pub static ref TOKEN2022_MINT_AND_OUT_AMOUNT: [(Pubkey, u64); 0] = [];

    // Mapping pubkey → amount
    pub static ref TOKEN_MINT_TO_IN_AMOUNT: HashMap<Pubkey, u64> = {
        let mut m = HashMap::from(*TOKEN_MINT_AND_IN_AMOUNT);
        m.extend(HashMap::from(*TOKEN2022_MINT_AND_IN_AMOUNT));
        m
    };

    pub static ref TOKEN_MINT_TO_OUT_AMOUNT: HashMap<Pubkey, u64> = {
        let mut m = HashMap::from(*TOKEN_MINT_AND_OUT_AMOUNT);
        m.extend(HashMap::from(*TOKEN2022_MINT_AND_OUT_AMOUNT));
        m
    };
}

pub struct AmmTestSwapParams<'a> {
    pub amm: &'a dyn Amm,
    pub source_mint: &'a Pubkey,
    pub destination_mint: &'a Pubkey,
    pub swap_mode: SwapMode,
    pub tolerance: u64,
    pub use_shared_accounts: bool,
    pub expected_error: Option<&'a anyhow::Error>,
}

pub struct AmmTestHarness {
    pub client: RpcClient,
    pub key: Pubkey,
    pub option: Option<String>,
}

pub struct AmmTestHarnessProgramTest {
    context: ProgramTestContext,
    program_test_authority: ProgramTestAuthority,
    program_test_user: ProgramTestUser,
    option: Option<String>,
}

fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

impl AmmTestHarnessProgramTest {
    async fn process_transaction(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<(), BanksClientError> {
        let recent_blockhash = self.context.banks_client.get_latest_blockhash().await?;

        let mut all_signers = vec![&self.context.payer];
        all_signers.extend_from_slice(signers);

        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.context.payer.pubkey()),
            &all_signers,
            recent_blockhash,
        );
        println!("tx: {:?}", transaction);

        self.context
            .banks_client
            .process_transaction_with_preflight(transaction)
            .await
    }

    /// To be used for exotic test setup
    pub async fn assert_out_amount_matches_simulated_swap(
        &mut self,
        swap_instruction: Instruction,
        _input_mint: &Pubkey,
        output_mint: &Pubkey,
        _in_amount: u64,
        out_amount: u64,
        tolerance: Option<u64>,
    ) {
        let user_output_account = self.program_test_user.get_user_ata(output_mint);
        let user_keypair = clone_keypair(&self.program_test_user.keypair);
        let token_account_before = self.get_token_account(&user_output_account).await;
        let process_transaction_result = self
            .process_transaction(
                &[
                    ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
                    ComputeBudgetInstruction::request_heap_frame(256 * 1024),
                    swap_instruction,
                ],
                &[&user_keypair],
            )
            .await;
        assert_matches!(process_transaction_result, Ok(()));
        let token_account_after = self.get_token_account(&user_output_account).await;

        let simulation_amount = token_account_after
            .amount
            .checked_sub(token_account_before.amount)
            .unwrap();

        assert!(
            (out_amount as i128 - simulation_amount as i128).abs()
                <= tolerance.map(Into::into).unwrap_or(0)
        );
    }

    async fn get_token_account(&mut self, address: &Pubkey) -> spl_token_2022::state::Account {
        let token_account = self
            .context
            .banks_client
            .get_account(*address)
            .await
            .unwrap()
            .unwrap();
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&token_account.data)
            .unwrap()
            .base
    }

    /// Limited as we clone banks_client to avoid convoluting the general case
    pub fn get_test_rpc_client(&self) -> nonblocking::rpc_client::RpcClient {
        let test_rpc_sender = TestRpcSender {
            banks_client: self.context.banks_client.clone(),
        };
        nonblocking::rpc_client::RpcClient::new_sender(test_rpc_sender, RpcClientConfig::default())
    }

    pub fn get_user(&self) -> Pubkey {
        self.program_test_user.keypair.pubkey()
    }

    pub async fn assert_quote_matches_simulated_swap(
        &mut self,
        AmmTestSwapParams {
            amm,
            source_mint,
            destination_mint,
            swap_mode,
            tolerance,
            use_shared_accounts,
            expected_error,
        }: AmmTestSwapParams<'_>,
    ) {
        let user = self.program_test_user.keypair.pubkey();

        let user_source_token_account = self
            .program_test_user
            .mint_to_ata_with_program_id
            .get(source_mint)
            .unwrap()
            .0;
        let user_destination_token_account = self
            .program_test_user
            .mint_to_ata_with_program_id
            .get(destination_mint)
            .unwrap()
            .0;

        let program_source_token_account = self
            .program_test_authority
            .mint_to_ata_with_program_id
            .get(source_mint)
            .unwrap()
            .0;
        let program_destination_token_account = self
            .program_test_authority
            .mint_to_ata_with_program_id
            .get(destination_mint)
            .unwrap()
            .0;

        let mut amount = match swap_mode {
            SwapMode::ExactIn => *TOKEN_MINT_TO_IN_AMOUNT.get(source_mint).unwrap(),
            SwapMode::ExactOut => *TOKEN_MINT_TO_OUT_AMOUNT.get(destination_mint).unwrap(),
        };

        println!(
            "Testing swap: {} -> {} with amount: {}",
            source_mint, destination_mint, amount
        );

        let is_input_mint_token2022 = TOKEN2022_MINT_AND_IN_AMOUNT
            .iter()
            .any(|(mint, _)| mint == source_mint);
        let is_output_mint_token2022 = TOKEN2022_MINT_AND_IN_AMOUNT
            .iter()
            .any(|(mint, _)| mint == destination_mint);
        let source_token_account = if !use_shared_accounts || is_input_mint_token2022 {
            user_source_token_account
        } else {
            program_source_token_account
        };
        let destination_token_account = if !use_shared_accounts || is_output_mint_token2022 {
            user_destination_token_account
        } else {
            program_destination_token_account
        };

        let token_authority = if !use_shared_accounts || is_input_mint_token2022 {
            user
        } else {
            self.program_test_authority.pubkey
        };

        let mut accounts: Vec<AccountMeta> = Vec::new();
        let mut quote_count: u32 = 0;
        let mut quote_result = None;
        let mut quote_err = None;

        let reserve_mints: [Pubkey; 2] = amm.get_reserve_mints().try_into().unwrap();
        let swap_for_y = is_swap_for_y(*source_mint, reserve_mints[0]);

        // solution for amm that cant quote certain amount and also could be bug introducing, divide by 2 until can quote
        while quote_result.is_none() && quote_count < 10 {
            amount /= 2;
            match amm.quote(&QuoteParams {
                amount,
                input_mint: *source_mint,
                output_mint: *destination_mint,
                swap_mode,
            }) {
                Ok(quote) => quote_result = Some(quote),
                Err(e) => {
                    println!("quote error: {}", e);
                    quote_err = Some(e);
                }
            }

            quote_count += 1;
        }

        let swap_params = SwapParams {
            swap_mode,
            source_mint: *source_mint,
            destination_mint: *destination_mint,
            source_token_account,
            destination_token_account,
            token_transfer_authority: token_authority,
            quote_mint_to_referrer: None,
            in_amount: quote_result.unwrap().in_amount,
            out_amount: quote_result.unwrap().out_amount,
            jupiter_program_id: &Pubkey::default(),
            missing_dynamic_accounts_as_default: false,
        };

        let SwapAndAccountMetas {
            swap: _swap,
            account_metas,
        } = amm.get_swap_and_account_metas(&swap_params).unwrap();

        accounts.extend(account_metas);

        let data = build_swap_instruction_data(BuildSwapInstructionDataParams {
            amount,
            other_amount_threshold: match swap_mode {
                SwapMode::ExactIn => 0,
                SwapMode::ExactOut => swap_params.in_amount * 2,
            },
            swap_for_y,
            swap_mode,
        })
        .unwrap();

        let swap_ix = Instruction {
            program_id: saros::ID,
            accounts: accounts,
            data,
        };

        let mut ixs: Vec<Instruction> =
            vec![ComputeBudgetInstruction::set_compute_unit_limit(1_400_000)];

        ixs.push(swap_ix);

        let user_keypair = clone_keypair(&self.program_test_user.keypair);
        let source_token_account_before = self.get_token_account(&user_source_token_account).await;
        let destination_token_account_before = self
            .get_token_account(&user_destination_token_account)
            .await;
        let process_transaction_result = self.process_transaction(&ixs, &[&user_keypair]).await;
        let source_token_account_after = self.get_token_account(&user_source_token_account).await;
        let destination_token_account_after = self
            .get_token_account(&user_destination_token_account)
            .await;

        let source_token_account_diff = source_token_account_before
            .amount
            .checked_sub(source_token_account_after.amount)
            .unwrap();
        let destination_token_account_diff = destination_token_account_after
            .amount
            .checked_sub(destination_token_account_before.amount)
            .unwrap();

        let quote = if let Some(expected_error) = expected_error {
            let quote_err = quote_err.unwrap();
            match expected_error.downcast_ref::<anchor_lang::error::Error>() {
                Some(error) => {
                    let quote_error = quote_err
                        .downcast_ref::<anchor_lang::error::Error>()
                        .unwrap();
                    assert_eq!(error, quote_error);
                }
                None => {
                    assert_eq!(expected_error.to_string(), quote_err.to_string());
                }
            }
            process_transaction_result.unwrap_err();
            return;
        } else {
            // We don't expect any errors
            process_transaction_result.unwrap();
            quote_result.unwrap()
        };

        println!("{source_mint} -> {destination_mint}");
        match swap_mode {
            SwapMode::ExactIn => {
                println!(
                    "ExactIn : quote.out_amount: {}, simulation_out_amount: {}, exact_in_amount: {}, simulation_in_amount: {}",
                    quote.out_amount,
                    destination_token_account_diff,
                    amount,
                    source_token_account_diff,
                );
                assert!(
                    (quote.out_amount as i128 - destination_token_account_diff as i128).abs()
                        <= tolerance as i128
                );
            }
            SwapMode::ExactOut => {
                println!(
                    "ExactOut : quote.in_amount: {}, simulation_in_amount: {}, exact_out_amount: {}, simulation_out_amount: {}",
                    quote.in_amount,
                    source_token_account_diff,
                    amount,
                    destination_token_account_diff,
                );
                assert!(
                    (quote.in_amount as i128 - source_token_account_diff as i128).abs()
                        <= tolerance as i128
                );
                assert!(amount == destination_token_account_diff);
            }
        }

        // Benchmark Quote
        let now = Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            let quote = amm
                .quote(&QuoteParams {
                    amount,
                    input_mint: *source_mint,
                    output_mint: *destination_mint,
                    swap_mode,
                })
                .unwrap();
            black_box(quote);
        }
        let elapsed = now.elapsed();
        println!(
            "Amm {}, iterations: {iterations}, Quote time per iteration: {} us",
            amm.label(),
            elapsed.as_micros() as f64 / (iterations as f64),
        );
    }
}

struct TestRpcSender {
    banks_client: BanksClient,
}

#[async_trait]
impl RpcSender for TestRpcSender {
    async fn send(
        &self,
        request: RpcRequest,
        params: serde_json::Value,
    ) -> std::result::Result<serde_json::Value, solana_client::client_error::ClientError> {
        let banks_client = self.banks_client.clone();
        let context = RpcResponseContext {
            slot: banks_client.get_root_slot().await.unwrap(),
            api_version: None,
        };
        match request {
            RpcRequest::GetAccountInfo => {
                let pubkey = Pubkey::from_str(params[0].as_str().unwrap()).unwrap();
                let account = banks_client.get_account(pubkey).await.unwrap().unwrap();

                Ok(serde_json::to_value(Response {
                    context,
                    value: encode_ui_account(
                        &pubkey,
                        &account,
                        UiAccountEncoding::Base64,
                        None,
                        None,
                    ),
                })
                .unwrap())
            }
            RpcRequest::GetVersion => Ok(json!({"solana-core": "2.1.16"})),
            _ => Err(solana_client::client_error::ClientError {
                request: Some(request),
                kind: solana_client::client_error::ClientErrorKind::Custom(
                    "Method not supported".into(),
                ),
            }),
        }
    }
    fn get_transport_stats(&self) -> RpcTransportStats {
        RpcTransportStats::default()
    }
    fn url(&self) -> String {
        "bla".into()
    }
}

pub type AccountsSnapshot = HashMap<Pubkey, Account>;

pub struct ProgramTestAuthority {
    id: u8,
    pubkey: Pubkey,
    mint_to_ata_with_program_id: HashMap<Pubkey, (Pubkey, Pubkey)>,
}

pub struct ProgramTestUser {
    keypair: Keypair,
    mint_to_ata_with_program_id: HashMap<Pubkey, (Pubkey, Pubkey)>,
}

impl ProgramTestUser {
    fn get_user_ata(&self, mint: &Pubkey) -> Pubkey {
        self.mint_to_ata_with_program_id.get(mint).unwrap().0
    }
}

/// Update AMM with only the accounts it requested,
/// to avoid relying on side effects
fn update_amm_precise(amm: &mut dyn Amm, account_map: &AccountMap) -> Result<()> {
    let account_map_requested = HashMap::from_iter(
        amm.get_accounts_to_update()
            .into_iter()
            .filter_map(|address| {
                account_map
                    .get(&address)
                    .cloned()
                    .map(|account| (address, account))
            }),
    );
    amm.update(&account_map_requested)
}

impl AmmTestHarness {
    pub fn new_with_rpc_url(rpc_url: String, key: Pubkey, option: Option<String>) -> Self {
        Self {
            client: RpcClient::new(rpc_url),
            key,
            option,
        }
    }

    pub fn directory_name(&self) -> String {
        let option = match &self.option {
            Some(option) => format!("-{}", option),
            None => "".to_string(),
        };

        format!("{}{option}", self.key)
    }

    pub fn get_keyed_account(&self, key: Pubkey) -> Result<KeyedAccount> {
        let account = self.client.get_account(&key)?;
        Ok(KeyedAccount {
            key,
            account,
            params: None,
        })
    }

    pub fn get_keyed_account_from_snapshot(&self) -> Result<KeyedAccount> {
        let directory_name = self.directory_name();
        let file_path = format!(
            "tests/fixtures/accounts/{0}/{1}.json",
            directory_name, self.key,
        );

        let file =
            File::open(&file_path).unwrap_or_else(|_| panic!("Snapshot file {file_path} exists"));
        let keyed_account: RpcKeyedAccount = serde_json::from_reader(file).unwrap();
        let account: Account = UiAccount::decode(&keyed_account.account).unwrap();
        let params_file_path = format!("tests/fixtures/accounts/{0}/params.json", directory_name);
        let mut params: Option<Value> = None;

        // check if params file exists
        if Path::new(&params_file_path).exists() {
            let file = File::open(params_file_path).unwrap();
            params = serde_json::from_reader(file).unwrap();
        }

        Ok(KeyedAccount {
            key: self.key,
            account,
            params,
        })
    }

    /// Returns an account from the snapshot
    pub fn get_account_from_snapshot(&self, address: &Pubkey) -> Account {
        let directory_name = self.directory_name();
        let file_path = format!(
            "tests/fixtures/accounts/{0}/{1}.json",
            directory_name, address,
        );
        let file = File::open(file_path).unwrap();
        let keyed_account: RpcKeyedAccount = serde_json::from_reader(file).unwrap();
        UiAccount::decode(&keyed_account.account).unwrap()
    }

    pub fn update_amm(&self, amm: &mut dyn Amm) {
        let accounts_to_update = amm.get_accounts_to_update();

        let account_map: HashMap<Pubkey, Account, RandomState> = self
            .client
            .get_multiple_accounts(&accounts_to_update)
            .unwrap()
            .into_iter()
            .zip(accounts_to_update)
            .fold(HashMap::default(), |mut m, (account, address)| {
                if let Some(account) = account {
                    m.insert(address, account);
                }
                m
            });

        amm.update(&account_map).unwrap();
    }

    fn load_accounts_snapshot(&self) -> HashMap<Pubkey, Account, RandomState> {
        let mut account_map: HashMap<Pubkey, Account, RandomState> = HashMap::default();
        for entry in glob(&format!(
            "tests/fixtures/accounts/{0}/*.json",
            self.directory_name()
        ))
        .unwrap()
        {
            if let Ok(entry) = entry {
                if entry.ends_with("params.json") {
                    continue;
                }
                let file = File::open(entry).unwrap();
                let keyed_account: RpcKeyedAccount = serde_json::from_reader(file).unwrap();
                let account: Account = UiAccount::decode(&keyed_account.account).unwrap();
                account_map.insert(Pubkey::from_str(&keyed_account.pubkey).unwrap(), account);
            }
        }
        account_map
    }

    pub fn update_amm_from_snapshot(&self, amm: &mut dyn Amm) -> Result<()> {
        let accounts_snapshot = self.load_accounts_snapshot();
        update_amm_precise(amm, &accounts_snapshot).unwrap();

        Ok(())
    }

    pub async fn load_program_test(
        &self,
        amm: &mut dyn Amm,
        before_test_setup: Option<&mut impl FnMut(&dyn Amm, &mut AccountMap)>,
    ) -> AmmTestHarnessProgramTest {
        use solana_program_test::ProgramTest;

        let mut pt = ProgramTest::default();

        pt.prefer_bpf(true);
        // https://github.com/solana-labs/solana/issues/26589
        // Some programs such as Raydium AMM are not functional once this feature gate is enabled
        pt.deactivate_feature(pubkey!("7Vced912WrRnfjaiKRiNBcbuFw7RrnLv3E3z95Y4GTNc"));

        pt.add_program("saros_dlmm", saros::ID, None);

        // let modified_label = amm.label().to_lowercase().replace(' ', "_");
        pt.add_program(
            Box::leak(amm.label().into_boxed_str()),
            amm.program_id(),
            None,
        );

        for (program_id, program_name) in amm.program_dependencies() {
            // if program_id == spl_stake_pool::ID {
            //     program_name = "spl_stake_pool".into(); // spl stake pool labels describe the state rather than the program
            // }
            pt.add_program(Box::leak(program_name.into_boxed_str()), program_id, None);
        }

        let mut accounts_snapshot = self.load_accounts_snapshot();

        // Modify the original snapshot before it gets loaded in the context or in the Amm
        if let Some(before_test_setup) = before_test_setup {
            before_test_setup(amm, &mut accounts_snapshot);
        }

        let mut clock: Option<Clock> = None;
        for (address, account) in accounts_snapshot.iter() {
            if address == &sysvar::clock::ID {
                clock = Some(bincode::deserialize::<Clock>(&account.data).unwrap());
            }
            if !account.executable {
                pt.add_account(*address, account.clone());
            }
        }

        for _ in 0..3 {
            update_amm_precise(amm, &accounts_snapshot).unwrap();
        }

        let mut context = pt.start_with_context().await;
        if let Some(clock) = clock {
            context.set_sysvar(&clock);
        }

        let program_test_authority =
            AmmTestHarness::setup_authority(&mut context, amm.get_reserve_mints()).await;
        let program_test_user =
            AmmTestHarness::setup_user(&mut context, amm.get_reserve_mints()).await;

        AmmTestHarnessProgramTest {
            context,
            program_test_authority,
            program_test_user,
            option: self.option.clone(),
        }
    }

    pub fn get_clock(&self) -> Clock {
        let account_map = self.load_accounts_snapshot();
        let clock: Clock = match account_map.get(&sysvar::clock::ID) {
            Some(account) => bincode::deserialize(&account.data)
                .context("Failed to deserialize sysvar::clock::ID")
                .unwrap(),
            None => Clock::default(), // some amms don't have clock snapshot
        };

        clock
    }

    /// Setup user and mutate token accounts with funded ATAs
    async fn setup_user(
        context: &mut ProgramTestContext,
        reserve_mints: Vec<Pubkey>,
    ) -> ProgramTestUser {
        let keypair = Keypair::new();
        let user = keypair.pubkey();

        let mint_to_ata_with_program_id =
            setup_token_accounts(&user, context, reserve_mints, true).await;

        ProgramTestUser {
            keypair,
            mint_to_ata_with_program_id,
        }
    }

    /// Setup progrma authority and mutate token accounts with funded ATAs
    async fn setup_authority(
        context: &mut ProgramTestContext,
        reserve_mints: Vec<Pubkey>,
    ) -> ProgramTestAuthority {
        use saros::find_program_authority;

        let authority_id = 0;
        let program_authority = find_program_authority(authority_id);

        let mint_to_ata_with_program_id =
            setup_token_accounts(&program_authority, context, reserve_mints, false).await;

        ProgramTestAuthority {
            id: authority_id,
            pubkey: program_authority,
            mint_to_ata_with_program_id,
        }
    }

    /// Setup AMM

    /// Snapshot necessary accounts to perform a swap so that we can reload it later on for reproducible tests
    /// Saved as <amm-id><option>/<address>.json, with the amm id to avoid collision between AMMs
    pub fn snapshot_amm_accounts(
        &self,
        amm: &dyn Amm,
        params: Option<Value>,
        force: bool,
    ) -> Result<()> {
        let placeholder = Pubkey::new_unique();
        let mut addresses_for_snapshot = HashSet::new();
        for (source_mint, destination_mint) in get_token_mints_permutations(amm) {
            let swap_leg_and_account_metas: jupiter_amm_interface::SwapAndAccountMetas = amm
                .get_swap_and_account_metas(&SwapParams {
                    source_mint,
                    destination_mint,
                    source_token_account: placeholder,
                    destination_token_account: placeholder,
                    token_transfer_authority: placeholder,
                    quote_mint_to_referrer: None,
                    in_amount: *TOKEN_MINT_TO_IN_AMOUNT
                        .get(&source_mint)
                        .unwrap_or_else(|| panic!("No in amount for mint: {}", source_mint)),
                    out_amount: *TOKEN_MINT_TO_IN_AMOUNT
                        .get(&source_mint)
                        .unwrap_or_else(|| panic!("No in amount for mint: {}", destination_mint)),
                    jupiter_program_id: &placeholder,
                    missing_dynamic_accounts_as_default: false,
                    swap_mode: SwapMode::ExactIn,
                })?;

            addresses_for_snapshot.extend(
                swap_leg_and_account_metas
                    .account_metas
                    .iter()
                    .map(|account_meta| account_meta.pubkey),
            );
        }

        addresses_for_snapshot.extend(amm.get_accounts_to_update());
        addresses_for_snapshot.extend(amm.get_reserve_mints());
        addresses_for_snapshot.remove(&placeholder);
        // Some AMMs read the clock sysvar
        addresses_for_snapshot.insert(sysvar::clock::ID);

        let snapshot_path_string = format!("tests/fixtures/accounts/{}", self.directory_name());
        let snapshot_path = Path::new(&snapshot_path_string);
        if force {
            if snapshot_path.exists() && snapshot_path.is_dir() {
                // Remove the directory if it exists
                remove_dir_all(snapshot_path)?;
            }
        }

        create_dir_all(snapshot_path)?;
        if params.is_some() {
            let mut f = File::create(snapshot_path.join("params.json")).unwrap();
            f.write_all(serde_json::to_value(params).unwrap().to_string().as_bytes())
                .unwrap();
        }

        let addresses = addresses_for_snapshot.into_iter().collect::<Vec<_>>();
        self.client
            .get_multiple_accounts(&addresses)
            .unwrap()
            .iter()
            .zip(addresses)
            .for_each(|(account, address)| {
                if let Some(account) = account {
                    if account.executable {
                        // Avoid snapshotting programs as it breaks program test
                        return;
                    }
                    let keyed_account = RpcKeyedAccount {
                        pubkey: address.to_string(),
                        account: encode_ui_account(
                            &address,
                            account,
                            UiAccountEncoding::Base64,
                            None,
                            None,
                        ),
                    };
                    let mut f =
                        File::create(snapshot_path.join(format!("{}.json", address))).unwrap();
                    f.write_all(
                        serde_json::to_value(keyed_account)
                            .unwrap()
                            .to_string()
                            .as_bytes(),
                    )
                    .unwrap();
                }
            });

        Ok(())
    }
}

async fn setup_token_accounts(
    wallet: &Pubkey,
    context: &mut ProgramTestContext,
    reserve_mints: Vec<Pubkey>,
    with_bootstrap_amounts: bool,
) -> HashMap<Pubkey, (Pubkey, Pubkey)> {
    use solana_sdk::system_instruction;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use spl_token_2022::extension::StateWithExtensionsMut;

    let mut setup_ixs = vec![system_instruction::transfer(
        &context.payer.pubkey(),
        wallet,
        1_000_000_000,
    )];
    let mut mint_to_ata_with_program_id = HashMap::new();
    let mut ata_to_set_amount = HashMap::new();

    // We only snapshot mints for token2022, as a result we can only naturally create ATAs for token2022
    for reserve_mint in reserve_mints.iter() {
        let (in_amount, token_program_id) = TOKEN_MINT_AND_IN_AMOUNT
            .iter()
            .find_map(|(mint, in_amount)| {
                if mint == reserve_mint {
                    Some((*in_amount, spl_token::ID))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                TOKEN2022_MINT_AND_IN_AMOUNT
                    .iter()
                    .find_map(|(mint, in_amount)| {
                        if mint == reserve_mint {
                            Some((*in_amount, spl_token_2022::ID))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| panic!("Token mint to be defined: {}", reserve_mint))
            });

        let ata = if token_program_id == spl_token_2022::ID {
            let ata = get_associated_token_address_with_program_id(
                wallet,
                reserve_mint,
                &spl_token_2022::ID,
            );
            setup_ixs.push(create_associated_token_account(
                &context.payer.pubkey(),
                wallet,
                reserve_mint,
                &spl_token_2022::ID,
            ));
            ata_to_set_amount.insert(ata, in_amount * 110 / 10);
            ata
        } else {
            let (ata, token_account) =
                create_ata_account(wallet, reserve_mint, in_amount * 110 / 10, spl_token::ID);
            context.set_account(&ata, &token_account.into());
            ata
        };

        mint_to_ata_with_program_id.insert(*reserve_mint, (ata, spl_token::ID));
    }

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &setup_ixs,
            Some(&context.payer.pubkey()),
            &[&context.payer],
            context.last_blockhash,
        ))
        .await
        .unwrap();

    if with_bootstrap_amounts {
        for (ata, set_amount) in ata_to_set_amount {
            let mut account = context
                .banks_client
                .get_account(ata)
                .await
                .unwrap()
                .unwrap();

            let mut token_account =
                StateWithExtensionsMut::<spl_token_2022::state::Account>::unpack(&mut account.data)
                    .unwrap();
            token_account.base.amount = set_amount;
            token_account.pack_base();

            context.set_account(&ata, &account.into());
        }
    }

    mint_to_ata_with_program_id
}

fn create_ata_account(
    user: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    token_program_id: Pubkey,
) -> (Pubkey, Account) {
    let ata = get_associated_token_address_with_program_id(user, mint, &token_program_id);

    let mut is_native = COption::None;
    let mut lamports = 10_000_000; // More than enough
    if mint == &spl_token::native_mint::ID {
        let rent = 2_039_280;
        is_native = COption::Some(rent);
        lamports = amount + rent;
    };
    let token_account = spl_token::state::Account {
        mint: *mint,
        owner: *user,
        amount,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let mut data = [0; spl_token::state::Account::LEN].to_vec();
    spl_token::state::Account::pack(token_account, &mut data).unwrap();

    (
        ata,
        Account {
            lamports,
            data,
            owner: token_program_id,
            executable: false,
            rent_epoch: 0,
        },
    )
}

pub async fn take_snapshot(
    rpc_url: String,
    amm_id: String,
    option: Option<String>,
    force: bool,
) -> Result<()> {
    let amm_key = Pubkey::from_str(&amm_id).unwrap();

    let client = RpcClient::new(&rpc_url);

    let amm_context = get_amm_context(&client).await?;

    let account = client
        .get_account(&amm_key)
        .expect("Should find AMM in markets cache or on-chain");

    let ui_account = encode_ui_account(&amm_key, &account, UiAccountEncoding::Base64, None, None);

    let keyed_ui_account = KeyedUiAccount {
        pubkey: amm_id,
        ui_account,
        params: None,
    };

    let keyed_account: KeyedAccount = keyed_ui_account.try_into()?;

    let test_harness = AmmTestHarness::new_with_rpc_url(rpc_url, amm_key, option);

    let mut saber_wrapper_mints = HashSet::new();

    let mut amm = amm_factory(&keyed_account, &amm_context, &mut saber_wrapper_mints)?;

    let amm: &mut (dyn Amm + Send + Sync) = amm.as_mut();
    for _ in 0..3 {
        test_harness.update_amm(amm);
    }

    test_harness.snapshot_amm_accounts(amm, keyed_account.params, force)?;

    Ok(())
}

pub async fn get_clock(rpc_client: &RpcClient) -> anyhow::Result<Clock> {
    let clock_data = rpc_client
        .get_account_with_commitment(&sysvar::clock::ID, rpc_client.commitment())?
        .value
        .context("Failed to get clock account")?;

    let clock: Clock = bincode::deserialize(&clock_data.data)
        .context("Failed to deserialize sysvar::clock::ID")?;

    Ok(clock)
}

pub async fn get_clock_ref(rpc_client: &RpcClient) -> anyhow::Result<ClockRef> {
    let clock = get_clock(rpc_client).await?;
    Ok(ClockRef::from(clock))
}

pub async fn get_amm_context(rpc_client: &RpcClient) -> anyhow::Result<AmmContext> {
    Ok(AmmContext {
        clock_ref: get_clock_ref(rpc_client).await?,
    })
}
