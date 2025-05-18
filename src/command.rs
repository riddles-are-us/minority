use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_abi::enforce;
use zkwasm_rest_convention::WithBalance;
use zkwasm_rest_convention::{SubCommand, CommandHandler};
use crate::history::RoundResult;
use crate::player::GamePlayer;
use crate::player::RoundInfo;
use crate::state::GLOBAL_STATE;
use crate::error::*;

#[derive (Clone)]
pub enum Activity {
    // activities
    Buy(u64, u64),
    Claim(u64),
    Settle,
}

const BUY_CARD: u64 = 4;
const CLAIM_REWARD: u64 = 5;
const SETTLE: u64 = 6;

impl SubCommand for Activity {
    fn decode(command: u64, params: &[u64]) -> Option<Self> {
        if command == BUY_CARD {
            Some(Activity::Buy(params[0], params[1]))
        } else if command == CLAIM_REWARD {
            Some(Activity::Claim(params[9]))
        } else if command == SETTLE {
            Some(Activity::Settle)
        } else {
            None
        }
    }
}


impl CommandHandler for Activity {
    fn handle<PlayerData>(&self, pid: &[u64; 2], nonce: u64, _rand: &[u64; 4], counter: u64) -> Result<(), u32> {
        let mut player = GamePlayer::get_from_pid(pid);
        match player.as_mut() {
            None => Err(ERROR_PLAYER_NOT_EXIST),
            Some(player) => {
                player.check_and_inc_nonce(nonce);
                match self {
                    Activity::Buy(index, amount) => {
                        enforce(*index <  26, "Index must less than 26");
                        let price = 1000/(1 + (counter+1).ilog2() as u64);
                        player.data.cost_balance(price)?;
                        let mut state = GLOBAL_STATE.0.borrow_mut();
                        let round = state.round;
                        if round > player.data.round {
                            let round_result = RoundResult::get_object(player.data.round).unwrap();
                            let ratio = player.data.get_purchase(round_result.data.winner);
                            if ratio != 0 {
                                player.data.rounds.push(RoundInfo {
                                    round: player.data.round,
                                    ratio
                                });
                            }
                            player.data.round = round;
                            player.data.purchase = vec![];
                        }
                        state.cards[*index as usize] += amount;
                        state.pool += price;
                        player.data.inc_purchase(*index, *amount);
                        player.store();
                        Ok(())
                    },
                    Activity::Claim(round) => {
                        let state = GLOBAL_STATE.0.borrow();
                        zkwasm_rust_sdk::dbg!("claim reward of round {}\n", {*round});
                        if *round < state.round {
                            player.data.settle(*round, state.round)?;
                            player.store();
                            Ok(())
                        } else {
                            Err(ROUND_NOT_FINISHED)
                        }
                    }
                    Activity::Settle => {
                        let state = GLOBAL_STATE.0.borrow();
                        let round = state.round;
                        zkwasm_rust_sdk::dbg!("settle {}\n", {round});
                        if round > player.data.round {
                            let round_result = RoundResult::get_object(player.data.round).unwrap();
                            let ratio = player.data.get_purchase(round_result.data.winner);
                            if ratio != 0 {
                                player.data.rounds.push(RoundInfo {
                                    round: player.data.round,
                                    ratio
                                });
                            }
                            player.data.round = round;
                            player.data.purchase = vec![];
                        }
                        player.store();
                        Ok(())
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
