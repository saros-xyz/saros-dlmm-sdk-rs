use solana_sdk::{pubkey, pubkey::Pubkey};

pub const HOOK_PROGRAM_ID: Pubkey = pubkey!("As95SYi4G1bJc4gReng7qp1rBjyS2ULbHmd6WYcmKzq2");

pub const POOL_LISTS: [Pubkey; 2] = [
    pubkey!("ADPKeitAZsAeRJfhG2GoDrZENB3xt9eZmggkj7iAXY78"),
    pubkey!("Cy75bt7SkreqcEE481HsKChWJPM7kkS3svVWKRPpS9UK"),
];

pub const RPC_URL: &str = "https://api.mainnet-beta.solana.com";
pub const SAROS_DLMM_SO_PATH: &str = "saros-dlmm/tests/programs/mainnet/saros_dlmm.so";
pub const SAROS_HOOK_SO_PATH: &str = "saros-dlmm/tests/programs/mainnet/rewarder_hook.so";
