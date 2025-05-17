use crate::config::ADMIN_PUBKEY;
use crate::player::{Owner, GamePlayer};
use crate::settlement::SettlementInfo;
use crate::Player;
use core::slice::IterMut;
use serde::Serialize;
use std::cell::RefCell;
use zkwasm_rest_abi::MERKLE_MAP;
use zkwasm_rust_sdk::require;
use zkwasm_rest_abi::enforce;
use crate::command::Command;
use crate::command::Activity;
use crate::command::Deposit;
use crate::command::Withdraw;
use crate::command::CommandHandler;
use crate::history::RoundResult;
use zkwasm_rest_convention::IndexedObject;
use zkwasm_rest_convention::clear_events;
use crate::error::*;


#[derive(Serialize)]
pub struct GlobalState {
    pub round: u64,
    pub counter: u64,
    pub pool: u64,
    pub cards: Vec<u64>,
}



#[derive(Serialize)]
pub struct QueryState {
    round: u64,
    counter: u64,
    pool: u64,
    cards: Vec<u64>,
}

const TICK: u64 = 0;
const INSTALL_PLAYER: u64 = 1;
const WITHDRAW: u64 = 2;
const DEPOSIT: u64 = 3;
const BUY_CARD: u64 = 4;
const CLAIM_REWARD: u64 = 5;
const SETTLE: u64 = 6;

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            round: 0,
            counter: 0,
            pool: 0,
            cards: vec![],
        }
    }

    pub fn snapshot() -> String {
        let round = GLOBAL_STATE.0.borrow().round;
        let counter = GLOBAL_STATE.0.borrow().counter;
        let pool = GLOBAL_STATE.0.borrow().pool;
        let cards = GLOBAL_STATE.0.borrow().cards.clone();
        serde_json::to_string(&QueryState { counter, round, pool, cards}).unwrap()
    }

    pub fn get_state(pid: Vec<u64>) -> String {
        let player = GamePlayer::get(&pid.try_into().unwrap());
        serde_json::to_string(&player).unwrap()
    }

    pub fn preempt() -> bool {
        let state = GLOBAL_STATE.0.borrow_mut();
        if state.counter == 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_result(&self) -> RoundResult {
        let mut min = 0;
        let mut idx = 0;
        for i in 0..self.cards.len() {
          if (self.cards[i] < min || min == 0) && self.cards[i] != 0 {
              min = self.cards[i];
              idx = i;
          }
        }
        return RoundResult {
            pool: self.pool,
            total: min,
            winner: idx as u64,
        }
    }

    pub fn flush_settlement() -> Vec<u8> {
        SettlementInfo::flush_settlement()
    }

    pub fn rand_seed() -> u64 {
        0
    }

    pub fn store_into_kvpair(&self) {
        let mut v = vec![];
        v.push(self.round);
        let kvpair = unsafe { &mut MERKLE_MAP };
        kvpair.set(&[0, 0, 0, 0], v.as_slice());
    }

    pub fn fetch(&mut self) {
        let kvpair = unsafe { &mut MERKLE_MAP };
        let mut data = kvpair.get(&[0, 0, 0, 0]);
        if !data.is_empty() {
            let mut u64data = data.iter_mut();
            let round = *u64data.next().unwrap();
            self.round = round;
        }
    }

    pub fn store() {
        GLOBAL_STATE.0.borrow_mut().store_into_kvpair();
    }

    pub fn initialize() {
        let mut s = GLOBAL_STATE.0.borrow_mut();
        s.fetch();
        s.round += 1;
        s.counter = 50;
        s.cards = [0;26].to_vec();
    }

    pub fn get_counter() -> u64 {
        GLOBAL_STATE.0.borrow().counter
    }
}

pub struct SafeState(pub RefCell<GlobalState>);
unsafe impl Sync for SafeState {}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATE: SafeState = SafeState(RefCell::new(GlobalState::new()));
}

