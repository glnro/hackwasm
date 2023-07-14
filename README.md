## HackWasm

### Overview
Convert a Quint Spec for a Defi Game into a wasm contract

Lotto is a simple game contract where an administrator can create time bound lotteries 

### Contract Spec

#### Types
```
Lotto {
    
}
```

#### State
```
admin_address: Addr
lotto_nonce: int
lottos: Map<> //lotto id to Lotto
```

#### Messages

**Initialization**

**Execution**

**Query**