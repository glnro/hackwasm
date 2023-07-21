use crate::state::Lotto;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp, Uint128};
use nois::NoisCallback;

#[cw_serde]
pub struct InstantiateMsg {
    pub manager: String,
    pub community_pool: String,
    pub nois_proxy: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Anyone can create a new lotto. This will also book a random beacon at the end of the round
    CreateLotto {
        ticket_price: Coin,
        duration_seconds: u64,
        number_of_winners: u16,
        // funded_addresses: Vec<(String, Uint128)>,
    },
    BuyTicket {
        lotto_id: u32,
    },
    //callback contains the randomness from drand (HexBinary) and job_id
    //callback should only be allowed to be called by the proxy contract
    NoisReceive {
        callback: NoisCallback,
    },
    // Withdraw all available balance to the withdrawal address for a specific denom
    WithdrawAll {
        address: String,
        denom: String,
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
    #[returns(LottosResponse)]
    Lottos { creator: String },
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
    pub winners: Option<Vec<String>>,
    pub creator: String,
}
#[cw_serde]
pub struct LottosResponse {
    /// True if expired, False if not expired
    pub lottos: Vec<Lotto>,
}

#[cw_serde]
pub struct ConfigResponse {
    /// manager if none set to info.sender.
    pub manager: String,
    /// Address of the Nois proxy contract
    pub nois_proxy: String,
}
