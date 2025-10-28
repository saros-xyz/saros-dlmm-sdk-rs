# Changelog

## [Unreleased]

## [Added]
- **Rewarder Hook (Mainnet Preparation)**  
  - Introduced a new `rewarder-hook` crate containing the IDL.
  - Added a new `rewarder_hook.so` binary fixture built with the `mainnet` feature.
  - Integrated rewarder hook support into DLMM swap flow.
  - Hook execution will be enabled once pools are upgraded and `has_hook` is set to `true` .

- **Liquidity Book Refactor**  
  - Introduced a new `liquidity-book` crate (previously part of the `saros` module).
  - Moved related IDL and logic to the new crate for better modularity and separation of concerns.

- **Swap Instruction Enhancements**  
  - Added two new required accounts to the swap instruction: `hook` and `hook_program`.
  - Added conditional hook execution logic triggered when `has_hook = true` on the pool.

- **Testing & Fixtures**  
  - Added `.so` fixture files for both `liquidity-book` and `rewarder-hook` programs (mainnet build).
  - Updated DLMM tests to include scenarios for rewarder hook integration.
  - Added example setup in `test_amms.rs` for future devnet testing once hooks are enabled.

## [Changed]
- **Workspace & Build Configuration**  
  - Updated `Cargo.toml` workspace to include `liquidity-book` and `rewarder-hook` crates.
  - Changed Rust edition from `2024` → `2021` for better toolchain compatibility.

- **DLMM SDK Core**  
  - Refactored `swap_instruction.rs`, `loader.rs`, and `test_harness.rs` to support hook account handling.
  - Improved program structure for future integration with rewarder hook logic.
  - Updated test and fixture handling for smoother local build & deployment.

## [Removed]
- Removed legacy references to the old `saros` structure and migrated related logic to the new `liquidity-book` crate.

## 📌 Notes
- Pool upgrade (to set `hook = Some(Pubkey)`) will be required to enable this feature.  
- Once activated, the swap instruction will require:
  - `hook` account
  - `hook_program` account
  - **Remaining accounts**: 2 additional accounts corresponding to the active hook bin array (lower & upper).
