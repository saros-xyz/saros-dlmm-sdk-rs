use solana_sdk::{pubkey, pubkey::Pubkey};

pub const HOOK_PROGRAM_ID: Pubkey = pubkey!("BXpvgXGKDKax82p9JK2E8jRoKLify6eq3mMaW1wHVhuK");

pub const POOL_LISTS: [Pubkey; 1] = [
    // pubkey!("B4y8jmpmMHUyV33DEYhtAm39SHqak9Peh2TAjnmwZKQQ"),
    // pubkey!("4GxEnxgBKhagZx1p8wKcThW2tCKjZdiNwojYxijy4Ec3"),
    // pubkey!("FgkVLL52vSYuHWUyH7zGotLYmYFeXEdvC65NaqafFiCK"),
    pubkey!("FvKuEuRyfDZ8catHJznC7heKLkC1uopRaaKMDY1Nym2T"),
    // pubkey!("Hqmo9QQVyJFq8RGodrEABbwyMbdQ6fmdWbUAtjMH3pyW"),
];

pub const RPC_URL: &str = "https://api.devnet.solana.com";

pub const SAROS_DLMM_SO_PATH: &str = "saros-dlmm/tests/programs/devnet/saros_dlmm.so";
pub const SAROS_HOOK_SO_PATH: &str = "saros-dlmm/tests/programs/devnet/rewarder_hook.so";
