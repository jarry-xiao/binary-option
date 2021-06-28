use solana_program::{
    pubkey::Pubkey,
    instruction::{AccountMeta, Instruction},
    sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};


#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct InitializeBettingPoolArgs {
    pub tick_size: u64,
    pub capacity: u8,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct PlaceBetArgs {
    pub amount: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct AssignWinnerArgs {
    pub amount: u64,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct JoinPoolArgs {
    pub position: u8,
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum BettingPoolInstruction {
    // TODO: Add comments here
    InitializeBettingPool(InitializeBettingPoolArgs),

    PlaceBet(PlaceBetArgs),

    ResetPot,

    ConcedePot,

    AssignWinner(AssignWinnerArgs),

    JoinPool(JoinPoolArgs),

    LeavePool,
}

/// Creates an InitializeBettingPool instruction
#[allow(clippy::too_many_arguments)]
pub fn initailize_betting_pool(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    update_authority: Pubkey,
    tick_size: u64,
    capacity: u8,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::InitializeBettingPool(InitializeBettingPoolArgs {tick_size, capacity})
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an PlaceBet instruction
#[allow(clippy::too_many_arguments)]
pub fn place_bet(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    player_account: Pubkey,
    update_authority: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(player_account, true),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::PlaceBet(PlaceBetArgs {amount})
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an InitializeBettingPool instruction
#[allow(clippy::too_many_arguments)]
pub fn reset_pot(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    update_authority: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::ResetPot
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an ConcedePot instruction
#[allow(clippy::too_many_arguments)]
pub fn concede_pot(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    player_account: Pubkey,
    update_authority: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(player_account, true),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::ConcedePot
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an AssignWinner instruction
#[allow(clippy::too_many_arguments)]
pub fn assign_winner(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    player_account: Pubkey,
    update_authority: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(player_account, false),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::AssignWinner(AssignWinnerArgs{amount})
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an JoinPool instruction
#[allow(clippy::too_many_arguments)]
pub fn join_pool(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    player_account: Pubkey,
    update_authority: Pubkey,
    position: u8,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(player_account, true),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::JoinPool(JoinPoolArgs{position, amount})
            .try_to_vec()
            .unwrap(),
    }
}

/// Creates an LeavePool instruction
#[allow(clippy::too_many_arguments)]
pub fn leave_pool(
    program_id: Pubkey,
    pool_account: Pubkey,
    mint: Pubkey,
    mint_authority: Pubkey,
    pool_host: Pubkey,
    player_account: Pubkey,
    update_authority: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(mint_authority, true),
            AccountMeta::new_readonly(pool_host, true),
            AccountMeta::new_readonly(player_account, true),
            AccountMeta::new_readonly(update_authority, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: 
            BettingPoolInstruction::LeavePool
            .try_to_vec()
            .unwrap(),
    }
}