use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo,
    },
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};
use spl_token::{
    state::{Mint, Account},
    instruction::AuthorityType,
};
use std::cmp::min;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::{
    error::BettingPoolError, 
    validation_utils::{
        assert_initialized,
        assert_owned_by,
        assert_mint_authority_matches_mint,
        assert_keys_equal,
    },
    spl_utils::{spl_burn, spl_mint_to, spl_set_authority, spl_token_transfer_signed},
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
            BettingPoolInstruction::InitializeBettingPool => {
                msg!("Instruction: InitializeBettingPool");
                process_initialize_betting_pool(program_id, accounts)
            }
            BettingPoolInstruction::Trade(args) => {
                msg!("Instruction: Trade");
                process_trade(program_id, accounts, args.size, args.buy_price, args.sell_price)
            }
        }
    }
}

pub fn process_initialize_betting_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let long_escrow_mint_info = next_account_info(account_info_iter)?;
    let short_escrow_mint_info = next_account_info(account_info_iter)?;
    let long_escrow_account_info = next_account_info(account_info_iter)?;
    let short_escrow_account_info = next_account_info(account_info_iter)?;
    let long_token_mint_info = next_account_info(account_info_iter)?;
    let short_token_mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let long_token_mint: Mint = assert_initialized(long_token_mint_info)?;
    let short_token_mint: Mint = assert_initialized(short_token_mint_info)?;
    assert_mint_authority_matches_mint(&long_token_mint, mint_authority_info)?;
    assert_mint_authority_matches_mint(&short_token_mint, mint_authority_info)?;
    assert_owned_by(long_token_mint_info, &spl_token::id())?;
    assert_owned_by(short_token_mint_info, &spl_token::id())?;
    assert_owned_by(long_escrow_account_info, update_authority_info.key)?;
    assert_owned_by(short_escrow_account_info, update_authority_info.key)?;
    assert_keys_equal(*token_program_info.key, spl_token::id())?;

    // Create Associtated Token Accounts
    let mut betting_pool = BettingPool::try_from_slice(&pool_account_info.data.borrow_mut())?;
    betting_pool.circulation = 0;
    betting_pool.long_mint_account_pubkey = *long_token_mint_info.key;
    betting_pool.short_mint_account_pubkey = *short_token_mint_info.key;
    betting_pool.long_escrow_mint_account_pubkey = *long_escrow_mint_info.key;
    betting_pool.short_escrow_mint_account_pubkey = *short_escrow_mint_info.key;
    betting_pool.long_escrow_account_pubkey = *long_escrow_account_info.key;
    betting_pool.short_escrow_account_pubkey = *short_escrow_account_info.key;
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;

    // Get program derived address for escrow
    let (escrow_owner_key, escrow_owner_bump_seed) = Pubkey::find_program_address(
        &[
            long_token_mint_info.key.as_ref(),
            short_token_mint_info.key.as_ref(),
            token_program_info.key.as_ref(),
            program_id.as_ref(),
        ],
         program_id,
    );

    // Transfer ownership of the escrow accounts to a PDA
    spl_set_authority(token_program_info, long_escrow_account_info, Some(escrow_owner_key), AuthorityType::AccountOwner, update_authority_info)?;
    spl_set_authority(token_program_info, long_escrow_account_info, Some(escrow_owner_key), AuthorityType::AccountOwner, update_authority_info)?;

    Ok(())
}

