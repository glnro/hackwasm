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

Creating the IBC channel for Nois and instantiate the Nois proxy on Neutron
[link](https://docs.nois.network/integrate_nois_to_your_chain.html)