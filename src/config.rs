use serde::Serialize;
lazy_static::lazy_static! {
    pub static ref ADMIN_PUBKEY: [u64; 4] = {
        let bytes = include_bytes!("./admin.pubkey");
        // Interpret the bytes as an array of u64
        let u64s = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u64, 4) };
        u64s.try_into().unwrap()
    };
}

#[derive(Serialize, Clone)]
pub struct Config {
    actions: [&'static str; 1],
    name: [&'static str; 1],
}

pub const MARKET_DEAL_DELAY: u64 = 24 * 60 * 60 / 5;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config {
        actions: ["nugget"],
        name: ["nugget"],
    };
}

impl Config {
    pub fn to_json_string() -> String {
        let meme_list: Vec<u64> = vec![];
        serde_json::to_string(&meme_list).unwrap()
    }

    // enable timer tick
    pub fn autotick() -> bool {
        true
    }
}

pub const NUGGET_INFO:u64 = 1;
pub const MARKET_INFO:u64 = 2;
