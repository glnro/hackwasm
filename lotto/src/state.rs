use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub nois_proxy: Addr,
    // Only the manager is able to withdraw funds from the contract
    pub manager: Addr,
    pub lotto_nonce: u64,
    pub community_pool: Addr,
    // commission that will stay in the contract
    pub protocol_commission_percent: u32,
    // commission that will got to the creator of the lotto
    pub creator_commission_percent: u32,
    // If set to true the contract is paused
    // When a contract is paused the creation of lottos is not possible
    pub is_paused: bool,
}

#[cw_serde]
pub struct Lotto {
    // The price of one ticket for the lotto
    pub ticket_price: Coin,
    // The cumulated amount of funds that the lotto has generated from depositors
    pub balance: Uint128,
    // The list of addresses that have deposited (i.e bought a lotto ticket)
    pub participants: Vec<Addr>,
    // The timestamp when the lotto round finishes
    // After this timestamp no deposits are accepted
    pub expiration: Timestamp,
    // The address of the lotto winners
    pub winners: Option<Vec<Addr>>,
    // This is the lotto id
    pub nonce: u64,
    // Creating a lotto is a permissionless transaction.
    // Anyone can create a lotto and are incentivised to do so
    pub creator: Addr,
    // How many winners will share the lotto prize
    pub number_of_winners: u32,
    // Community pool percentage
    pub community_pool_percentage: u32,
}

pub const CONFIG_KEY: &str = "config";
pub const LOTTOS_KEY: &str = "lottos";
pub const PROTOCOL_BALANCES_KEY: &str = "balances";

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const LOTTOS: Map<u64, Lotto> = Map::new(LOTTOS_KEY);
pub const PROTOCOL_BALANCES: Map<String, Uint128> = Map::new(PROTOCOL_BALANCES_KEY);
