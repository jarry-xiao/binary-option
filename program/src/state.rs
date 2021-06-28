
use solana_program::{
    account_info::{
        AccountInfo,
    },
    pubkey::Pubkey,
    program_error::ProgramError,
};

use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum ParticipantStatus {
    Wait,
    Bet,
    Out,
    Leaving,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum GameStatus {
    InProgress,
    Ready,
    Completed,
    NotEnoughParticipants,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct ParticipantState {
    initialized: bool,
    key: Pubkey,
    funds: u64,
    status: ParticipantStatus,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BettingPool {
    pub escrow_account: Pubkey,
    pub tick_size: u64,
    pub capacity: u8,
    pub mint: Pubkey,
    pub update_authority: Pubkey,
    pub size: u8,
    pub leader: u8,
    pub active_index: u8,
    pub game_status: GameStatus,
    pub pot: u64,
    pub participant_state: Vec<ParticipantState>,
}

// impl ParticipantState {
//     pub const LEN: usize = 42;
// }

impl BettingPool {
    // pub const LEN: usize = 725;

    pub fn from_account_info(a: &AccountInfo) -> Result<BettingPool, ProgramError> {
        let betting_pool = BettingPool::try_from_slice(&a.data.borrow_mut())?;
        Ok(betting_pool)
    }

    pub fn initialize(&mut self, tick_size: u64, capacity: u8) {
        self.tick_size = tick_size;
        self.capacity = capacity;
        self.participant_state.reserve(capacity.into())
    }

    pub fn place_bet(&mut self, account: Pubkey, amount: u64) {
        let participant_state = &mut self.participant_state[self.active_index as usize];
        if !participant_state.initialized {
            return;
        }
        if participant_state.key != account {
            return;
        }
        if participant_state.status != ParticipantStatus::Wait {
            return;
        }
        if participant_state.funds < amount * self.tick_size {
            return;
        }
        if self.active_index == self.leader && self.game_status == GameStatus::Ready {
            self.game_status = GameStatus::InProgress; 
        } else if self.game_status != GameStatus::InProgress {
            return;
        }
        participant_state.funds -= amount * self.tick_size;
        self.pot += amount * self.tick_size;
        participant_state.status = ParticipantStatus::Bet;
        loop {
            self.active_index = (self.active_index + 1) % self.capacity;
            if self.participant_state[self.active_index as usize].initialized {
                break;
            }
        };
    }

    pub fn concede_pot(&mut self, account: Pubkey) {
        let participant_state = &mut self.participant_state[self.active_index as usize];
        if !participant_state.initialized {
            return;
        }
        if participant_state.key != account {
            return;
        }
        if participant_state.status != ParticipantStatus::Wait {
            return;
        }
        participant_state.status = ParticipantStatus::Out; 
        loop {
            self.active_index = (self.active_index + 1) % self.capacity;
            if self.participant_state[self.active_index as usize].initialized {
                break;
            }
        };
    }

    pub fn reset_pot(&mut self) {
        if self.game_status != GameStatus::Completed || self.game_status != GameStatus::NotEnoughParticipants {
            return;
        }
        if self.pot != 0 {
            return;
        }
        for i in 0..self.capacity as usize {
            let participant_state = &mut self.participant_state[i];
            if participant_state.initialized {
                if participant_state.status == ParticipantStatus::Leaving {
                    participant_state.initialized = false;
                    self.size -= 1;
                } else {
                    participant_state.status = ParticipantStatus::Wait;
                }
            }
        }
        if self.size < 1 {
            self.game_status = GameStatus::NotEnoughParticipants;
            return;
        }
        self.game_status = GameStatus::Ready;
        loop {
            self.leader = (self.leader + 1) % self.capacity;
            if self.participant_state[self.leader as usize].initialized {
                break;
            }
        };
        self.active_index = self.leader;
    }

    pub fn leave_pool(&mut self, account: Pubkey) -> Option<u64> {
        if self.size == 0 {
            return None;
        }
        let i = match self._get_index(account) {
            Some(pos) => pos,
            None => return None,
        };
        let participant_state = &mut self.participant_state[i];
        if !participant_state.initialized {
            return None;
        }
        let winnings = participant_state.funds;
        participant_state.funds = 0;
        participant_state.status = ParticipantStatus::Leaving;
        return Some(winnings);
    }

    pub fn join_pool(&mut self, account: Pubkey, position: u8, amount: u64) {
        if self.size >= self.capacity || position >= self.capacity {
            return;
        }
        if !self._get_index(account).is_none() {
            return;
        }

        let participant_state = &mut self.participant_state[position as usize];
        if participant_state.initialized {
            return;
        }
        participant_state.initialized = true;
        participant_state.key = account;
        participant_state.funds = amount * self.tick_size;
        participant_state.status = ParticipantStatus::Out;
        self.size += 1;
        if self.size == 1 {
            self.leader = position;
        }
    }

    fn _get_index(&self, account: Pubkey) -> Option<usize> {
        for i in 0..self.capacity as usize {
            if account == self.participant_state[i].key {
                return Some(i);
            }
        }
        return None;
    }
}