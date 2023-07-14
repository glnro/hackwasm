use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub manager_address: String,
    pub nois_proxy: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateLotto{
        min_deposit: Uint128
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