pub struct Transaction {
    command: Command,
    nonce: u64,
}

impl Transaction {
    pub fn decode_error(e: u32) -> &'static str {
        crate::command::decode_error(e)
    }

    pub fn decode(params: &[u64]) -> Self {
        let command = params[0] & 0xff;
        let nonce = params[0] >> 16;
        //zkwasm_rust_sdk::dbg!("command is {}\n", command); // only token index 0 is supported
        //zkwasm_rust_sdk::dbg!("nonce is {}\n", nonce); // only token index 0 is supported
        let command = if command == WITHDRAW {
            Command::Withdraw (Withdraw {
                data: [params[2], params[3], params[4]]
            })
        } else if command == DEPOSIT {
            enforce(params[3] == 0, "check deposit index"); // only token index 0 is supported
            Command::Deposit (Deposit {
                data: [params[1], params[2], params[4]]
            })
        } else if command == INSTALL_PLAYER {
            Command::InstallPlayer
        } else if command == BUY_CARD {
            Command::Activity (Activity::Buy(params[1], params[2]))
        } else if command == CLAIM_REWARD {
            Command::Activity (Activity::Claim(params[1]))
        } else if command == SETTLE {
            Command::Activity (Activity::Settle)
        } else {
            unsafe {zkwasm_rust_sdk::require(command == TICK)};
            Command::Tick
        };
        Transaction {
            command,
            nonce,
        }
    }

    pub fn create_player(&self, pkey: &[u64; 4]) -> Result<(), u32> {
        let round = GLOBAL_STATE.0.borrow().round;
        let player = GamePlayer::get(pkey);
        match player {
            Some(_) => Err(ERROR_PLAYER_ALREADY_EXIST),
            None => {
                let mut player = Player::new(pkey);
                player.data.balance = 100000;
                player.data.round = round;
                player.store();
                Ok(())
            }
        }
    }

    pub fn tick(&self) {
        let mut s = GLOBAL_STATE.0.borrow_mut();
        enforce(s.counter > 0, "counter must large than 1");
        s.counter -= 1;
        if s.counter == 0 {
            let result = s.get_result();
            let r = RoundResult::new_object(result, s.round);
            zkwasm_rust_sdk::dbg!("store ... {}\n", {s.round});
            r.store();
            RoundResult::emit_event(s.round, &r.data);
        }
    }

    pub fn process(&self, pkey: &[u64; 4], rand: &[u64; 4]) -> Vec<u64> {
        let pid = GamePlayer::pkey_to_pid(&pkey);
        let counter = GLOBAL_STATE.0.borrow().counter;
        let e = match &self.command {
            Command::Tick => {
                unsafe { require(*pkey == *ADMIN_PUBKEY) };
                self.tick();
                0
            },
            Command::InstallPlayer => self.create_player(pkey)
                .map_or_else(|e| e, |_| 0),
            Command::Withdraw(cmd) => cmd.handle(&pid, self.nonce, rand, counter)
                .map_or_else(|e| e, |_| 0),
            Command::Activity(cmd) => cmd.handle(&pid, self.nonce, rand, counter)
                .map_or_else(|e| e, |_| 0),
            Command::Deposit(cmd) => {
                unsafe { require(*pkey == *ADMIN_PUBKEY) };
                cmd.handle(&pid, self.nonce, rand, counter)
                    .map_or_else(|e| e, |_| 0)
            },
        };
        let round = GLOBAL_STATE.0.borrow().round;
        zkwasm_rust_sdk::dbg!("events before {:?}\n", {unsafe {&zkwasm_rest_convention::EVENTS}});
        let events = clear_events(vec![e as u64, round]);
        zkwasm_rust_sdk::dbg!("events {:?}\n", {unsafe {&zkwasm_rest_convention::EVENTS}});
        zkwasm_rust_sdk::dbg!("events {:?}\n", {&events});
        events
    }
}
