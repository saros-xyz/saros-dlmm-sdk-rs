use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorCode {
    #[error("Unable to divide by zero")]
    DivideByZero, // 0x1770
    #[error("Unable to cast number into BigInt")]
    NumberCastError, //  0x1771

    #[error("Bin array index mismatch")]
    BinArrayIndexMismatch, // 0x1772

    #[error("Bin not found within bin array")]
    BinNotFound, // 0x1773

    #[error("Invalid Mint")]
    InvalidMint, // 0x1774

    #[error("Transfer fee calculation error")]
    TransferFeeCalculationError, // 0x1775

    #[error("Amount Over Flow")]
    AmountOverflow, // 0x1776

    #[error("Amount Under Flow")]
    AmountUnderflow, // 0x1777

    #[error("Active id underflow")]
    ActiveIdUnderflow, // 0x1778

    #[error("Active id overflow")]
    ActiveIdOverflow, // 0x1779

    #[error("Invalid amount in")]
    InvalidAmountIn, // 0x177a

    #[error("Invalid amount out")]
    InvalidAmountOut, // 0x177b

    #[error("MulShr Math Error")]
    MulShrMathError, // 0x177c

    #[error("ShlDiv Math Error")]
    ShlDivMathError, // 0x177d

    #[error("U64 conversion overflow")]
    U64ConversionOverflow, // 0x177e

    #[error("Swap crosses too many bins – quote aborted")]
    SwapCrossesTooManyBins,
}

impl From<TryFromIntError> for ErrorCode {
    fn from(_: TryFromIntError) -> Self {
        ErrorCode::NumberCastError
    }
}
