use zkwasm_rest_convention::IndexedObject;
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::WithdrawInfo;
use zkwasm_rest_abi::enforce;
use zkwasm_rest_convention::WithBalance;
use crate::history::RoundResult;
use crate::settlement::SettlementInfo;
use crate::player::GamePlayer;
use crate::player::RoundInfo;
use crate::state::GLOBAL_STATE;
use crate::error::*;

#[derive (Clone)]
pub enum Command {
    // standard activities
    Activity(Activity),
    // standard withdraw and deposit
    Withdraw(Withdraw),
    Deposit(Deposit),
    // standard player install and timer
    InstallPlayer,
    Tick,
}


pub trait CommandHandler {
    fn handle(&self, pid: &[u64; 2], nonce: u64, rand: &[u64; 4], counter: u64) -> Result<(), u32>;
}

#[derive (Clone)]
pub struct Withdraw {
    pub data: [u64; 3],
}

impl CommandHandler for Withdraw {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut player = GamePlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let balance = player.data.balance;
                let amount = self.data[0] & 0xffffffff;
                unsafe { require(balance >= amount) };
                player.data.balance -= amount;
                let withdrawinfo =
                    WithdrawInfo::new(&[self.data[0], self.data[1], self.data[2]], 0);
                SettlementInfo::append_settlement(withdrawinfo);
                player.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub struct Deposit {
    pub data: [u64; 3],
}

impl CommandHandler for Deposit {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], _counter: u64) -> Result<(), u32> {
        let mut admin = GamePlayer::get_from_pid(pid).unwrap();
        admin.check_and_inc_nonce(nonce);
        let mut player = GamePlayer::get_from_pid(&[self.data[0], self.data[1]]);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.data.balance += self.data[2];
                player.store();
                admin.store();
                Ok(())
            }
        }
    }
}

#[derive (Clone)]
pub enum Activity {
    // activities
    Buy(u64, u64),
    Settle(u64),
}


impl CommandHandler for Activity {
    fn handle(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], counter: u64) -> Result<(), u32> {
        let mut player = GamePlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                let mut state = GLOBAL_STATE.0.borrow_mut();
                match self {
                    Activity::Buy(index, amount) => {
                        enforce(*index <  26, "Index must less than 26");
                        let round = state.round;
                        let price = 100000/counter;
                        player.data.cost_balance(price)?;
                        let round_result = RoundResult::get_object(player.data.round).unwrap();
                        if round > player.data.round {
                            player.data.rounds.push(RoundInfo {
                                round: player.data.round,
                                ratio: player.data.get_purchase(round_result.data.winner)
                            });
                            player.data.round = round;
                            player.data.purchase = vec![];
                        }
                        state.cards[*index as usize] += amount;
                        state.pool += price;
                        player.data.inc_purchase(*index, *amount);

                        Ok(())
                    },
                    Activity::Settle(round) => {
                        let state = GLOBAL_STATE.0.borrow();
                        if *round < state.round {
                            player.data.settle(*round, state.round)
                        } else {
                            Err(ROUND_NOT_FINISHED)
                        }
                    }
                }
            }
        }
    }
}

pub fn decode_error(e: u32) -> &'static str {
    match e {
        ERROR_PLAYER_NOT_EXIST => "PlayerNotExist",
        ERROR_PLAYER_ALREADY_EXIST => "PlayerAlreadyExist",
        ERROR_NOT_SELECTED_PLAYER => "PlayerNotSelected",
        SELECTED_PLAYER_NOT_EXIST => "SelectedPlayerNotExist",
        PLAYER_NOT_ENOUGH_BALANCE=> "PlayerNotEnoughBalance",
        ROUND_NO_REWARD => "RoundNoReward",
        ROUND_NOT_FINISHED => "RoundNotFinished",
        _ => "Unknown",
    }
}