pub fn process_trade(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    size: u64,
    buy_price: u64,
    sell_price: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let long_escrow_mint_info = next_account_info(account_info_iter)?;
    let short_escrow_mint_info = next_account_info(account_info_iter)?;
    let long_escrow_account_info = next_account_info(account_info_iter)?;
    let short_escrow_account_info = next_account_info(account_info_iter)?;
    let long_token_mint_info = next_account_info(account_info_iter)?;
    let short_token_mint_info = next_account_info(account_info_iter)?;
    let buyer_info = next_account_info(account_info_iter)?;
    let seller_info = next_account_info(account_info_iter)?;
    let buyer_long_account_info = next_account_info(account_info_iter)?;
    let buyer_short_account_info = next_account_info(account_info_iter)?;
    let seller_long_account_info = next_account_info(account_info_iter)?;
    let seller_short_account_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let escrow_authority_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    // Unpack accounts
    let long_token_mint: Mint = assert_initialized(long_token_mint_info)?;
    let short_token_mint: Mint = assert_initialized(short_token_mint_info)?;
    let buyer_long_account: Account = assert_initialized(buyer_long_account_info)?;
    let buyer_short_account: Account = assert_initialized(buyer_short_account_info)?;
    let seller_long_account: Account = assert_initialized(seller_long_account_info)?;
    let seller_short_account: Account = assert_initialized(seller_short_account_info)?;
    let mut betting_pool = BettingPool::try_from_slice(&pool_account_info.data.borrow_mut())?;
    // Get program derived address for escrow
    let (escrow_owner_key, bump_seed) = Pubkey::find_program_address(
        &[
            long_token_mint_info.key.as_ref(),
            short_token_mint_info.key.as_ref(),
            token_program_info.key.as_ref(),
            program_id.as_ref(),
        ],
         program_id,
    );
    let seeds = &[
        long_token_mint_info.key.as_ref(),
        short_token_mint_info.key.as_ref(),
        token_program_info.key.as_ref(),
        program_id.as_ref(),
        &[bump_seed],
    ];

    // Validate data
    assert_mint_authority_matches_mint(&long_token_mint, mint_authority_info)?;
    assert_mint_authority_matches_mint(&short_token_mint, mint_authority_info)?;
    assert_owned_by(long_token_mint_info, &spl_token::id())?;
    assert_owned_by(short_token_mint_info, &spl_token::id())?;
    assert_keys_equal(betting_pool.long_mint_account_pubkey, *long_token_mint_info.key)?;
    assert_keys_equal(betting_pool.short_mint_account_pubkey, *short_token_mint_info.key)?;
    assert_keys_equal(escrow_owner_key, *escrow_authority_info.key)?;


    if buyer_long_account.amount > 0 && buyer_short_account.amount > 0 {
        let burn_amount = min(buyer_long_account.amount, buyer_short_account.amount);
        spl_burn(&token_program_info, &buyer_long_account_info, &long_token_mint_info, &buyer_info, burn_amount)?;
        spl_burn(&token_program_info, &buyer_short_account_info, &short_token_mint_info, &buyer_info, burn_amount)?;
        betting_pool.decrement_supply(burn_amount)?;
    }
    if seller_long_account.amount > 0 && seller_short_account.amount > 0 {
        let burn_amount = min(seller_long_account.amount, seller_short_account.amount);
        spl_burn(&token_program_info, &seller_long_account_info, &long_token_mint_info, &seller_info, burn_amount)?;
        spl_burn(&token_program_info, &seller_short_account_info, &short_token_mint_info, &seller_info, burn_amount)?;
        betting_pool.decrement_supply(burn_amount)?;
    }

    let N = size;
    let Nb = buyer_short_account.amount; 
    let Ns = seller_long_account.amount; 
    match [Nb > N, Ns > N] {
        [true, true] => {
            spl_burn(&token_program_info, &buyer_short_account_info, &short_token_mint_info, &buyer_info, N)?;
            spl_burn(&token_program_info, &seller_short_account_info, &long_token_mint_info, &seller_info, N)?;
            spl_token_transfer_signed(token_program_info, short_escrow_account_info, buyer_info, escrow_authority_info, N * buy_price, seeds)?;
            spl_token_transfer_signed(token_program_info, long_escrow_account_info, seller_info, escrow_authority_info, N * sell_price, seeds)?;
            betting_pool.decrement_supply(N)?;
        }
        [false, false] => {
            spl_burn(&token_program_info, &buyer_short_account_info, &short_token_mint_info, &buyer_info, Nb)?;
            spl_burn(&token_program_info, &seller_short_account_info, &long_token_mint_info, &seller_info, Ns)?;
            spl_mint_to(&token_program_info, &buyer_long_account_info, &long_token_mint_info, &buyer_info, N - Nb)?;
            spl_mint_to(&token_program_info, &seller_short_account_info, &short_token_mint_info, &seller_info, N - Ns)?;
            spl_token_transfer_signed(token_program_info, short_escrow_account_info, buyer_info, escrow_authority_info, Nb * buy_price, seeds)?;
            spl_token_transfer_signed(token_program_info, long_escrow_account_info, seller_info, escrow_authority_info, Ns * sell_price, seeds)?;
            if N > Nb + Ns {
                betting_pool.increment_supply(N - Nb - Ns);
            } else {
                betting_pool.decrement_supply(N - Nb - Ns)?;
            }
        } 
        [false, true] => {
            spl_burn(&token_program_info, &buyer_short_account_info, &short_token_mint_info, &buyer_info, N)?;
            spl_burn(&token_program_info, &seller_short_account_info, &long_token_mint_info, &seller_info, Ns)?;
            spl_mint_to(&token_program_info, &seller_short_account_info, &short_token_mint_info, &seller_info, N - Ns)?;
            spl_token_transfer_signed(token_program_info, short_escrow_account_info, buyer_info, escrow_authority_info, N * buy_price, seeds)?;
            spl_token_transfer_signed(token_program_info, long_escrow_account_info, seller_info, escrow_authority_info, Ns * sell_price, seeds)?;
            betting_pool.decrement_supply(Ns)?;
        } 
        [true, false] => {
            spl_burn(&token_program_info, &seller_short_account_info, &long_token_mint_info, &seller_info, N)?;
            spl_burn(&token_program_info, &buyer_short_account_info, &short_token_mint_info, &buyer_info, Nb)?;
            spl_mint_to(&token_program_info, &buyer_long_account_info, &long_token_mint_info, &buyer_info, N - Nb)?;
            spl_token_transfer_signed(token_program_info, short_escrow_account_info, buyer_info, escrow_authority_info, Nb * buy_price, seeds)?;
            spl_token_transfer_signed(token_program_info, long_escrow_account_info, seller_info, escrow_authority_info, N * sell_price, seeds)?;
            betting_pool.decrement_supply(Nb)?;
        }
    }
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_settle(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    // TODO
    Ok(())
}