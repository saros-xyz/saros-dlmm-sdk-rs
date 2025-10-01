#[cfg(not(any(feature = "mainnet", feature = "devnet")))]
compile_error!("Either 'mainnet' or 'devnet' feature must be enabled");

#[cfg(feature = "devnet")]
mod devnet;

#[cfg(feature = "mainnet")]
mod mainnet;

#[cfg(feature = "mainnet")]
pub use mainnet::*;

#[cfg(feature = "devnet")]
pub use devnet::*;
