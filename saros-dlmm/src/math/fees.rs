use solana_sdk::{clock::Clock, pubkey::Pubkey, sysvar::Sysvar};
use spl_token_2022::{
    self,
    extension::{self, BaseStateWithExtensions, StateWithExtensions, transfer_fee::TransferFee},
};

use crate::{constants::BASIS_POINT_MAX, errors::ErrorCode};
use anyhow::Result;

/// Encapsulates all fee information and calculations for swap operations
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TokenTransferFee {
    pub epoch_transfer_fee_x: Option<TransferFee>,
    pub epoch_transfer_fee_y: Option<TransferFee>,
}

impl TokenTransferFee {
    pub fn new(
        &mut self,
        mint_x_data: &[u8],
        mint_x_owner: &Pubkey,
        mint_y_data: &[u8],
        mint_y_owner: &Pubkey,
    ) -> Result<Self> {
        if mint_x_owner == &spl_token::ID {
            self.epoch_transfer_fee_x = None;
        } else {
            let token_mint_unpacked =
                StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_x_data)?;

            if let Ok(transfer_fee_config) =
                token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>()
            {
                let epoch = Clock::get()?.epoch;
                self.epoch_transfer_fee_x = Some(*transfer_fee_config.get_epoch_fee(epoch));
            }
        }

        if mint_y_owner == &spl_token::ID {
            self.epoch_transfer_fee_y = None;
        } else {
            let token_mint_unpacked =
                StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_y_data)?;

            if let Ok(transfer_fee_config) =
                token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>()
            {
                let epoch = Clock::get()?.epoch;
                self.epoch_transfer_fee_y = Some(*transfer_fee_config.get_epoch_fee(epoch));
            }
        }

        Ok(Self {
            epoch_transfer_fee_x: self.epoch_transfer_fee_x,
            epoch_transfer_fee_y: self.epoch_transfer_fee_y,
        })
    }
}

pub fn compute_transfer_fee(
    epoch_transfer_fee_token_mint: Option<TransferFee>,
    amount: u64,
) -> Result<(u64, u64)> {
    if let Some(epoch_transfer_fee) = epoch_transfer_fee_token_mint {
        let transfer_fee = epoch_transfer_fee
            .calculate_fee(amount)
            .ok_or(ErrorCode::TransferFeeCalculationError)?;
        let transfer_output_amount = amount
            .checked_sub(transfer_fee)
            .ok_or(ErrorCode::TransferFeeCalculationError)?;
        return Ok((transfer_output_amount, transfer_fee));
    }

    Ok((amount, 0))
}

pub fn compute_transfer_amount(
    epoch_transfer_fee_token_mint: Option<TransferFee>,
    expected_output: u64,
) -> Result<(u64, u64)> {
    if expected_output == 0 {
        return Ok((0, 0));
    }
    if let Some(epoch_transfer_fee) = epoch_transfer_fee_token_mint {
        let transfer_fee: u64 =
            if u16::from(epoch_transfer_fee.transfer_fee_basis_points) == BASIS_POINT_MAX as u16 {
                // edge-case: if transfer fee rate is 100%, current SPL implementation returns 0 as inverse fee.
                // https://github.com/solana-labs/solana-program-library/blob/fe1ac9a2c4e5d85962b78c3fc6aaf028461e9026/token/program-2022/src/extension/transfer_fee/mod.rs#L95

                // But even if transfer fee is 100%, we can use maximum_fee as transfer fee.
                // if transfer_fee_excluded_amount + maximum_fee > u64 max, the following checked_add should fail.
                u64::from(epoch_transfer_fee.maximum_fee)
            } else {
                epoch_transfer_fee
                    .calculate_inverse_fee(expected_output)
                    .ok_or(ErrorCode::TransferFeeCalculationError)?
            };
        let transfer_fee_include_amount = expected_output
            .checked_add(transfer_fee)
            .ok_or(ErrorCode::TransferFeeCalculationError)?;

        // verify transfer fee calculation for safety
        let transfer_fee_verification = epoch_transfer_fee
            .calculate_fee(transfer_fee_include_amount)
            .ok_or(ErrorCode::TransferFeeCalculationError)?;

        if transfer_fee_verification != transfer_fee {
            return Err(ErrorCode::TransferFeeCalculationError.into());
        }
        return Ok((transfer_fee_include_amount, transfer_fee));
    }
    Ok((expected_output, 0))
}

pub fn compute_transfer_fee_amount(
    token_mint_transfer_fee: Option<TransferFee>,
    transfer_amount: u64,
) -> Result<(u64, u64)> {
    return compute_transfer_fee(token_mint_transfer_fee, transfer_amount);
}

pub fn compute_transfer_amount_for_expected_output(
    token_mint_transfer_fee: Option<TransferFee>,
    expected_output: u64,
) -> Result<(u64, u64)> {
    if expected_output == 0 {
        return Ok((0, 0));
    }
    return compute_transfer_amount(token_mint_transfer_fee, expected_output);
}
