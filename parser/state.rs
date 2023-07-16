use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
	pub manager: Addr,
	pub lotto_nonce: u32,
	pub nois_proxy: Addr,
}

#[cw_serde]
pub struct Lotto {
	pub deposit: Coin,
	pub balance: Uint128,
	pub depositors: Vec<Addr>,
	pub expiration: Timestamp,
	pub winner: Option<Addr>,
	pub nonce: u32,
}

pub const CONFIG_KEY: &str = "config";
pub const LOTTOS_KEY: &str = "lottos";

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const LOTTOS: Map<u32,Lotto> = Map::new(LOTTOS_KEY);