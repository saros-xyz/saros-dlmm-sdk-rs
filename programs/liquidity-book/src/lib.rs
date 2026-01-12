use anchor_lang::prelude::*;

declare_id!("1qbkdrr3z4ryLA7pZykqxvxWPoeifcVKo6ZG9CfkvVE");
declare_program!(liquidity_book);

use rand::distributions::{Distribution, Uniform};
use solana_sdk::pubkey::Pubkey;

// Now, we only support up to 8 authorities between [0, 1, 2, 3, 4, 5, 6, 7]. To create more authorities, we need to
// add them in the monorepo. We can use from 0 up to 255 in order to prevent hot accounts.
pub const AUTHORITY_COUNT: u8 = 16;
pub const AUTHORITY_SEED: &[u8] = b"authority";

pub fn find_authorities() -> Vec<Pubkey> {
    (0..AUTHORITY_COUNT).map(find_program_authority).collect()
}

pub fn find_event_authority() -> Pubkey {
    Pubkey::find_program_address(
        &[b"__event_authority"],
        &Pubkey::new_from_array(*crate::ID.as_array()),
    )
    .0
}

pub fn find_find_program_authority_id((start, end): (u8, u8)) -> u8 {
    let mut rng = rand::thread_rng();
    let ids = Uniform::from(start..end);
    ids.sample(&mut rng)
}

pub fn find_program_authority(id: u8) -> Pubkey {
    Pubkey::find_program_address(
        &[AUTHORITY_SEED, &[id]],
        &Pubkey::new_from_array(*crate::ID.as_array()),
    )
    .0
}
