use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum BettingPoolError {
    #[error("ExpectedAmountMismatch")]
    ExpectedAmountMismatch,
    #[error("InvalidInstruction")]
    InvalidInstruction,
    #[error("AlreadyInUse")]
    AlreadyInUse,
    #[error("Uninitialized")]
    Uninitialized,
    #[error("ExpectedMint")]
    ExpectedMint,
    #[error("NotMintAuthority")]
    NotMintAuthority,
    #[error("InvalidMintAuthority")]
    InvalidMintAuthority,
    #[error("IncorrectOwner")]
    IncorrectOwner,
    #[error("NotRentExempt")]
    NotRentExempt,
    #[error("InvalidPoolKey")]
    InvalidPoolKey,
    #[error("InsufficientFunds")]
    InsufficientFunds,
    #[error("InvalidProgramAddress")]
    InvalidProgramAddress,
    #[error("InvalidAuthorityAccount")]
    InvalidAuthorityAccount,
    #[error("InvalidOwner")]
    InvalidOwner,
    #[error("DifferentCollateralUsed")]
    DifferentCollateralUsed,
    #[error("InvalidSupply")]
    InvalidSupply,
    #[error("InvalidFreezeAuthority")]
    InvalidFreezeAuthority,
    #[error("IncorrectPoolMint")]
    IncorrectPoolMint,
    #[error("IncorrectTokenProgramId")]
    IncorrectTokenProgramId,
    #[error("InvalidMints")]
    InvalidMints,
    #[error("InvalidAccountKeys")]
    InvalidAccountKeys,
    #[error("WouldBeLiquidated")]
    WouldBeLiquidated,
    #[error("InsufficientMargin")]
    InsufficientMargin,
    #[error("InvalidTransferTime")]
    InvalidTransferTime,
    #[error("ExpectedAccount")]
    ExpectedAccount,
    #[error("AccountNotInitialized")]
    AccountNotInitialized,
}

impl From<BettingPoolError> for ProgramError {
    fn from(e: BettingPoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
