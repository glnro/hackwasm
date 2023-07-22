use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, LottoResponse, LottosResponse, QueryMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, to_binary, Attribute, BankMsg, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    QueryResponse, Response, StdResult, Uint128, WasmMsg,
};
use cw_storage_plus::Bound;
use nois::{NoisCallback, ProxyExecuteMsg};

// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{Config, Lotto, CONFIG, LOTTOS};

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
        .addr_validate(msg.community_pool.as_str())
        .map_err(|_| ContractError::InvalidAddress {})?;

    let proxy = deps
        .api
        .addr_validate(&msg.nois_proxy)
        .map_err(|_| ContractError::InvalidAddress {})?;
    let protocol_commission_percent = msg.protocol_commission_percent;
    let creator_commission_percent = msg.creator_commission_percent;

    if protocol_commission_percent + creator_commission_percent >= 100 {
        return Err(ContractError::IncorrectRates {});
    }

    let cnfg = Config {
        manager: addr,
        lotto_nonce: 0,
        nois_proxy: proxy,
        community_pool,
        protocol_commission_percent,
        creator_commission_percent,
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
        ExecuteMsg::CreateLotto {
            ticket_price,
            duration_seconds,
            number_of_winners,
            community_pool_percentage,
        } => execute_create_lotto(
            deps,
            env,
            info,
            ticket_price,
            duration_seconds,
            number_of_winners,
            community_pool_percentage,
        ),
        ExecuteMsg::BuyTicket { lotto_id } => execute_deposit_lotto(deps, env, info, lotto_id),
        ExecuteMsg::NoisReceive { callback } => execute_receive(deps, env, info, callback),
        ExecuteMsg::SetConfig {
            nois_proxy,
            manager,
            lotto_nonce,
            community_pool,
            protocol_commission_percent,
            creator_commission_percent,
        } => execute_set_config(
            deps,
            info,
            nois_proxy,
            manager,
            lotto_nonce,
            community_pool,
            protocol_commission_percent,
            creator_commission_percent,
        ),
        ExecuteMsg::WithdrawAll { address, denom } => {
            execute_withdraw_all(deps, env, info, address, denom)
        }
    }
}

