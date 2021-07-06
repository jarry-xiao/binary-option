use crate::{
    instruction::BettingPoolInstruction,
    spl_utils::{
        spl_burn, spl_initialize, spl_mint_to, spl_set_authority, spl_token_transfer,
        spl_token_transfer_signed,
    },
    error::BettingPoolError,
    state::BettingPool,
    system_utils::create_or_allocate_account_raw,
    validation_utils::{
        assert_initialized, assert_keys_equal, assert_mint_authority_matches_mint, assert_owned_by,
    },
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::AuthorityType,
    state::{Account, Mint},
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
                process_trade(
                    program_id,
                    accounts,
                    args.size,
                    args.buy_price,
                    args.sell_price,
                )
            }
            BettingPoolInstruction::Settle => {
                msg!("Instruction: Settle");
                process_settle(program_id, accounts)
            }
            BettingPoolInstruction::Collect => {
                msg!("Instruction: Settle");
                process_collect(program_id, accounts)
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
    let update_authority_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;

    let long_token_mint: Mint = assert_initialized(long_token_mint_info)?;
    let short_token_mint: Mint = assert_initialized(short_token_mint_info)?;
    let long_escrow_account: Account = assert_initialized(long_escrow_account_info)?;
    let short_escrow_account: Account = assert_initialized(short_escrow_account_info)?;

    let (long_escrow_key, long_escrow_seed) = Pubkey::find_program_address(
        &[
            long_token_mint_info.key.as_ref(),
            token_program_info.key.as_ref(),
            program_id.as_ref(),
        ],
        program_id,
    );
    let long_escrow_seeds = &[
        long_token_mint_info.key.as_ref(),
        token_program_info.key.as_ref(),
        program_id.as_ref(),
        &[long_escrow_seed],
    ];

    let (short_escrow_key, short_escrow_seed) = Pubkey::find_program_address(
        &[
            short_token_mint_info.key.as_ref(),
            token_program_info.key.as_ref(),
            program_id.as_ref(),
        ],
        program_id,
    );
    let short_escrow_seeds = &[
        short_token_mint_info.key.as_ref(),
        token_program_info.key.as_ref(),
        program_id.as_ref(),
        &[short_escrow_seed],
    ];

    assert_mint_authority_matches_mint(&long_token_mint, mint_authority_info)?;
    assert_mint_authority_matches_mint(&short_token_mint, mint_authority_info)?;
    assert_owned_by(long_token_mint_info, &spl_token::id())?;
    assert_owned_by(short_token_mint_info, &spl_token::id())?;
    assert_owned_by(long_escrow_account_info, update_authority_info.key)?;
    assert_owned_by(short_escrow_account_info, update_authority_info.key)?;
    assert_keys_equal(*token_program_info.key, spl_token::id())?;
    assert_keys_equal(long_escrow_account.mint, *long_token_mint_info.key)?;
    assert_keys_equal(short_escrow_account.mint, *short_token_mint_info.key)?;
    assert_keys_equal(*long_escrow_account_info.key, long_escrow_key)?;
    assert_keys_equal(*short_escrow_account_info.key, short_escrow_key)?;

    create_or_allocate_account_raw(
        *program_id,
        long_escrow_account_info,
        rent_info,
        system_account_info,
        update_authority_info,
        Account::LEN,
        long_escrow_seeds,
    )?;
    spl_initialize(
        &token_program_info,
        &long_escrow_account_info,
        &long_escrow_mint_info,
        &update_authority_info,
        &rent_info,
    )?;

    create_or_allocate_account_raw(
        *program_id,
        short_escrow_account_info,
        rent_info,
        system_account_info,
        update_authority_info,
        Account::LEN,
        short_escrow_seeds,
    )?;
    spl_initialize(
        &token_program_info,
        &short_escrow_account_info,
        &short_escrow_mint_info,
        &update_authority_info,
        &rent_info,
    )?;

    // Transfer ownership of the escrow accounts to a PDA
    let (escrow_owner_key, _) = Pubkey::find_program_address(
        &[
            long_token_mint_info.key.as_ref(),
            short_token_mint_info.key.as_ref(),
            token_program_info.key.as_ref(),
            program_id.as_ref(),
        ],
        program_id,
    );
    spl_set_authority(
        token_program_info,
        long_escrow_account_info,
        Some(escrow_owner_key),
        AuthorityType::AccountOwner,
        update_authority_info,
    )?;
    spl_set_authority(
        token_program_info,
        short_escrow_account_info,
        Some(escrow_owner_key),
        AuthorityType::AccountOwner,
        update_authority_info,
    )?;

    create_or_allocate_account_raw(
        *program_id,
        pool_account_info,
        rent_info,
        system_account_info,
        update_authority_info,
        BettingPool::LEN,
        long_escrow_seeds,
    )?;

    let mut betting_pool = BettingPool::try_from_slice(&pool_account_info.data.borrow_mut())?;
    betting_pool.circulation = 0;
    betting_pool.settled = false;
    betting_pool.long_mint_account_pubkey = *long_token_mint_info.key;
    betting_pool.short_mint_account_pubkey = *short_token_mint_info.key;
    betting_pool.long_escrow_mint_account_pubkey = *long_escrow_mint_info.key;
    betting_pool.short_escrow_mint_account_pubkey = *short_escrow_mint_info.key;
    betting_pool.long_escrow_account_pubkey = *long_escrow_account_info.key;
    betting_pool.short_escrow_account_pubkey = *short_escrow_account_info.key;
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;

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
    let buyer_long_token_account_info = next_account_info(account_info_iter)?;
    let buyer_short_token_account_info = next_account_info(account_info_iter)?;
    let seller_long_token_account_info = next_account_info(account_info_iter)?;
    let seller_short_token_account_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let escrow_authority_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    // Unpack accounts
    let long_token_mint: Mint = assert_initialized(long_token_mint_info)?;
    let short_token_mint: Mint = assert_initialized(short_token_mint_info)?;
    let buyer_long_token_account: Account = assert_initialized(buyer_long_token_account_info)?;
    let buyer_short_token_account: Account = assert_initialized(buyer_short_token_account_info)?;
    let seller_long_token_account: Account = assert_initialized(seller_long_token_account_info)?;
    let seller_short_token_account: Account = assert_initialized(seller_short_token_account_info)?;
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
    if betting_pool.settled {
        return Err(BettingPoolError::AlreadySettled.into());
    }
    assert_mint_authority_matches_mint(&long_token_mint, mint_authority_info)?;
    assert_mint_authority_matches_mint(&short_token_mint, mint_authority_info)?;
    assert_owned_by(long_token_mint_info, &spl_token::id())?;
    assert_owned_by(short_token_mint_info, &spl_token::id())?;
    assert_owned_by(buyer_long_token_account_info, buyer_info.key)?;
    assert_owned_by(buyer_short_token_account_info, buyer_info.key)?;
    assert_owned_by(seller_long_token_account_info, seller_info.key)?;
    assert_owned_by(seller_short_token_account_info, seller_info.key)?;
    assert_owned_by(buyer_long_account_info, buyer_info.key)?;
    assert_owned_by(buyer_short_account_info, buyer_info.key)?;
    assert_owned_by(seller_long_account_info, seller_info.key)?;
    assert_owned_by(seller_short_account_info, seller_info.key)?;
    assert_keys_equal(escrow_owner_key, *escrow_authority_info.key)?;
    assert_keys_equal(
        *long_token_mint_info.key,
        betting_pool.long_mint_account_pubkey,
    )?;
    assert_keys_equal(
        *short_token_mint_info.key,
        betting_pool.short_mint_account_pubkey,
    )?;
    assert_keys_equal(
        *short_escrow_account_info.key,
        betting_pool.short_escrow_account_pubkey,
    )?;
    assert_keys_equal(
        *long_escrow_account_info.key,
        betting_pool.long_escrow_account_pubkey,
    )?;
    assert_keys_equal(
        buyer_long_token_account.mint,
        betting_pool.long_mint_account_pubkey,
    )?;
    assert_keys_equal(
        buyer_short_token_account.mint,
        betting_pool.short_mint_account_pubkey,
    )?;
    assert_keys_equal(
        seller_long_token_account.mint,
        betting_pool.long_mint_account_pubkey,
    )?;
    assert_keys_equal(
        seller_short_token_account.mint,
        betting_pool.short_mint_account_pubkey,
    )?;
    assert_keys_equal(
        buyer_long_account.mint,
        betting_pool.long_escrow_mint_account_pubkey,
    )?;
    assert_keys_equal(
        buyer_short_account.mint,
        betting_pool.short_escrow_mint_account_pubkey,
    )?;
    assert_keys_equal(
        seller_long_account.mint,
        betting_pool.long_escrow_mint_account_pubkey,
    )?;
    assert_keys_equal(
        seller_short_account.mint,
        betting_pool.short_escrow_mint_account_pubkey,
    )?;

    let n = size;
    let n_b = buyer_short_token_account.amount;
    let n_s = seller_long_token_account.amount;
    match [n_b >= n, n_s >= n] {
        /*
        When n is less than both n_b and n_s, this means that both buyer and seller are simply reducing their existing inventory.
        Therefore, we can just buy n long tokens and n short tokens from circulation. Both parties are also entitled to the locked up
        funds for their positions that were closed. This always results in a decrease in total circulation.
        */
        [true, true] => {
            spl_burn(
                &token_program_info,
                &buyer_short_token_account_info,
                &short_token_mint_info,
                &buyer_info,
                n,
            )?;
            spl_burn(
                &token_program_info,
                &seller_short_token_account_info,
                &long_token_mint_info,
                &seller_info,
                n,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &short_escrow_account_info,
                &buyer_info,
                &escrow_authority_info,
                n * buy_price,
                seeds,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &long_escrow_account_info,
                &seller_info,
                &escrow_authority_info,
                n * sell_price,
                seeds,
            )?;
            betting_pool.decrement_supply(n)?;
        }
        /*
        When n is greater than both n_b and n_s, this means that both buyer and seller have put on a position that is different from their
        existing position. We will first burn the tokens of representing the opposite position and then mint new tokens to ensure the buyer's
        change is +n and the seller's change is -n. Both parties are also entitled to the locked up funds for their positions that were closed.
        The net change in tokens can be calculated as follows: (-n_b - n_s + 2n - n_b - n_s) / 2 = n - n_b - n_s. If this quantity is positive, this
        means that the trade causes a net increase in the total supply of contracts in the betting pool. Otherwise, it results in a net decrease
        in total circulation.
        */
        [false, false] => {
            spl_burn(
                &token_program_info,
                &buyer_short_token_account_info,
                &short_token_mint_info,
                &buyer_info,
                n_b,
            )?;
            spl_burn(
                &token_program_info,
                &seller_short_token_account_info,
                &long_token_mint_info,
                &seller_info,
                n_s,
            )?;
            spl_mint_to(
                &token_program_info,
                &buyer_long_token_account_info,
                &long_token_mint_info,
                &mint_authority_info,
                n - n_b,
            )?;
            spl_mint_to(
                &token_program_info,
                &seller_short_token_account_info,
                &short_token_mint_info,
                &mint_authority_info,
                n - n_s,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &short_escrow_account_info,
                &buyer_short_account_info,
                &escrow_authority_info,
                n_b * buy_price,
                seeds,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &long_escrow_account_info,
                &seller_long_account_info,
                &escrow_authority_info,
                n_s * sell_price,
                seeds,
            )?;
            spl_token_transfer(
                &token_program_info,
                &buyer_long_account_info,
                &long_escrow_account_info,
                &buyer_info,
                (n - n_b) * buy_price,
            )?;
            spl_token_transfer(
                &token_program_info,
                &seller_short_account_info,
                &short_escrow_account_info,
                &seller_info,
                (n - n_s) * sell_price,
            )?;
            if n > n_b + n_s {
                betting_pool.increment_supply(n - n_b - n_s);
            } else {
                betting_pool.decrement_supply(n - n_b - n_s)?;
            }
        }
        /*
        When n is greater than n_b bust less than n_s, this means that the buyer has put on a position that is different from their
        existing position, and the seller has reduced their inventory. We will burn and mint tokens such the buyer's net change in
        position is +n and the seller's net change is -n. Both parties are also entitled to the locked up funds for their positions that were closed.
        The net change in tokens can be calculated as follows: (-n - n_s + n - n_s) / 2 = -n_s. This always results in a decrease in total
        circulation.
        */
        [false, true] => {
            spl_burn(
                &token_program_info,
                &buyer_short_token_account_info,
                &short_token_mint_info,
                &buyer_info,
                n,
            )?;
            spl_burn(
                &token_program_info,
                &seller_short_token_account_info,
                &long_token_mint_info,
                &seller_info,
                n_s,
            )?;
            spl_mint_to(
                &token_program_info,
                &seller_short_token_account_info,
                &short_token_mint_info,
                &mint_authority_info,
                n - n_s,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &short_escrow_account_info,
                &buyer_short_account_info,
                &escrow_authority_info,
                n * buy_price,
                seeds,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &long_escrow_account_info,
                &seller_long_account_info,
                &escrow_authority_info,
                n_s * sell_price,
                seeds,
            )?;
            spl_token_transfer(
                &token_program_info,
                &seller_short_account_info,
                &short_escrow_account_info,
                &seller_info,
                (n - n_s) * sell_price,
            )?;
            betting_pool.decrement_supply(n_s)?;
        }
        /*
        When n is greater than n_s bust less than n_b, this means that the seller has put on a position that is different from their
        existing position, and the buyer has reduced their inventory. We will burn and mint tokens such the buyer's net change in
        position is +n and the seller's net change is -n. Both parties are also entitled to the locked up funds for their positions that were closed.
        The net change in tokens can be calculated as follows: (-n - n_b + n - n_b) / 2 = -n_b. This always results in a decrease in total
        circulation.
        */
        [true, false] => {
            spl_burn(
                &token_program_info,
                &seller_short_token_account_info,
                &long_token_mint_info,
                &seller_info,
                n,
            )?;
            spl_burn(
                &token_program_info,
                &buyer_short_token_account_info,
                &short_token_mint_info,
                &buyer_info,
                n_b,
            )?;
            spl_mint_to(
                &token_program_info,
                &buyer_long_token_account_info,
                &long_token_mint_info,
                &mint_authority_info,
                n - n_b,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &short_escrow_account_info,
                &buyer_short_account_info,
                &escrow_authority_info,
                n_b * buy_price,
                seeds,
            )?;
            spl_token_transfer_signed(
                &token_program_info,
                &long_escrow_account_info,
                &seller_long_account_info,
                &escrow_authority_info,
                n * sell_price,
                seeds,
            )?;
            spl_token_transfer(
                &token_program_info,
                &buyer_long_account_info,
                &long_escrow_account_info,
                &buyer_info,
                (n - n_b) * buy_price,
            )?;
            betting_pool.decrement_supply(n_b)?;
        }
    }
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}

