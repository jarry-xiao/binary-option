use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo,
    },
    entrypoint::ProgramResult,
    msg,
    // program::{invoke, invoke_signed},
    // program_error::ProgramError,
    // program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    // rent::Rent,
    // sysvar::Sysvar,
};

use spl_token::{
    // instruction::{set_authority, AuthorityType},
    state::Mint,
};

use borsh::{BorshDeserialize, BorshSerialize};
// use spl_token::state::Account;

use crate::{
    error::BettingPoolError, 
    utils::{
        assert_initialized,
        assert_owned_by,
        assert_mint_authority_matches_mint,
    },
    instruction::BettingPoolInstruction,
    state::BettingPool,
};


pub struct Processor;
impl Processor {

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = BettingPoolInstruction::try_from_slice(instruction_data)?;
        match instruction {
            BettingPoolInstruction::InitializeBettingPool(args) => {
                msg!("Instruction: InitializeBettingPool");
                process_initialize_betting_pool(program_id, accounts, args.tick_size, args.capacity)
            }
            BettingPoolInstruction::PlaceBet(args) => {
                msg!("Instruction: PlaceBet");
                process_place_bet(program_id, accounts, args.amount)
            }
            BettingPoolInstruction::ResetPot => {
                msg!("Instruction: ResetPot");
                process_reset_pot(program_id, accounts)
            }
            BettingPoolInstruction::ConcedePot => {
                msg!("Instruction: ConcedePot");
                process_concede_pot(program_id, accounts)
            }
            BettingPoolInstruction::AssignWinner(args) => {
                msg!("Instruction: AssignWinner");
                process_assign_winner(program_id, accounts, args.amount)
            }
            BettingPoolInstruction::JoinPool(args) => {
                msg!("Instruction: JoinPool");
                process_join_pool(program_id, accounts, args.position, args.amount)
            }
            BettingPoolInstruction::LeavePool => {
                msg!("Instruction: LeavePool");
                process_leave_pool(program_id, accounts)
            }
        }
    }
}

pub fn process_initialize_betting_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    tick_size: u64,
    capacity: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    betting_pool.initialize(tick_size, capacity);
    betting_pool.mint = *mint_info.key;
    betting_pool.update_authority = *update_authority_info.key;
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}


pub fn process_place_bet(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let participant_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    betting_pool.place_bet(*participant_account_info.key, amount);
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_reset_pot(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    betting_pool.reset_pot();
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_concede_pot(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let participant_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    betting_pool.concede_pot(*participant_account_info.key);
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_assign_winner(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let participant_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    // betting_pool.assign_winner(*participant_account_info.key, amount);
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_join_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    position: u8,
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let participant_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    betting_pool.join_pool(*participant_account_info.key, position, amount);
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_leave_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let pool_host_account_info = next_account_info(account_info_iter)?;
    let participant_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let mint: Mint = assert_initialized(mint_info)?;
    assert_mint_authority_matches_mint(&mint, mint_authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    let betting_pool_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
    ];
    let (betting_pool_key, betting_pool_bump_seed) =
        Pubkey::find_program_address(betting_pool_seeds, program_id);
    let metadata_authority_signer_seeds = &[
        b"betting_pool",
        program_id.as_ref(),
        mint_info.key.as_ref(),
        &[betting_pool_bump_seed],
    ];
    if pool_account_info.key != &betting_pool_key {
        return Err(BettingPoolError::InvalidPoolKey.into());
    }
    let mut betting_pool = BettingPool::from_account_info(pool_account_info)?;
    betting_pool.leave_pool(*participant_account_info.key);
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}