use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum BettingPoolError {
    #[error("PublicKeyMismatch")]
    PublicKeyMismatch,
}

impl From<BettingPoolError> for ProgramError {
    fn from(e: BettingPoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
