use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, LottoResponse, QueryMsg};
use anybuf::Anybuf;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Addr, Attribute, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, QueryResponse, Response, StdResult, Uint128, WasmMsg,
};
use nois::{NoisCallback, ProxyExecuteMsg};

// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{Config, Lotto, CONFIG, LOTTOS};

const GAME_DURATION: u64 = 90;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lotto";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // validate address is correct
    let addr = deps
        .api
        .addr_validate(msg.manager.as_str())
        .map_err(|_| ContractError::InvalidAddress {})?;
    let community_pool = deps
        .api
        .addr_validate(&msg.community_pool.as_str())
        .map_err(|_| ContractError::InvalidAddress {})?;

    let proxy = deps
        .api
        .addr_validate(&msg.nois_proxy)
        .map_err(|_| ContractError::InvalidAddress {})?;

    let cnfg = Config {
        manager: addr,
        lotto_nonce: 0,
        nois_proxy: proxy,
        community_pool,
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
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateLotto { deposit } => execute_create_lotto(deps, env, info, deposit),
        ExecuteMsg::Deposit { lotto_id } => execute_deposit_lotto(deps, env, info, lotto_id),
        ExecuteMsg::NoisReceive { callback } => execute_receive(deps, env, info, callback),
    }
}

fn execute_create_lotto(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deposit: Coin,
) -> Result<Response, ContractError> {
    // validate Timestamp
    let mut config = CONFIG.load(deps.storage)?;
    let mut nonce = config.lotto_nonce;

    let expiration = env.block.time.plus_seconds(GAME_DURATION);

    let lotto = Lotto {
        nonce,
        deposit,
        balance: Uint128::new(0),
        depositors: vec![],
        expiration,
        winner: None,
    };

    LOTTOS.save(deps.storage, nonce, &lotto)?;

    let msg = WasmMsg::Execute {
        contract_addr: config.clone().nois_proxy.into_string(),
        // GetRandomnessAfter requests the randomness from the proxy after a specific timestamp
        // The job id is needed to know what randomness we are referring to upon reception in the callback.
        msg: to_binary(&ProxyExecuteMsg::GetRandomnessAfter {
            after: expiration,
            job_id: "lotto-".to_string() + nonce.to_string().as_str(),
        })?,
        // We pay here the proxy contract with whatever the depositors sends. The depositor needs to check in advance the proxy prices.
        funds: info.funds, // Just pass on all funds we got
    };
    nonce += 1;
    config.lotto_nonce = nonce;
    CONFIG.save(deps.storage, &config)?;

    // save config
    Ok(Response::new()
        .add_message(msg)
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
    let deposit = lotto.clone().deposit;

    // Not sure the best way to go about validating the coin
    validate_payment(&deposit, info.funds.as_slice())?;

    // Check if lotto is active
    if env.block.time >= lotto.expiration {
        return Err(ContractError::InvalidAddress {});
    }
    // Increment total deposit
    let balance: Coin = info
        .funds
        .iter()
        .filter(|coin| coin.denom == deposit.denom)
        .last()
        .unwrap()
        .clone();

    lotto.balance += balance.amount;
    // Add depositor address
    lotto.depositors.push(info.clone().sender);

    // Save the state
    LOTTOS.save(deps.storage, lotto_id, &lotto)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.as_ref())
        .add_attribute("new_balance", lotto.balance.to_string()))
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback: NoisCallback,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // callback should only be allowed to be called by the proxy contract
    // otherwise anyone can cut the randomness workflow and cheat the randomness by sending the randomness directly to this contract
    ensure_eq!(
        info.sender,
        config.nois_proxy,
        ContractError::UnauthorizedReceive
    );
    let randomness: [u8; 32] = callback
        .randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness)?;

    // extract lotto nonce
    let job_id = callback.job_id;
    let lotto_nonce: u32 = job_id
        .strip_prefix("lotto-")
        .expect("Strange, how is the job-id not prefixed with lotto-")
        .parse()
        .unwrap(); //Needs to check that the received nonce is a number

    // Make sure the lotto nonce is valid
    let lotto = LOTTOS.load(deps.storage, lotto_nonce)?;
    assert!(lotto.winner.is_none(), "Strange, there's already a winner");
    let depositors = lotto.depositors;

    let winner = match nois::pick(randomness, 1, depositors.clone()).first() {
        Some(wn) => wn.clone(),
        None => return Err(ContractError::NoDepositors {}),
    };

    let amount_winner = lotto.balance.mul_floor((50u128, 100)); // 50%
    let amount_community_pool = lotto.balance.mul_floor((50u128, 100)); // 50%
    let denom = lotto.deposit.clone().denom;

    let mut msgs = Vec::<CosmosMsg>::new();

    msgs.push(
        BankMsg::Send {
            to_address: winner.clone().into_string(),
            amount: vec![Coin {
                amount: amount_winner,
                denom: denom.clone(),
            }],
        }
        .into(),
    );

    msgs.push(
        BankMsg::Send {
            to_address: config.community_pool.into_string(),
            amount: vec![Coin {
                amount: amount_winner,
                denom: denom.clone(),
            }],
        }
        .into(),
    );

    // Update Lotto Data
    let new_lotto = Lotto {
        nonce: lotto_nonce,
        deposit: lotto.deposit,
        balance: lotto.balance,
        expiration: lotto.expiration,
        depositors,
        winner: Some(winner.clone()),
    };
    LOTTOS.save(deps.storage, lotto_nonce, &new_lotto)?;

    // msgs.push(CosmosMsg::Stargate {
    //     type_url: "/cosmos.distribution.v1beta1.MsgFundCommunityPool".to_string(),
    //     value: encode_msg_fund_community_pool(
    //         &Coin {
    //             denom: denom.clone(),
    //             amount: amount_community_pool,
    //         },
    //         &env.contract.address,
    //     )
    //     .into(),
    // });

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        Attribute::new("action", "receive-randomness-and-send-prize"),
        Attribute::new("winner", winner.to_string()),
        Attribute::new("job_id", job_id),
        Attribute::new(
            "winner_send_amount",
            Coin {
                amount: amount_winner,
                denom,
            }
            .to_string(),
        ), // actual send amount
    ]))
}

