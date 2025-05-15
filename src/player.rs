use crate::Player;
use crate::StorageData;
use core::slice::IterMut;
use serde::Serialize;
use crate::error::*;
use crate::history::RoundResult;
use zkwasm_rest_convention::WithBalance;
use zkwasm_rest_convention::IndexedObject;

#[derive(Clone, Serialize, Debug)]
pub struct RoundInfo {
    pub round: u64,
    pub ratio: u64,
}

impl StorageData for RoundInfo {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let round = *u64data.next().unwrap();
        let ratio = *u64data.next().unwrap();
        RoundInfo {
            round,
            ratio,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.round);
        data.push(self.ratio);
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct PlayerData {
    pub balance: u64,
    pub round: u64,
    pub rounds: Vec<RoundInfo>,
    pub purchase: Vec<u64>,
}


impl Default for PlayerData {
    fn default() -> Self {
        Self {
            balance: 0,
            round: 0,
            rounds: vec![],
            purchase: vec![],
        }
    }
}

impl PlayerData {
    pub fn get_purchase(&self, index: u64) -> u64 {
        for p in self.purchase.iter() {
            let i = *p >> 32;
            if i == index {
                return *p & 0xffffffff;
            }
        }
        return 0
    }
    pub fn inc_purchase(&mut self, index: u64, amount: u64) {
        for p in self.purchase.iter_mut() {
            let i = *p >> 32;
            if i == index {
                *p = (i << 32) + ((*p & 0xffffffff) + amount);
                return;
            }
        }
        self.purchase.push((index << 32) + amount)
    }
    pub fn settle(&mut self, round: u64, global_round: u64) -> Result<(), u32> {
        let r = RoundResult::get_object(round).unwrap();
        for i in 0..self.rounds.len() {
            let p = self.rounds[i].clone();
            if p.round == round {
                self.inc_balance(r.data.pool * p.ratio / r.data.total);
                self.rounds.swap_remove(i);
                return Ok(())
            }
        }
        if self.round == round {
            let ratio = self.get_purchase(r.data.winner);
            self.inc_balance(r.data.pool * ratio / r.data.total);
            self.round = global_round;
            self.purchase = vec![];
            return Ok(())
        } else {
            return Err(ROUND_NO_REWARD)
        }
    }
}

impl StorageData for PlayerData {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let balance = *u64data.next().unwrap();
        let round = *u64data.next().unwrap();
        let ilength = *u64data.next().unwrap();
        let mut rounds = Vec::with_capacity(ilength as usize);
        for _ in 0..ilength {
            rounds.push(RoundInfo::from_data(u64data))
        }
        let plength = *u64data.next().unwrap();
        let mut purchase = Vec::with_capacity(plength as usize);
        for _ in 0..plength {
            purchase.push(*u64data.next().unwrap());
        }
        PlayerData {
            balance,
            round,
            rounds,
            purchase,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.balance);
        data.push(self.round);
        data.push(self.rounds.len() as u64);
        for round in self.rounds.iter() {
            round.to_data(data)
        }
        data.push(self.purchase.len() as u64);
        for p in self.purchase.iter() {
            data.push(*p)
        }
    }
}

pub type GamePlayer = Player<PlayerData>;

pub trait Owner: Sized {
    fn new(pkey: &[u64; 4]) -> Self;
    fn get(pkey: &[u64; 4]) -> Option<Self>;
}

impl Owner for GamePlayer {
    fn new(pkey: &[u64; 4]) -> Self {
        Self::new_from_pid(Self::pkey_to_pid(pkey))
    }

    fn get(pkey: &[u64; 4]) -> Option<Self> {
        Self::get_from_pid(&Self::pkey_to_pid(pkey))
    }
}

impl WithBalance for PlayerData {
    fn cost_balance(&mut self, amount: u64) -> Result<(), u32> {
        if self.balance < amount {
            Err(PLAYER_NOT_ENOUGH_BALANCE)
        } else {
            self.balance -= amount;
            Ok(())
        }
    }
    fn inc_balance(&mut self, amount: u64) {
        self.balance += amount;
    }
}
