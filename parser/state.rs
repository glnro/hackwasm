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
	pub deposit: Coin,
	pub balance: Uint128,
	pub depositors: Vec<Addr>,
	pub expiration: Timestamp,
	pub o_winner: Addr,
	pub nonce: u32,
}