fn encode_msg_fund_community_pool(amount: &Coin, depositor: &Addr) -> Vec<u8> {
    // Coin: https://github.com/cosmos/cosmos-sdk/blob/v0.45.15/proto/cosmos/base/v1beta1/coin.proto#L14-L19
    // MsgFundCommunityPool: https://github.com/cosmos/cosmos-sdk/blob/v0.45.15/proto/cosmos/distribution/v1beta1/tx.proto#L69-L76
    let coin = Anybuf::new()
        .append_string(1, &amount.denom)
        .append_string(2, amount.amount.to_string());
    Anybuf::new()
        .append_message(1, &coin)
        .append_string(2, depositor)
        .into_vec()
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    let response = match msg {
        QueryMsg::Lotto { lotto_nonce } => to_binary(&query_lotto(deps, env, lotto_nonce)?)?,
        QueryMsg::Config {} => to_binary(&query_config(deps)?)?,
    };
    Ok(response)
}

fn query_lotto(deps: Deps, env: Env, nonce: u32) -> StdResult<LottoResponse> {
    let lotto = LOTTOS.load(deps.storage, nonce)?;
    let winner = lotto.winner.map(|wn| wn.to_string());
    let is_expired = env.block.time > lotto.expiration;
    Ok(LottoResponse {
        nonce: lotto.nonce,
        deposit: lotto.deposit,
        balance: lotto.balance,
        depositors: lotto.depositors.iter().map(|dep| dep.to_string()).collect(),
        winner,
        is_expired,
        expiration: lotto.expiration,
    })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        manager: config.manager.to_string(),
        nois_proxy: config.nois_proxy.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        from_binary, from_slice, Empty, HexBinary, OwnedDeps, StdError, SubMsg, Timestamp,
    };
    use serde::Deserialize;

    const CREATOR: &str = "creator";
    const PROXY_ADDRESS: &str = "the proxy of choice";
    const MANAGER: &str = "manager1";
    const COM_POOL: &str = "com_pool";

    fn instantiate_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            manager: MANAGER.to_string(),
            nois_proxy: PROXY_ADDRESS.to_string(),
            community_pool: COM_POOL.to_string(),
        };

        let info = mock_info(CREATOR, &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        deps
    }

    #[test]
    fn proper_instantiation() {
        let deps = instantiate_contract();
        let env = mock_env();

        // it worked, let's query the state
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(MANAGER, config.manager.as_str());
    }

    #[test]
    fn lotto_works() {
        let mut deps = instantiate_contract();

        let env = mock_env();

        // manager starts a lotto instance
        let info = mock_info(MANAGER, &[]);
        let msg = ExecuteMsg::CreateLotto {
            deposit: Coin {
                denom: "untrn".to_string(),
                amount: Uint128::new(1),
            },
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone deposits
        let info = mock_info(MANAGER, &[Coin::new(1, "untrn".to_string())]);
        let msg = ExecuteMsg::Deposit { lotto_id: 0 };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Receive randomness
        let msg = ExecuteMsg::NoisReceive {
            callback: NoisCallback {
                job_id: "lotto-0".to_string(),
                published: Timestamp::from_seconds(1682086395),
                randomness: HexBinary::from_hex(
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa129",
                )
                .unwrap(),
            },
        };
        let info = mock_info(PROXY_ADDRESS, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "receive-randomness-and-send-randdrop"),
                Attribute::new("address", "the proxy of choice"),
                Attribute::new(
                    "job_id",
                    "randdrop-nois1tfg9ptr84t9zshxxf5lkvrd6ej7gxjh75lztve"
                ),
                Attribute::new("participant", "nois1tfg9ptr84t9zshxxf5lkvrd6ej7gxjh75lztve"),
                Attribute::new("is_winner", true.to_string()),
                Attribute::new("merkle_amount", 4500000.to_string()),
                Attribute::new(
                    "send_amount",
                    "13500000ibc/717352A5277F3DE916E8FD6B87F4CA6A51F2FBA9CF04ABCFF2DF7202F8A8BC50"
                        .to_string()
                ),
            ]
        );
        let expected = SubMsg::new(BankMsg::Send {
            to_address: "nois1tfg9ptr84t9zshxxf5lkvrd6ej7gxjh75lztve".to_string(),
            amount: vec![Coin {
                amount: Uint128::new(13500000), // 4500000*3
                denom: "ibc/717352A5277F3DE916E8FD6B87F4CA6A51F2FBA9CF04ABCFF2DF7202F8A8BC50"
                    .to_string(),
            }],
        });
        assert_eq!(res.messages, vec![expected]);
    }
}
