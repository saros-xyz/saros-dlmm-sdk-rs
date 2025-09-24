use crate::constants::{
    BASIS_POINT_MAX, MAX_ACTIVE_ID, PRECISION, SQUARED_PRECISION, VARIABLE_FEE_PRECISION,
};
use crate::errors::ErrorCode;
use crate::state::bin::BIN_ARRAY_SIZE;
use crate::state::fee::{DynamicFeeParameters, StaticFeeParameters};
use anyhow::Result;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use jupiter_amm_interface::SwapMode;
use solana_sdk::program_error::ProgramError;
use solana_sdk::{
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub struct Pair {
    _discriminator: [u8; 8],
    pub bump: [u8; 1],

    pub liquidity_book_config: Pubkey,

    pub bin_step: u8,
    pub bin_step_seed: [u8; 1],

    pub token_mint_x: Pubkey,
    pub token_mint_y: Pubkey,

    pub static_fee_parameters: StaticFeeParameters,

    // Dynamic parameters
    pub active_id: u32,
    pub dynamic_fee_parameters: DynamicFeeParameters,
    pub protocol_fees_x: u64,
    pub protocol_fees_y: u64,

    pub hook: Option<Pubkey>,
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for Pair {
    fn is_initialized(&self) -> bool {
        true
    }
}

impl Sealed for Pair {}

impl Pack for Pair {
    const LEN: usize = 204;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Pair::LEN];

        let (
            discriminator,
            bump,
            liquidity_book_config,
            bin_step,
            bin_step_seed,
            token_mint_x,
            token_mint_y,
            static_fee_parameters,
            active_id,
            dynamic_fee_parameters,
            protocol_fees_x,
            protocol_fees_y,
            hook_flag_dst,
            hook_pubkey_dst,
        ) = mut_array_refs![output, 8, 1, 32, 1, 1, 32, 32, 20, 4, 24, 8, 8, 1, 32];

        discriminator.copy_from_slice(&[85, 72, 49, 176, 182, 228, 141, 82]);
        bump.copy_from_slice(&self.bump);
        liquidity_book_config.copy_from_slice(self.liquidity_book_config.as_ref());
        bin_step.copy_from_slice(&self.bin_step.to_le_bytes());
        bin_step_seed.copy_from_slice(&self.bin_step_seed);
        token_mint_x.copy_from_slice(self.token_mint_x.as_ref());
        token_mint_y.copy_from_slice(self.token_mint_y.as_ref());
        self.static_fee_parameters
            .pack_into_slice(&mut static_fee_parameters[..]);
        active_id.copy_from_slice(&self.active_id.to_le_bytes());
        self.dynamic_fee_parameters
            .pack_into_slice(&mut dynamic_fee_parameters[..]);
        protocol_fees_x.copy_from_slice(&self.protocol_fees_x.to_le_bytes());
        protocol_fees_y.copy_from_slice(&self.protocol_fees_y.to_le_bytes());

        match &self.hook {
            Some(hook) => {
                hook_flag_dst[0] = 1;
                hook_pubkey_dst.copy_from_slice(hook.as_ref());
            }
            None => {
                hook_flag_dst[0] = 0;
                hook_pubkey_dst.fill(0);
            }
        }
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Pair::LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            discriminator,
            bump,
            liquidity_book_config,
            bin_step,
            bin_step_seed,
            token_mint_x,
            token_mint_y,
            static_fee_parameters,
            active_id,
            dynamic_fee_parameters,
            protocol_fees_x,
            protocol_fees_y,
            hook_flag_dst,
            hook_pubkey_dst,
        ) = array_refs![input, 8, 1, 32, 1, 1, 32, 32, 20, 4, 24, 8, 8, 1, 32];

        Ok(Self {
            _discriminator: *discriminator,
            bump: *bump,
            liquidity_book_config: Pubkey::new_from_array(*liquidity_book_config),
            bin_step: u8::from_le_bytes(*bin_step),
            bin_step_seed: *bin_step_seed,
            token_mint_x: Pubkey::new_from_array(*token_mint_x),
            token_mint_y: Pubkey::new_from_array(*token_mint_y),
            static_fee_parameters: StaticFeeParameters::unpack_from_slice(static_fee_parameters)?,
            active_id: u32::from_le_bytes(*active_id),
            dynamic_fee_parameters: DynamicFeeParameters::unpack_from_slice(
                dynamic_fee_parameters,
            )?,
            protocol_fees_x: u64::from_le_bytes(*protocol_fees_x),
            protocol_fees_y: u64::from_le_bytes(*protocol_fees_y),
            hook: match hook_flag_dst {
                [0] => None,
                [1] => Some(Pubkey::new_from_array(*hook_pubkey_dst)),
                _ => return Err(ProgramError::InvalidAccountData),
            },
        })
    }
}

