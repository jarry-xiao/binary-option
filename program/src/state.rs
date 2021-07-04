
use solana_program::{
    account_info::{
        AccountInfo,
    },
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
};

use borsh::{BorshDeserialize, BorshSerialize};
use crate::{
    error::BettingPoolError,
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BettingPool {
    pub circulation: u64,
    pub long_escrow_mint_account_pubkey: Pubkey,
    pub short_escrow_mint_account_pubkey: Pubkey,
    pub long_escrow_account_pubkey: Pubkey,
    pub short_escrow_account_pubkey: Pubkey,
    pub long_mint_account_pubkey: Pubkey,
    pub short_mint_account_pubkey: Pubkey,
}

impl BettingPool {
    // pub const LEN: usize = 725;

    pub fn from_account_info(a: &AccountInfo) -> Result<BettingPool, ProgramError> {
        let betting_pool = BettingPool::try_from_slice(&a.data.borrow_mut())?;
        Ok(betting_pool)
    }

    pub fn increment_supply(&mut self, n: u64) {
        self.circulation += n;
    }

    pub fn decrement_supply(&mut self, n: u64) -> ProgramResult {
        if self.circulation < n {
            return Err(BettingPoolError::InvalidSupply.into());
        }
        self.circulation -= n;
        Ok(())
    }
}