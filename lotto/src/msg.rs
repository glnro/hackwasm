use crate::state::Lotto;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp, Uint128};
use nois::NoisCallback;

#[cw_serde]
pub struct InstantiateMsg {
    pub manager: String,
    pub nois_proxy: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateLotto {
        deposit: Coin, // Can already book the safe randomness after this timestamp (faster)
    },
    Deposit {
        lotto_id: u32,
    },
    //callback contains the randomness from drand (HexBinary) and job_id
    //callback should only be allowed to be called by the proxy contract
    NoisReceive {
        callback: NoisCallback,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get the config state
    #[returns(ConfigResponse)]
    Config {},
    #[returns(LottoResponse)]
    Lotto { lotto_nonce: u32 },
}

// GetLotto response, can be null or Lotto
#[cw_serde]
pub struct GetLottoResponse {
    pub lotto: Option<Lotto>,
}

#[cw_serde]
pub struct LottoResponse {
    /// True if expired, False if not expired
    pub is_expired: bool,
    pub nonce: u32,
    pub deposit: Coin,
    pub balance: Uint128,
    pub depositors: Vec<String>,
    pub expiration: Timestamp, // how to set expiration
    pub winner: Option<String>,
}

#[cw_serde]
pub struct ConfigResponse {
    /// manager if none set to info.sender.
    pub manager: String,
    /// Address of the Nois proxy contract
    pub nois_proxy: String,
}