impl Clone for Pair {
    fn clone(&self) -> Self {
        Self {
            _discriminator: self._discriminator,
            bump: self.bump,
            liquidity_book_config: self.liquidity_book_config,
            bin_step: self.bin_step,
            bin_step_seed: self.bin_step_seed,
            token_mint_x: self.token_mint_x,
            token_mint_y: self.token_mint_y,
            static_fee_parameters: self.static_fee_parameters.clone(),
            active_id: self.active_id,
            dynamic_fee_parameters: self.dynamic_fee_parameters.clone(),
            protocol_fees_x: self.protocol_fees_x,
            protocol_fees_y: self.protocol_fees_y,
            hook: self.hook,
        }
    }
}

impl Pair {
    pub fn bin_array_index(&self) -> u32 {
        let idx = self.active_id / BIN_ARRAY_SIZE;

        idx
    }

    pub fn resolve_mints(&self, input_mint: Pubkey, swap_mode: SwapMode) -> Result<bool> {
        match swap_mode {
            SwapMode::ExactIn => {
                if input_mint == self.token_mint_x {
                    Ok(true)
                } else if input_mint == self.token_mint_y {
                    Ok(false)
                } else {
                    Err(ErrorCode::InvalidMint.into())
                }
            }
            SwapMode::ExactOut => {
                if input_mint == self.token_mint_x {
                    Ok(false)
                } else if input_mint == self.token_mint_y {
                    Ok(true)
                } else {
                    Err(ErrorCode::InvalidMint.into())
                }
            }
        }
    }

    pub fn get_total_fee(&self) -> Result<u64> {
        Ok(self.get_base_fee()? + self.get_variable_fee()?)
    }

    fn get_base_fee(&self) -> Result<u64> {
        // Base factor is in basis points, binStep is in basis points, so we multiply by 10
        Ok(u64::from(self.static_fee_parameters.base_factor)
            .checked_mul(self.bin_step.into())
            .ok_or(ErrorCode::AmountOverflow)?
            .checked_mul(10)
            .ok_or(ErrorCode::AmountOverflow)?)
    }

    fn get_variable_fee(&self) -> Result<u64> {
        let variable_fee_control = self.static_fee_parameters.variable_fee_control;

        // (volatilityAccumulator * binStep)^2 * variableFeeControl / VARIABLE_FEE_PRECISION, rounded up
        if variable_fee_control > 0 {
            let prod = u128::from(self.dynamic_fee_parameters.volatility_accumulator)
                .checked_mul(self.bin_step.into())
                .ok_or(ErrorCode::AmountOverflow)?;

            let variable_fee = (prod
                .checked_mul(prod)
                .ok_or(ErrorCode::AmountUnderflow)?
                .checked_mul(variable_fee_control.into())
                .ok_or(ErrorCode::AmountOverflow)?
                .checked_add(VARIABLE_FEE_PRECISION)
                .ok_or(ErrorCode::AmountOverflow)?
                .checked_sub(1)
                .ok_or(ErrorCode::AmountUnderflow)?)
                / VARIABLE_FEE_PRECISION;

            Ok(u64::try_from(variable_fee).map_err(|_| ErrorCode::U64ConversionOverflow)?)
        } else {
            Ok(0)
        }
    }

