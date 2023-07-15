use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};
use crate::state::Lotto;

#[cw_serde]
pub struct InstantiateMsg {
    pub manager_address: String,
    pub nois_proxy: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateLotto{
        min_deposit: Coin
        // Can already book the safe randomness after this timestamp (faster)
    },
    Deposit{
        lotto_id: u32,
    },
    RandomnessReceive{
        lotto_id: u32,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    LottoStatus{
        lotto_id: u32,
    }
}

// GetLotto response, can be null or Lotto
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GetLottoResponse {
    pub lotto: Option<Lotto>,
}
