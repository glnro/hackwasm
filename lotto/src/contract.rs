#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Timestamp, Uint128};
use schemars::schema::SingleOrVec::Vec;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONFIG_KEY, Lotto};
use crate::state::{NOIS_PROXY};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lotto";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    // validate address is correct
    let addr = deps.api.addr_validate(&info.sender.as_ref())
        .map_err(|_| ContractError::InvalidAddress)?;

    let proxy = deps.api.addr_validate(&msg.nois_proxy)
        .map_err(|_| ContractError::InvalidAddress)?;

    let cnfg = Config{
        manager_addr: addr ,
        lotto_nonce: 0,
        nois_proxy: proxy,
    };

    CONFIG.save(deps.storage, &cnfg)?;

    Ok(Response::new().add_attribute("action","instantiate")
        .add_attribute("manager", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateLotto { min_deposit } => execute_create_lotto(deps, env, info, min_deposit),
        ExecuteMsg::Deposit { lotto_id } => execute_deposit_lotto(deps, env, info, lotto_id),
        ExecuteMsg::RandomnessReceive { lotto_id } => execute_payout(deps, env, info, lotto_id),
    }
}

fn execute_create_lotto(deps: DepsMut, _env: Env, _info: MessageInfo, min_deposit: Uint128, expiration_date: Timestamp) -> Result<Response, ContractError>{
    // validate min deposit
    // validate Timestamp 

    let state = CONFIG.load(deps.storage)?;

    let lotto = Lotto{ nonce: state.lotto_nonce, min_deposit: min_deposit, deposit_amount: Uint128(0), depositors: Vec::new(), expiration: expiration_date, winner: None };

    lotto_nonce += 1;
    // save config
    unimplemented!()
}

fn execute_deposit_lotto(deps: DepsMut, _env: Env, _info: MessageInfo, lotto_id: u32) -> Result<Response, ContractError>{
    unimplemented!()
}

fn execute_payout(deps: DepsMut, _env: Env, _info: MessageInfo, lotto_id: u32) -> Result<Response, ContractError>{
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LottoStatus { lotto_id } => query_lotto_status(deps, env, info, lotto_id),
    }
}

fn query_lotto_status(deps: DepsMut, _env: Env, _info: MessageInfo, lotto_id: u32) ->  StdResult<Binary>{
    unimplemented!()
}

#[cfg(test)]
mod tests {}
