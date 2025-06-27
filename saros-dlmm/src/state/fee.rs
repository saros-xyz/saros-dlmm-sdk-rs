use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_sdk::program_error::ProgramError;
use solana_sdk::program_pack::{IsInitialized, Pack, Sealed};
pub struct StaticFeeParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub max_volatility_accumulator: u32,
    pub protocol_share: u16,
    pub _space: [u8; 2],
}

/// IsInitialized is required to use `Pack::pack` and `Pack::unpack`
impl IsInitialized for StaticFeeParameters {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Sealed for StaticFeeParameters {}
impl Pack for StaticFeeParameters {
    const LEN: usize = 64;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 20];
        let (
            base_factory,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            max_volatility_accumulator,
            protocol_share,
            _space,
        ) = mut_array_refs![output, 2, 2, 2, 2, 4, 4, 2, 2];

        *base_factory = self.base_factor.to_le_bytes();
        *filter_period = self.filter_period.to_le_bytes();
        *decay_period = self.decay_period.to_le_bytes();
        *reduction_factor = self.reduction_factor.to_le_bytes();
        *variable_fee_control = self.variable_fee_control.to_le_bytes();
        *max_volatility_accumulator = self.max_volatility_accumulator.to_le_bytes();
        *protocol_share = self.protocol_share.to_le_bytes();
        *_space = self._space;
    }

    fn unpack_from_slice(input: &[u8]) -> Result<StaticFeeParameters, ProgramError> {
        let input = array_ref![input, 0, 20];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            base_factory,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            max_volatility_accumulator,
            protocol_share,
            _space,
        ) = array_refs![input, 2, 2, 2, 2, 4, 4, 2, 2];

        Ok(Self {
            base_factor: u16::from_le_bytes(*base_factory),
            filter_period: u16::from_le_bytes(*filter_period),
            decay_period: u16::from_le_bytes(*decay_period),
            reduction_factor: u16::from_le_bytes(*reduction_factor),
            variable_fee_control: u32::from_le_bytes(*variable_fee_control),
            max_volatility_accumulator: u32::from_le_bytes(*max_volatility_accumulator),
            protocol_share: u16::from_le_bytes(*protocol_share),
            _space: *_space,
        })
    }
}

impl Clone for StaticFeeParameters {
    fn clone(&self) -> Self {
        Self {
            base_factor: self.base_factor,
            filter_period: self.filter_period,
            decay_period: self.decay_period,
            reduction_factor: self.reduction_factor,
            variable_fee_control: self.variable_fee_control,
            max_volatility_accumulator: self.max_volatility_accumulator,
            protocol_share: self.protocol_share,
            _space: self._space,
        }
    }
}

pub struct DynamicFeeParameters {
    pub time_last_updated: u64,
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub id_reference: u32,
    _space: [u8; 4],
}

impl IsInitialized for DynamicFeeParameters {
    fn is_initialized(&self) -> bool {
        true
    }
}
impl Sealed for DynamicFeeParameters {}
impl Pack for DynamicFeeParameters {
    const LEN: usize = 24;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 24];
        let (time_last_updated, volatility_accumulator, volatility_reference, id_reference, _space) =
            mut_array_refs![output, 8, 4, 4, 4, 4];
        *time_last_updated = self.time_last_updated.to_le_bytes();
        *volatility_accumulator = self.volatility_accumulator.to_le_bytes();
        *volatility_reference = self.volatility_reference.to_le_bytes();
        *id_reference = self.id_reference.to_le_bytes();
        *_space = self._space;
    }

    fn unpack_from_slice(input: &[u8]) -> Result<DynamicFeeParameters, ProgramError> {
        let input = array_ref![input, 0, 24];
        #[allow(clippy::ptr_offset_with_cast)]
        let (time_last_updated, volatility_accumulator, volatility_reference, id_reference, _space) =
            array_refs![input, 8, 4, 4, 4, 4];

        Ok(Self {
            time_last_updated: u64::from_le_bytes(*time_last_updated),
            volatility_accumulator: u32::from_le_bytes(*volatility_accumulator),
            volatility_reference: u32::from_le_bytes(*volatility_reference),
            id_reference: u32::from_le_bytes(*id_reference),
            _space: *_space,
        })
    }
}

impl Clone for DynamicFeeParameters {
    fn clone(&self) -> Self {
        Self {
            time_last_updated: self.time_last_updated,
            volatility_accumulator: self.volatility_accumulator,
            volatility_reference: self.volatility_reference,
            id_reference: self.id_reference,
            _space: self._space,
        }
    }
}