pub fn process_settle(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let winning_mint_account_info = next_account_info(account_info_iter)?;
    let update_authority_info = next_account_info(account_info_iter)?;

    let winning_mint: Mint = assert_initialized(winning_mint_account_info)?;
    let mut betting_pool = BettingPool::try_from_slice(&pool_account_info.data.borrow_mut())?;

    assert_mint_authority_matches_mint(&winning_mint, update_authority_info)?;

    if *winning_mint_account_info.key == betting_pool.long_mint_account_pubkey || *winning_mint_account_info.key == betting_pool.short_mint_account_pubkey {
        betting_pool.winning_side_pubkey = *winning_mint_account_info.key;
    } else {
        return Err(BettingPoolError::InvalidWinner.into());
    }
    betting_pool.settled = true;
    betting_pool.serialize(&mut *pool_account_info.data.borrow_mut())?;
    Ok(())
}


pub fn process_collect(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pool_account_info = next_account_info(account_info_iter)?;
    let collector_account_info = next_account_info(account_info_iter)?;
    let collector_long_token_account_info = next_account_info(account_info_iter)?;
    let collector_short_token_account_info = next_account_info(account_info_iter)?;
    let collector_long_account_info = next_account_info(account_info_iter)?;
    let collector_short_account_info = next_account_info(account_info_iter)?;
    let long_token_mint_info = next_account_info(account_info_iter)?;
    let short_token_mint_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let long_escrow_account_info = next_account_info(account_info_iter)?;
    let short_escrow_account_info = next_account_info(account_info_iter)?;
    let escrow_authority_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    let long_token_mint: Mint = assert_initialized(long_token_mint_info)?;
    let short_token_mint: Mint = assert_initialized(short_token_mint_info)?;
    let collector_long_token_account: Account = assert_initialized(collector_long_token_account_info)?;
    let collector_short_token_account: Account = assert_initialized(collector_short_token_account_info)?;
    let collector_long_account: Account = assert_initialized(collector_long_account_info)?;
    let collector_short_account: Account = assert_initialized(collector_short_account_info)?;
    let long_escrow_account: Account = assert_initialized(long_escrow_account_info)?;
    let short_escrow_account: Account = assert_initialized(short_escrow_account_info)?;
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

    if !betting_pool.settled {
        return Err(BettingPoolError::BetNotSettled.into());
    }
    assert_mint_authority_matches_mint(&long_token_mint, mint_authority_info)?;
    assert_mint_authority_matches_mint(&short_token_mint, mint_authority_info)?;
    assert_owned_by(long_token_mint_info, &spl_token::id())?;
    assert_owned_by(short_token_mint_info, &spl_token::id())?;
    assert_owned_by(collector_long_token_account_info, collector_account_info.key)?;
    assert_owned_by(collector_short_token_account_info, collector_account_info.key)?;
    assert_owned_by(collector_long_account_info, collector_account_info.key)?;
    assert_owned_by(collector_short_account_info, collector_account_info.key)?;
    assert_keys_equal(escrow_owner_key, *escrow_authority_info.key)?;
    assert_keys_equal(
        *long_token_mint_info.key,
        betting_pool.long_mint_account_pubkey,
    )?;
    assert_keys_equal(
        *short_token_mint_info.key,
        betting_pool.short_mint_account_pubkey,
    )?;
    assert_keys_equal(
        *short_escrow_account_info.key,
        betting_pool.short_escrow_account_pubkey,
    )?;
    assert_keys_equal(
        *long_escrow_account_info.key,
        betting_pool.long_escrow_account_pubkey,
    )?;
    assert_keys_equal(
        collector_long_token_account.mint,
        betting_pool.long_mint_account_pubkey,
    )?;
    assert_keys_equal(
        collector_short_token_account.mint,
        betting_pool.short_mint_account_pubkey,
    )?;
    assert_keys_equal(
        collector_long_account.mint,
        betting_pool.long_escrow_mint_account_pubkey,
    )?;
    assert_keys_equal(
        collector_short_account.mint,
        betting_pool.short_escrow_mint_account_pubkey,
    )?;


    let winner_long = collector_long_token_account.mint == betting_pool.winning_side_pubkey;
    let winner_short = collector_short_token_account.mint == betting_pool.winning_side_pubkey;
    let reward = match [winner_long, winner_short] {
        [true, false] => {collector_long_token_account.amount}
        [false, true] => {collector_short_token_account.amount}
        _ => return Err(BettingPoolError::TokenNotFoundInPool.into())
    };
    if reward > 0 {
        spl_token_transfer_signed(
            &token_program_info,
            &long_escrow_account_info,
            &collector_long_account_info,
            &escrow_authority_info,
            (reward * long_escrow_account.amount) / betting_pool.circulation,
            seeds,
        )?;
        spl_token_transfer_signed(
            &token_program_info,
            &short_escrow_account_info,
            &collector_short_account_info,
            &escrow_authority_info,
            (reward * short_escrow_account.amount) / betting_pool.circulation,
            seeds,
        )?;
        spl_burn(token_program_info, collector_long_account_info, long_token_mint_info, mint_authority_info, collector_long_token_account.amount)?;
        spl_burn(token_program_info, collector_short_account_info, short_token_mint_info, mint_authority_info, collector_short_token_account.amount)?;
        betting_pool.decrement_supply(reward)?;
    }
    Ok(())
}