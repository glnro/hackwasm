module lotto {
  type Addr = str
  type GameId = int
  type Timestamp = int
  type U32 = int
  type U128 = int
  type Coin = str

  pure val None: Addr = ""

  type ss_Config = {
    manager: Addr,
    lotto_nonce: U32,
    nois_proxy: Addr
  }

  type ss_Lotto = {
    nonce: U32,
    deposit: Coin,
    balance: U128,
    depositors: Set[Addr],
    expiration: Timestamp,
    o_winner: Addr
  }

  type SS_ContractState = {
    config: Config,
    time: Timestamp,
    lottos: U32 -> lotto
  }

  type Result = {
    err: str,
    cs: ContractState
  }

  type MessageInfo = {
    sender: Addr
  }

  type InstantiateMsg = {
    manager: Addr,
    vault: Addr
  }

  type DepositMsg = {
    lotto: GameId,
    deposit : int
  }

  type WinnerMsg = {
    lotto: GameId,
  }

  type CreateLottoMsg = { 
    deposit: int, 
    expiration: Timestamp
  }

  pure def instantiate(info: MessageInfo,
                       msg: InstantiateMsg): ContractState = {
    {
      config: { manager: msg.manager, lotto_nonce: 0, nois_proxy: "nois" },
      time: 0,
      lottos: Map()
    }
  }

  pure def execute_create_lotto(cs: ContractState, info: MessageInfo,
                                msg: CreateLottoMsg): Result = {
    if (cs.config.manager != info.sender) {
     { err: "Unauthorized", cs: cs }
    } else {
     val nonce_lotto = cs.config.lotto_nonce
     val new_lotto = { nonce: nonce_lotto, deposit: msg.deposit, depositors: Set(), winner: None, expiration: msg.expiration + cs.time } //The expiration date is current time + expiration duration
      { err: "", cs: { ...cs,
         lottos: cs.lottos.put(nonce_lotto, new_lotto),
         config: { manager: cs.config.manager , lotto_nonce: nonce_lotto+1, nois_proxy: "nois" }
       }
      }
    }
  }

  pure def execute_deposit(cs: ContractState, info: MessageInfo,
                           msg: DepositMsg): Result = {
    if (not(cs.lottos.keys().contains(msg.lotto))) {
      { err: "Lotto does not exist", cs: cs }
    } else {
      val lotto = cs.lottos.get(msg.lotto)
      if (msg.deposit != lotto.deposit) {
        { err: "Wrong deposit amount", cs: cs}
      } else {
        val updatedLotto = {...lotto, depositors: lotto.depositors.union(Set(info.sender))}
        val updatedLottos = cs.lottos.set(msg.lotto, updatedLotto)
        // cw20-Send(info.sender, cs.config.vault, lotto.amount)

        { err: "", cs: {...cs, lottos: updatedLottos} }
      }
    }
  }

  //didn't check for expiration date
  pure def execute_winner(cs: ContractState, info: MessageInfo,
                                msg: WinnerMsg): Result = {
    if (cs.config.manager != info.sender) { // Verification that the function can't be used by participants, here it's admin only
      { err: "Unauthorized", cs: cs }
    } else {
      if (not(cs.lottos.keys().contains(msg.lotto))) {
      { err: "Lotto does not exist", cs: cs }
      } else {
        val lotto = cs.lottos.get(msg.lotto)
        nondet lottoWinner = lotto.depositors.oneOf() // To change with nois contract
        val updatedLotto = {...lotto, winner: lottoWinner}
        val updatedLottos = cs.lottos.set(msg.lotto, updatedLotto)
        { err: "", cs: {...cs, lottos: updatedLottos} }
     }
    }
  }
}

module tests {
  import lotto.* 

  var state: ContractState 

  pure val ADMIN: Addr = "admin"
  pure val VAULT: Addr = "vault"
  pure val USER_ADDR = Set("alice", "bob", "charlie", "eve")

  // $ quint -r lotto.qnt::tests
  // >>> init
  // >>> state
  action init = {
    val info = { sender: ADMIN }
    val msg_inst = { manager: ADMIN, vault: VAULT }
    state' = instantiate(info, msg_inst)
  }

  action exec_create_lotto(sender: Addr, deposit: int, expiration: Timestamp): bool = {
    val info = { sender: sender }
    val res = execute_create_lotto(state, info, {deposit: deposit, expiration: expiration})
    all {
      res.err == "",
      state' = res.cs
    }
  }

  action exec_deposit(sender: Addr, nonce: GameId, amountTx :int): bool = {
    val info = { sender: sender }
    val res = execute_deposit(state, info, {lotto: nonce, deposit: amountTx})
    all {
      res.err == "",
      state' = res.cs
    }
  }

  action exec_winner(sender: Addr, nonce: GameId): bool = {
    val info = { sender: sender }
    val res = execute_winner(state, info, {lotto: nonce})
    all {
      res.err == "",
      state' = res.cs
    }
  }

  action step = {
    any {

      nondet deposit = Int.oneOf()
      nondet expiration = Int.oneOf()
      exec_create_lotto(ADMIN, deposit, expiration),

      nondet sender = USER_ADDR.oneOf()
      nondet nonce = Nat.oneOf()
      nondet deposit = Int.oneOf()
      exec_deposit(sender, nonce, deposit),
      
      nondet lottoWin = Nat.oneOf() //Should be triggered by time when expiration date for any game occurs
      exec_winner(ADMIN,lottoWin)
    }
  }

  // $ quint run --main=tests --invariant=invAllGameAmountsPos lotto.qnt
  val invAllGameAmountsPos = state.lottos.keys().forall(gameId =>
    state.lottos.get(gameId).deposit > 0
  )
}