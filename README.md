## HackWasm

### Overview
Convert a Quint Spec for a Defi Game into a wasm contract

Lotto is a simple game contract where an administrator can create time bound lotteries 

### Contract Spec
[Quint Spec](./quint/lotto.qnt)

#### Types
```
Config {
    manager: Addr,
	lotto_nonce: u32,
	nois_proxy: Addr,
}

Lotto {
    nonce: u32,
    deposit: Coin,
    balance: Uint128,
    depositors: Vec<Addr>,
    expiration: Timestamp,
    winner: Option<Addr>,
}
```

#### State
```
CONFIG: Config
LOTTOS: Map<Nonce, Lotto>
```

#### Messages

**Initialization**
```
InstantiateMsg {
    manager: String,
    nois_proxy: String,
}
```

**Execution**
```
CreateLotto {
    deposit: Coin
}

Deposit {
    lotto_id: u32,
}

NoisReceive {
    callback: NoisCallback,
}
```

**Query**
```
Config {}

Lotto { 
    lotto_nonce: u32 
}
```

<hr>

## Demo

### Compile Contract
```sh
cd lotto/
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked --lib
```

### Deploy
```sh
export ADDR=neutron1zzpvtlt9wth347wr9sdv9944zpv02j9wkkkhlk
export MNEM="cliff chalk grow bunker great grain option mass can bronze income layer license bracket theme vibrant actress predict invest defense merit tongue lecture home"
echo $MNEM | neutrond keys add deployment-key --recover  --keyring-backend test
neutrond q bank balances $(neutrond keys show deployment-key --keyring-backend test -a) --node=https://neutron-testnet-rpc.polkachu.com:443 
neutrond q bank balances $(neutrond keys show bob --keyring-backend test -a) --node=https://neutron-testnet-rpc.polkachu.com:443 
#  Request funds from discord faucet 
neutrond tx wasm store target/wasm32-unknown-unknown/release/lotto.wasm  --from deployment-key --chain-id=pion-1  --gas-prices 0.025untrn --gas=auto --gas-adjustment 1.4   --broadcast-mode=block --node=https://neutron-testnet-rpc.polkachu.com:443 --keyring-backend test -y
```

### Play

**Config**
```sh
export LOTTO_CONTRACT=neutron13qtdl7xxzu459k936qupxtwr22hyk7xx9sqwrn6yu0e530l2y0ns6a4utc
```


**Create Lotto**
```
neutrond tx wasm execute $LOTTO_CONTRACT '{"create_lotto":{"deposit":{"amount":"100","denom":"untrn"}}}' --amount 1untrn --from deployment-key --chain-id=pion-1  --gas-prices 0.025untrn --gas=auto --gas-adjustment 1.4   --broadcast-mode=block --node=https://rpc-palvus.pion-1.ntrn.tech:443 --keyring-backend test -y
```

**Query Lotto**
```
neutrond query wasm contract-state smart $LOTTO_CONTRACT  '{"lotto":{"lotto_nonce":9}}'  --node=https://rpc-palvus.pion-1.ntrn.tech:443 --chain-id=pion-1
```

**Deposit Lotto**
```
neutrond tx wasm execute $LOTTO_CONTRACT  '{"deposit":{"lotto_id":9}}' --amount 100untrn --from deployment-key --chain-id=pion-1  --gas-prices 0.025untrn --gas=auto --gas-adjustment 1.4   --broadcast-mode=block --node=https://rpc-palvus.pion-1.ntrn.tech:443 --keyring-backend test -y
neutrond tx wasm execute $LOTTO_CONTRACT  '{"deposit":{"lotto_id":9}}' --amount 100000untrn --from bob --chain-id=pion-1  --gas-prices 0.025untrn --gas=auto --gas-adjustment 1.4   --broadcast-mode=block --node=https://rpc-palvus.pion-1.ntrn.tech:443 --keyring-backend test -y
```

[Nois Proxy Contract](https://neutron.celat.one/testnet/contracts/neutron1tw9sg9e4l09l5rjglf4qfvcft470ljk5grdq3luagysyk83nzfusw2sxgq)

Creating the IBC channel for Nois and instantiate the Nois proxy on Neutron
[link](https://docs.nois.network/integrate_nois_to_your_chain.html)