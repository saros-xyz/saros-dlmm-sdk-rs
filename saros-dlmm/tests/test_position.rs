use std::collections::HashMap;

use ahash::RandomState;
use jupiter_amm_interface::{Amm, AmmContext, ClockRef, KeyedAccount};
use saros_dlmm_sdk::amms::position_manager::SarosPositionManagement;
use saros_dlmm_sdk::amms::test_harness::AmmTestHarness;
use saros_dlmm_sdk::route::get_token_mints_permutations;
use saros_dlmm_sdk::SarosDlmm;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{account::Account, pubkey};

/// Common test entry for position lifecycle
async fn test_position_full_circle_for_amm_key<T>(
    amm_key: Pubkey,
    option: Option<String>,
    before_test_setup: Option<impl FnMut(&dyn Amm, &mut HashMap<Pubkey, Account, RandomState>)>,
    restricted_mint_permutations: Option<Vec<(Pubkey, Pubkey)>>,
    bin_left: i32,
    bin_right: i32,
) where
    T: SarosPositionManagement + Amm + Send + Sync + 'static,
{
    let test_harness = AmmTestHarness::new_with_rpc_url("".into(), amm_key, option);
    let keyed_account: KeyedAccount = test_harness.get_keyed_account_from_snapshot().unwrap();

    let amm_context = AmmContext {
        clock_ref: ClockRef::from(test_harness.get_clock()),
    };

    let amm = T::from_keyed_account(&keyed_account, &amm_context).unwrap();

    test_position_full_circle::<T>(
        &test_harness,
        Box::new(amm),
        before_test_setup,
        restricted_mint_permutations,
        bin_left,
        bin_right,
    )
    .await;
}

/// Macro generate tests for position lifecycle
macro_rules! test_position_full_circle {
    ($(($amm_key:expr, $amm_struct:ty, $bin_left:expr, $bin_right:expr),)*) => {
        $(
            paste::item! {
                #[tokio::test]
                async fn [<test_position_full_circle_ $amm_key:lower>]() {
                    let before_test_setup: Option<
                        fn(&dyn Amm, &mut HashMap<Pubkey, Account, RandomState>)
                    > = None;
                    let option = None;
                    test_position_full_circle_for_amm_key::<$amm_struct>(
                        $amm_key,
                        option,
                        before_test_setup,
                        None,
                        $bin_left,
                        $bin_right,
                    ).await;
                }
            }
        )*
    };
}

const SAROS_DLMM_USD1_USDC_POOL: Pubkey = pubkey!("8yrUdy1XufCuupHgbpptcer1npNkQDVh95sLnc67CfR2");
const SAROS_DLMM_LAUNCHCOIN_USDT_POOL: Pubkey =
    pubkey!("Cy75bt7SkreqcEE481HsKChWJPM7kkS3svVWKRPpS9UK");

// Pool with hook
const SAROS_DLMM_USDT_USDC_POOL: Pubkey = pubkey!("5MapG47uiTvj8ua9EKxXHuvwq1wCx8y78cgB2jSjNqzT");

// run
test_position_full_circle! {
    (SAROS_DLMM_USD1_USDC_POOL, SarosDlmm, -5, 5),
    (SAROS_DLMM_LAUNCHCOIN_USDT_POOL, SarosDlmm, -5, 5),
    (SAROS_DLMM_USDT_USDC_POOL, SarosDlmm, -5, 5),
}

async fn test_position_full_circle<T: SarosPositionManagement + Amm + Send + Sync>(
    test_harness: &AmmTestHarness,
    mut amm: Box<T>,
    mut before_test_setup: Option<impl FnMut(&dyn Amm, &mut HashMap<Pubkey, Account, RandomState>)>,
    restricted_mint_permutations: Option<Vec<(Pubkey, Pubkey)>>,
    bin_left: i32,
    bin_right: i32,
) where
    T: SarosPositionManagement + Amm + Send + Sync,
{
    let amm = amm.as_mut();
    let reserve_token_mint_permutations =
        restricted_mint_permutations.unwrap_or(get_token_mints_permutations(amm));

    let mut test_harness_program_test = test_harness
        .load_program_test(amm, before_test_setup.as_mut())
        .await;

    let (mint_x, mint_y) = reserve_token_mint_permutations[0].clone();
    test_harness_program_test
        .assert_position_life_circle(amm, &mint_x, &mint_y, bin_left, bin_right)
        .await;
}