    pub fn get_composition_fee(&self, amount: u64) -> Result<u64> {
        let fee = self.get_total_fee()?;
        let fee_plus_precision = fee
            .checked_add(PRECISION)
            .ok_or(ErrorCode::AmountOverflow)?;

        let composition_fee = u128::from(amount)
            .checked_mul(fee.into())
            .ok_or(ErrorCode::AmountOverflow)?
            .checked_mul(fee_plus_precision.into())
            .ok_or(ErrorCode::AmountOverflow)?
            / SQUARED_PRECISION;

        Ok(u64::try_from(composition_fee).map_err(|_| ErrorCode::U64ConversionOverflow)?)
    }

    pub fn get_protocol_share(&self) -> u64 {
        self.static_fee_parameters.protocol_share as u64
    }

    pub fn update_references(&mut self, block_timestamp: u64) -> Result<()> {
        let time_delta = block_timestamp - self.dynamic_fee_parameters.time_last_updated;

        if time_delta >= self.static_fee_parameters.filter_period as u64 {
            self.dynamic_fee_parameters.id_reference = self.active_id;

            if time_delta >= self.static_fee_parameters.decay_period as u64 {
                self.dynamic_fee_parameters.volatility_reference = 0;
            } else {
                self.update_volatility_reference()?;
            }
        }

        self.dynamic_fee_parameters.time_last_updated = block_timestamp;

        Ok(())
    }

    pub fn update_volatility_reference(&mut self) -> Result<()> {
        let volatility_accumulator = u64::from(self.dynamic_fee_parameters.volatility_accumulator);

        self.dynamic_fee_parameters.volatility_reference = u32::try_from(
            volatility_accumulator
                .checked_mul(self.static_fee_parameters.reduction_factor.into())
                .ok_or(ErrorCode::AmountOverflow)?
                / BASIS_POINT_MAX as u64,
        )?;

        Ok(())
    }

    pub fn update_volatility_accumulator(&mut self) -> Result<()> {
        let delta_id = self
            .active_id
            .abs_diff(self.dynamic_fee_parameters.id_reference);

        let volatility_accumulator = u128::from(delta_id)
            .checked_mul(BASIS_POINT_MAX.into())
            .ok_or(ErrorCode::AmountOverflow)?
            .checked_add(self.dynamic_fee_parameters.volatility_reference.into())
            .ok_or(ErrorCode::AmountOverflow)?;

        let max_volatility_accumulator = self.static_fee_parameters.max_volatility_accumulator;

        self.dynamic_fee_parameters.volatility_accumulator = if volatility_accumulator
            > u128::from(max_volatility_accumulator)
        {
            max_volatility_accumulator
        } else {
            u32::try_from(volatility_accumulator).map_err(|_| ErrorCode::U64ConversionOverflow)?
        };

        Ok(())
    }

    pub fn move_active_id(&mut self, swap_for_y: bool) -> Result<()> {
        if swap_for_y {
            self.move_active_id_left()
        } else {
            self.move_active_id_right()
        }
    }

    fn move_active_id_left(&mut self) -> Result<()> {
        self.active_id = self
            .active_id
            .checked_sub(1)
            .ok_or(ErrorCode::ActiveIdUnderflow)?;

        Ok(())
    }

    fn move_active_id_right(&mut self) -> Result<()> {
        // require!(self.active_id < MAX_ACTIVE_ID, ErrorCode::ActiveIdOverflow);
        if self.active_id >= MAX_ACTIVE_ID {
            Err(ErrorCode::ActiveIdOverflow)?;
        }
        self.active_id += 1;
        Ok(())
    }
}
