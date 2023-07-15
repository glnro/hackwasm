#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use nois::NoisCallback;

// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{self, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, Lotto, CONFIG, LOTTOS};

const GAME_DURATION: u64 = 300;

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
    let addr = deps
        .api
        .addr_validate(&info.sender.as_ref())
        .map_err(|_| ContractError::InvalidAddress {})?;

    let proxy = deps
        .api
        .addr_validate(&msg.nois_proxy)
        .map_err(|_| ContractError::InvalidAddress {})?;

    let cnfg = Config {
        manager: addr,
        lotto_nonce: 0,
        nois_proxy: proxy,
    };

    CONFIG.save(deps.storage, &cnfg)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("manager", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateLotto { deposit } => execute_create_lotto(deps, env, info, deposit),
        ExecuteMsg::Deposit { lotto_id } => execute_deposit_lotto(deps, env, info, lotto_id),
        ExecuteMsg::NoisReceive { callback } => execute_payout(deps, env, info, callback),
    }
}

fn execute_create_lotto(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    deposit: Coin,
) -> Result<Response, ContractError> {
    // validate Timestamp
    let mut config = CONFIG.load(deps.storage)?;
    let mut nonce = config.lotto_nonce;

    let expiration = env.block.time.plus_seconds(GAME_DURATION);

    let lotto = Lotto {
        nonce,
        deposit,
        deposit_amount: Uint128::new(0),
        depositors: vec![],
        expiration,
        winner: None,
    };
    nonce += 1;

    LOTTOS.save(deps.storage, nonce, &lotto)?;
    config.lotto_nonce = nonce;
    CONFIG.save(deps.storage, &config)?;

    // save config
    Ok(Response::new()
        .add_attribute("action", "create_lotto")
        .add_attribute("next_nonce", nonce.to_string()))
}

fn validate_payment(deposit: &Coin, funds: &[Coin]) -> Result<(), ContractError> {
    if funds.is_empty() {
        return Err(ContractError::NoFundsProvided);
    }
    // TODO disallow participant to deposit more than one denom

    for fund in funds {
        if deposit == fund {
            return Ok(());
        }
    }
    Err(ContractError::InvalidPayment)
}

fn execute_deposit_lotto(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lotto_id: u32,
) -> Result<Response, ContractError> {
    let mut lotto = LOTTOS.load(deps.storage, lotto_id)?;
    let deposit = lotto.deposit;

    // Not sure the best way to go about validating the coin
    validate_payment(&deposit, info.funds.as_slice());

    // Check if lotto is active
    if env.block.time >= lotto.expiration {
        return Err(ContractError::InvalidAddress {});
    }
    // Increment total deposit
    let balance: &Coin = info
        .funds
        .clone()
        .iter()
        .filter(|coin| coin.denom == lotto.deposit.denom)
        .last()
        .unwrap();

    lotto.balance += balance.amount;
    // Add depositor address
    lotto.depositors.push(info.sender);

    // Save the state
    LOTTOS.save(deps.storage, lotto_id, &lotto);

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.as_ref())
        .add_attribute("new_balance", lotto.balance.to_string()))
}

fn execute_payout(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LottoStatus { lotto_id } => query_lotto_status(deps, env, info, lotto_id),
    }
}

fn query_lotto_status(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    lotto_id: u32,
) -> StdResult<Binary> {
    let lotto = LOTTOS.load(deps.storage, lotto_id)?;

    to_binary(&LottoResponse { lotto })
}

#[cfg(test)]
mod tests {}
