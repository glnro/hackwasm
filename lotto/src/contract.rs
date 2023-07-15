use std::cmp::min;
use std::time::Duration;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Timestamp, Uint128};
use schemars::schema::SingleOrVec::Vec;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, CONFIG_KEY, Lotto, LOTTOS};
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

fn execute_create_lotto(deps: DepsMut, _env: Env, _info: MessageInfo, min_deposit: Coin, expiration_date: Timestamp) -> Result<Response, ContractError>{
    // TODO: validate default denom

    // validate min deposit
    if min_deposit.amount <= 0 {
        return Err(ContractError::InvalidAmount {val: "min_deposit must be larger than 0".to_string()})
    }

    // validate Timestamp

    let state = CONFIG.load(deps.storage)?;
    let id = state.lotto_nonce;
    let lotto = Lotto{ nonce: id,
        min_deposit: min_deposit,
        deposit_amount: Uint128(0),
        depositors: Vec::new(),
        expiration: expiration_date,
        winner: None };

    LOTTOS.save(deps.storage, id, &lotto)?;
    lotto_nonce += 1;
    // save config
    Ok(Response::new().add_attribute("action", "create_lotto")
        .add_attribute("next_nonce", id))
}


fn execute_deposit_lotto(deps: DepsMut, _env: Env, info: MessageInfo, lotto_id: u32) -> Result<Response, ContractError>{
    let mut lotto = LOTTOS.load(deps.storage, lotto_id)?;
    let min_dep = lotto.min_deposit;
    let mut current_dep: Uint128 = Default::default();

    if info.funds.iter().any(|coin| {
        coin.denom == min_dep.denom && coin.amount >= min_dep.amount
    }){
        current_dep = info.funds.get(0).unwrap().amount;
    }

    idx = lotto.depositors.len();
    lotto.depositors[idx-1] = info.sender;
    lotto.deposit_amount += current_dep;

    LOTTOS.save(deps.storage, lotto_id, &lotto);

    Ok(Response::new().add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.as_ref())
        .add_attribute("deposit_amount", current_dep.to_string())
        )
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
