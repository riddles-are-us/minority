use std::{ops::BitXor, slice::IterMut};
use serde::Serialize;
use zkwasm_rest_abi::StorageData;
use zkwasm_rest_convention::IndexedObject;

#[derive(Serialize)]
pub struct RoundResult {
    pub winner: u64,
    pub pool: u64,
    pub total: u64, // total tickets
}

impl StorageData for RoundResult {
    fn from_data(u64data: &mut IterMut<u64>) -> Self {
        let winner = *u64data.next().unwrap();
        let pool = *u64data.next().unwrap();
        let total = *u64data.next().unwrap();
        RoundResult {
            winner,
            pool,
            total,
        }
    }
    fn to_data(&self, data: &mut Vec<u64>) {
        data.push(self.winner);
        data.push(self.pool);
        data.push(self.total);
    }
}



impl IndexedObject<RoundResult> for RoundResult {
    const PREFIX: u64 = 0x1ee1;
    const POSTFIX: u64 = 0xfee1;
    const EVENT_NAME: u64 = 0x02;
}
