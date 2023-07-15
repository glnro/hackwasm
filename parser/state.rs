use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};

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
	pub expiration: Timestamp,
	pub winner: Option<Addr>,
}

pub const CONFIG_KEY: &str = "config";
pub const NOIS_PROXY_KEY: &str = "nois_proxy";
pub const MIN_DEP_KEY: &str = "min_dep";
pub const LOTTOS_KEY: &str = "lottos";

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const NOIS_PROXY: Item<Addr> = Item::new(NOIS_PROXY_KEY);
pub const MIN_DEP: Item<Coin> = Item::new(MIN_DEP_KEY);
pub const LOTTOS: Map<u32:Lotto> = Map::new(LOTTOS_KEY);