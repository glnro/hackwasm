use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
    pub manager: Addr,
    pub lotto_nonce: u32,
    pub community_pool: Addr,
}

#[cw_serde]
pub struct Lotto {
    // The price of one ticket for the lotto
    pub deposit: Coin,
    // The cumulated amount of funds that the lotto has generated from depositors
    pub balance: Uint128,
    // The list of addresses that have deposited (i.e bought a lotto ticket)
    pub depositors: Vec<Addr>,
    // The timestamp when the lotto round finishes
    // After this timestamp no deposits are accepted
    pub expiration: Timestamp,
    // The address of the lotto winner
    pub winner: Option<Addr>,
    // This is the lotto id
    pub nonce: u32,
    // Creating a lotto is a permissionless transaction.
    // Anyone can create a lotto and are incentivised to do so
    pub creator: Addr,
}

pub const CONFIG_KEY: &str = "config";
pub const LOTTOS_KEY: &str = "lottos";

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const LOTTOS: Map<u32, Lotto> = Map::new(LOTTOS_KEY);
