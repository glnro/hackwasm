use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

// Initialize a contract with the admin address and lotto id generator nonce
#[cw_serde]
pub struct Config {
    pub manager: Addr,
    pub lotto_nonce: u32,
    pub nois_proxy: Addr,
}

#[cw_serde]
pub struct Lotto {
    pub nonce: u32,
    pub deposit: Coin,
    pub balance: Uint128,
    pub depositors: Vec<Addr>,
    pub expiration: Timestamp, // how to set expiration
    pub winner: Option<Addr>,
}

pub const CONFIG_KEY: &str = "config";
pub const LOTTOS_KEY: &str = "lottos";

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const LOTTOS: Map<u32, Lotto> = Map::new(LOTTOS_KEY);