fn execute_create_lotto(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ticket_price: Coin,
    duration_seconds: u64,
    number_of_winners: u32,
    community_pool_percentage: u32,
) -> Result<Response, ContractError> {
    // validate Timestamp
    let mut config = CONFIG.load(deps.storage)?;
    let mut nonce = config.lotto_nonce;

    let expiration = env.block.time.plus_seconds(duration_seconds);

    if config.protocol_commission_percent
        + config.creator_commission_percent
        + community_pool_percentage
        >= 100
    {
        return Err(ContractError::IncorrectRates {});
    }

    let lotto = Lotto {
        nonce,
        ticket_price,
        balance: Uint128::new(0),
        participants: vec![],
        expiration,
        winners: None,
        creator: info.sender,
        number_of_winners,
        community_pool_percentage,
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

fn execute_set_config(
    deps: DepsMut,
    info: MessageInfo,
    nois_proxy: Option<String>,
    manager: Option<String>,
    lotto_nonce: Option<u64>,
    community_pool: Option<String>,
    protocol_commission_percent: Option<u32>,
    creator_commission_percent: Option<u32>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(info.sender, config.manager, ContractError::Unauthorized);

    let manager = match manager {
        Some(ma) => deps.api.addr_validate(&ma)?,
        None => config.manager,
    };
    let nois_proxy = match nois_proxy {
        Some(np) => deps.api.addr_validate(&np)?,
        None => config.nois_proxy,
    };
    let community_pool = match community_pool {
        Some(cp) => deps.api.addr_validate(&cp)?,
        None => config.community_pool,
    };
    let lotto_nonce = lotto_nonce.unwrap_or(config.lotto_nonce);
    let protocol_commission_percent =
        protocol_commission_percent.unwrap_or(config.protocol_commission_percent);
    let creator_commission_percent =
        creator_commission_percent.unwrap_or(config.creator_commission_percent);

    // TODO Check that the commissions are less than 100% and that the new values don't mess up with currently running lottos

    let new_config = Config {
        manager,
        nois_proxy,
        lotto_nonce,
        community_pool,
        protocol_commission_percent,
        creator_commission_percent,
    };

    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::default().add_attribute("action", "set_config"))
}

fn execute_deposit_lotto(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lotto_id: u64,
) -> Result<Response, ContractError> {
    if !LOTTOS.has(deps.storage, lotto_id) {
        return Err(ContractError::LottoNotFound {});
    }
    let mut lotto = LOTTOS.load(deps.storage, lotto_id)?;
    let ticket_price = lotto.clone().ticket_price;

    // Not sure the best way to go about validating the coin
    validate_payment(&ticket_price, info.funds.as_slice())?;

    // Check if lotto is active
    if env.block.time >= lotto.expiration {
        return Err(ContractError::LottoDepositStageEnded {});
    }
    // Increment total deposit
    let balance: Coin = info
        .funds
        .iter()
        .filter(|coin| coin.denom == ticket_price.denom)
        .last()
        .unwrap()
        .clone();

    lotto.balance += balance.amount;
    // Add participant address
    lotto.participants.push(info.clone().sender);

    // Save the state
    LOTTOS.save(deps.storage, lotto_id, &lotto)?;

    Ok(Response::new()
        .add_attribute("action", "participate")
        .add_attribute("sender", info.sender.as_ref())
        .add_attribute("new_balance", lotto.balance.to_string()))
}

pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
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
    let lotto_nonce: u64 = job_id
        .strip_prefix("lotto-")
        .expect("Strange, how is the job-id not prefixed with lotto-")
        .parse()
        .unwrap(); //Needs to check that the received nonce is a number

    // Make sure the lotto nonce is valid
    let lotto = LOTTOS.load(deps.storage, lotto_nonce)?;
    assert!(lotto.winners.is_none(), "Strange, there's already winners");
    let participants = lotto.participants;

    let winners = nois::pick(
        randomness,
        lotto.number_of_winners as usize,
        participants.clone(),
    );

    if winners.is_empty() {
        return Err(ContractError::NoDepositors {});
    }

    let amount_creator = lotto
        .balance
        .mul_floor((config.creator_commission_percent as u128, 100));
    let amount_protocol = lotto
        .balance
        .mul_floor((config.protocol_commission_percent as u128, 100));
    let amount_community_pool = lotto
        .balance
        .mul_floor((lotto.community_pool_percentage as u128, 100));
    let prize_amount = lotto.balance - (amount_protocol + amount_creator + amount_community_pool);
    let amount_winner = prize_amount.multiply_ratio(
        Uint128::new(1),
        Uint128::new(lotto.number_of_winners as u128),
    );

    let denom = lotto.ticket_price.clone().denom;

    let mut msgs = vec![
        // Community Pool
        BankMsg::Send {
            to_address: config.community_pool.into_string(),
            amount: vec![Coin {
                amount: amount_community_pool,
                denom: denom.clone(),
            }],
        },
        // creator
        BankMsg::Send {
            to_address: lotto.creator.clone().into_string(),
            amount: vec![Coin {
                amount: amount_creator,
                denom: denom.clone(),
            }],
        },
    ];
    for winner in winners.clone() {
        msgs.push(
            // Winner
            BankMsg::Send {
                to_address: winner.clone().into_string(),
                amount: vec![Coin {
                    amount: amount_winner,
                    denom: denom.clone(),
                }],
            },
        );
    }

    // Update Lotto Data
    let new_lotto = Lotto {
        nonce: lotto_nonce,
        ticket_price: lotto.ticket_price,
        balance: lotto.balance,
        expiration: lotto.expiration,
        participants,
        winners: Some(winners.clone()),
        creator: lotto.creator,
        number_of_winners: lotto.number_of_winners,
        community_pool_percentage: lotto.community_pool_percentage,
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

fn execute_withdraw_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_address: String,
    denom: String,
) -> Result<Response, ContractError> {
    // TODO CRITICAL! Make sure not to withdraw current deposits that have not been settled
    // Keep a state of the manager revenue

    let config = CONFIG.load(deps.storage)?;
    // check the calling address is the authorised address
    ensure_eq!(info.sender, config.manager, ContractError::Unauthorized);

    let amount = deps
        .querier
        .query_balance(env.contract.address.clone(), denom.clone())?;
    let msg = BankMsg::Send {
        to_address,
        amount: vec![amount.clone()],
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_all")
        .add_attribute("amount", amount.to_string());
    Ok(res)
}

// For chains that have a community pool module, you can use this function.
// Neutron has a community pool built as a cosmwasm contract
// fn encode_msg_fund_community_pool(amount: &Coin, depositor: &Addr) -> Vec<u8> {
//     // Coin: https://github.com/cosmos/cosmos-sdk/blob/v0.45.15/proto/cosmos/base/v1beta1/coin.proto#L14-L19
//     // MsgFundCommunityPool: https://github.com/cosmos/cosmos-sdk/blob/v0.45.15/proto/cosmos/distribution/v1beta1/tx.proto#L69-L76
//     let coin = Anybuf::new()
//         .append_string(1, &amount.denom)
//         .append_string(2, amount.amount.to_string());
//     Anybuf::new()
//         .append_message(1, &coin)
//         .append_string(2, depositor)
//         .into_vec()
// }

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    let response = match msg {
        QueryMsg::Lotto { lotto_nonce } => to_binary(&query_lotto(deps, env, lotto_nonce)?)?,
        QueryMsg::LottosDesc {
            creator: _,
            start_after,
            limit,
        } => to_binary(&query_lottos(
            deps,
            env,
            start_after,
            limit,
            Order::Descending,
        )?)?,
        QueryMsg::LottosAsc {
            creator: _,
            start_after,
            limit,
        } => to_binary(&query_lottos(
            deps,
            env,
            start_after,
            limit,
            Order::Ascending,
        )?)?,
        QueryMsg::Config {} => to_binary(&query_config(deps)?)?,
    };
    Ok(response)
}

fn query_lotto(deps: Deps, env: Env, nonce: u64) -> StdResult<LottoResponse> {
    let lotto = LOTTOS.load(deps.storage, nonce)?;
    let winners = match lotto.winners {
        Some(winners) => Some(winners.iter().map(|wn| wn.clone().into_string()).collect()),
        None => None,
    };
    let is_expired = env.block.time > lotto.expiration;
    Ok(LottoResponse {
        nonce: lotto.nonce,
        ticket_price: lotto.ticket_price,
        balance: lotto.balance,
        participants: lotto
            .participants
            .iter()
            .map(|dep| dep.to_string())
            .collect(),
        winners,
        is_expired,
        expiration: lotto.expiration,
        creator: lotto.creator.to_string(),
        number_of_winners: lotto.number_of_winners as u32,
        community_pool_percentage: lotto.community_pool_percentage,
    })
}

fn query_lottos(
    deps: Deps,
    env: Env,
    start_after: Option<u64>,
    limit: Option<u64>,
    order: Order,
) -> StdResult<LottosResponse> {
    let limit: usize = limit.unwrap_or(100) as usize;
    let (low_bound, top_bound) = match order {
        Order::Ascending => (start_after.map(Bound::exclusive), None),
        Order::Descending => (None, start_after.map(Bound::exclusive)),
    };
    let lottos: Vec<LottoResponse> = LOTTOS
        .range(deps.storage, low_bound, top_bound, order)
        .take(limit)
        .map(|c| {
            c.map(|(nonce, lotto)| {
                let winners = match lotto.winners {
                    Some(winners) => {
                        Some(winners.iter().map(|wn| wn.clone().into_string()).collect())
                    }
                    None => None,
                };
                LottoResponse {
                    ticket_price: lotto.ticket_price,
                    balance: lotto.balance,
                    participants: lotto
                        .participants
                        .iter()
                        .map(|dep| dep.to_string())
                        .collect(),
                    expiration: lotto.expiration,
                    winners,
                    nonce,
                    creator: lotto.creator.to_string(),
                    number_of_winners: lotto.number_of_winners as u32,
                    community_pool_percentage: lotto.community_pool_percentage,
                    is_expired: env.block.time > lotto.expiration,
                }
            })
        })
        .collect::<Result<_, _>>()?;
    Ok(LottosResponse { lottos })
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
    use cosmwasm_std::{from_binary, Empty, HexBinary, OwnedDeps, SubMsg, Timestamp};

    const CREATOR: &str = "creator1";
    const PROXY_ADDRESS: &str = "the proxy of choice";
    const MANAGER: &str = "manager";
    const COM_POOL: &str = "community_pool";

    fn instantiate_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            manager: MANAGER.to_string(),
            nois_proxy: PROXY_ADDRESS.to_string(),
            community_pool: COM_POOL.to_string(),
            protocol_commission_percent: 5,
            creator_commission_percent: 15,
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
    fn query_lottos_works() {
        let mut deps = instantiate_contract();
        let env = mock_env();
        // Create few lottos
        // lotto-0
        let info = mock_info(CREATOR, &[]);
        let msg = ExecuteMsg::CreateLotto {
            ticket_price: Coin {
                denom: "untrn".to_string(),
                amount: Uint128::new(100_000_000),
            },
            duration_seconds: 90,
            number_of_winners: 2,
            community_pool_percentage: 20,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        // lotto-1
        let info = mock_info(CREATOR, &[]);
        let msg = ExecuteMsg::CreateLotto {
            ticket_price: Coin {
                denom: "untrn".to_string(),
                amount: Uint128::new(100_000_000),
            },
            duration_seconds: 90,
            number_of_winners: 2,
            community_pool_percentage: 20,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        // lotto-2
        let info = mock_info(CREATOR, &[]);
        let msg = ExecuteMsg::CreateLotto {
            ticket_price: Coin {
                denom: "untrn".to_string(),
                amount: Uint128::new(100_000_000),
            },
            duration_seconds: 90,
            number_of_winners: 2,
            community_pool_percentage: 20,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let LottosResponse { lottos } = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::LottosAsc {
                    creator: "creator".to_string(),
                    start_after: None,
                    limit: Some(10),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let response_lotto_nonces = lottos.iter().map(|b| b.nonce).collect::<Vec<u64>>();
        assert_eq!(response_lotto_nonces, [0, 1, 2]);
    }
    #[test]
    fn lotto_works() {
        let mut deps = instantiate_contract();
        let env = mock_env();

        // manager starts a lotto instance
        let info = mock_info(CREATOR, &[]);
        let msg = ExecuteMsg::CreateLotto {
            ticket_price: Coin {
                denom: "untrn".to_string(),
                amount: Uint128::new(100_000_000),
            },
            duration_seconds: 90,
            number_of_winners: 2,
            community_pool_percentage: 20,
        };
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // someone deposits wrong amount
        let info = mock_info(
            "participant-1",
            &[Coin::new(50_000_000, "untrn".to_string())],
        );
        let msg = ExecuteMsg::BuyTicket { lotto_id: 0 };
        let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidPayment {});
        // someone deposits for inexistant lotto
        let info = mock_info(
            "participant-1",
            &[Coin::new(50_000_000, "untrn".to_string())],
        );
        let msg = ExecuteMsg::BuyTicket { lotto_id: 1 };
        let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::LottoNotFound {});

        // someone deposits correctly
        let msg = ExecuteMsg::BuyTicket { lotto_id: 0 };
        let info = mock_info(
            "participant-1",
            &[Coin::new(100_000_000, "untrn".to_string())],
        );
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        let info = mock_info(
            "participant-2",
            &[Coin::new(100_000_000, "untrn".to_string())],
        );
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        let info = mock_info(
            "participant-3",
            &[Coin::new(100_000_000, "untrn".to_string())],
        );
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        let info = mock_info(
            "participant-4",
            &[Coin::new(100_000_000, "untrn".to_string())],
        );
        execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
        let info = mock_info(
            "participant-5",
            &[Coin::new(100_000_000, "untrn".to_string())],
        );
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Receive randomness
        let msg = ExecuteMsg::NoisReceive {
            callback: NoisCallback {
                job_id: "lotto-0".to_string(),
                published: Timestamp::from_seconds(1682086395),
                randomness: HexBinary::from_hex(
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa115",
                )
                .unwrap(),
            },
        };
        let info = mock_info(PROXY_ADDRESS, &[]);
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "receive-randomness-and-send-prize"),
                Attribute::new("job_id", "lotto-0"),
                Attribute::new("winner_send_amount", "150000000untrn"),
            ]
        );
        let expected = vec![
            SubMsg::new(BankMsg::Send {
                to_address: "community_pool".to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(100_000000),
                    denom: "untrn".to_string(),
                }],
            }),
            SubMsg::new(BankMsg::Send {
                to_address: "creator1".to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(75_000000),
                    denom: "untrn".to_string(),
                }],
            }),
            SubMsg::new(BankMsg::Send {
                to_address: "participant-4".to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(150_000000),
                    denom: "untrn".to_string(),
                }],
            }),
            SubMsg::new(BankMsg::Send {
                to_address: "participant-5".to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(150_000000),
                    denom: "untrn".to_string(),
                }],
            }),
        ];
        assert_eq!(res.messages, expected);

        // someone tries to withdraw smart contract funds
        let info = mock_info("someone", &[]);
        let msg = ExecuteMsg::WithdrawAll {
            address: "someone".to_string(),
            denom: "untrn".to_string(),
        };
        let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // manager tries to withdraw smart contract funds
        let info = mock_info(MANAGER, &[]);
        let msg = ExecuteMsg::WithdrawAll {
            address: "manager_second_address".to_string(),
            denom: "untrn".to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "withdraw_all"),
                Attribute::new("amount", "withdraw_all"),
            ]
        );
        let expected = vec![SubMsg::new(BankMsg::Send {
            to_address: "manager_second_address".to_string(),
            amount: vec![Coin {
                amount: Uint128::new(42500000),
                denom: "untrn".to_string(),
            }],
        })];
        assert_eq!(res.messages, expected);
    }
}
