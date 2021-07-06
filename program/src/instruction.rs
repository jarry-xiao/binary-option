use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub struct TradeArgs {
    pub size: u64,
    pub buy_price: u64,
    pub sell_price: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub enum BettingPoolInstruction {
    // TODO: Add comments here
    InitializeBettingPool,

    Trade(TradeArgs),

    Settle,

    Collect,
}

/// Creates an InitializeBettingPool instruction
#[allow(clippy::too_many_arguments)]
pub fn initailize_betting_pool(
    program_id: Pubkey,
    pool_account: Pubkey,
    long_escrow_mint: Pubkey,
    short_escrow_mint: Pubkey,
    long_escrow_account: Pubkey,
    short_escrow_account: Pubkey,
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
    mint_authority: Pubkey,
    update_authority: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(pool_account, false),
            AccountMeta::new_readonly(long_escrow_mint, false),
            AccountMeta::new_readonly(short_escrow_mint, false),
            AccountMeta::new_readonly(long_escrow_account, false),
            AccountMeta::new_readonly(short_escrow_account, false),
            AccountMeta::new_readonly(long_token_mint, false),
            AccountMeta::new_readonly(short_token_mint, false),
            AccountMeta::new_readonly(mint_authority, false),
            AccountMeta::new_readonly(update_authority, true),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: BettingPoolInstruction::InitializeBettingPool
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
    size: u64,
    buy_price: u64,
    sell_price: u64,
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
        data: BettingPoolInstruction::Trade(TradeArgs {
            size,
            buy_price,
            sell_price,
        })
        .try_to_vec()
        .unwrap(),
    }
}
