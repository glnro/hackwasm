module lotto {
  type Addr = str
  type GameId = int
  type Time = int

  //const %winner,%community pool

  pure val None: Addr = ""

  type Config = {
    admin: Addr,
    vault: Addr   // address to escrow deposits to --toDelete  
    //nois contract: Addr
  }

  type Game = {
    players: Set[Addr],
    amount: int, //participationAmount
    //totalAmount: int,
    winner: Addr,
    expires: Time
  }

  type ContractState = {
    config: Config,
    time: Time,
    gameIdCounter: GameId,
    games: GameId -> Game
  }

  type Result = {
    err: str,
    cs: ContractState
  }

  type MessageInfo = {
    sender: Addr
  }

  type InstantiateMsg = {
    admin: Addr,
    vault: Addr
  }

  type DepositMsg = {
    game: GameId,
    amount : int
  }

  type WinnerMsg = {
    game: GameId,
  }

  type CreateLottoMsg = { amount: int, expires: Time }

  pure def instantiate(info: MessageInfo,
                       msg: InstantiateMsg): ContractState = {
    {
      config: { admin: msg.admin, vault: msg.vault},
      time: 0,
      gameIdCounter: 0,
      games: Map()
    }
  }

  pure def execute_create_lotto(cs: ContractState, info: MessageInfo,
                                msg: CreateLottoMsg): Result = {
    if (cs.config.admin != info.sender) {
      { err: "Unauthorized", cs: cs }
    } else {
      val new_game = { players: Set(), amount: msg.amount, winner: None, expires: msg.expires + cs.time } //The expiration date is current time + expiration duration
      { err: "", cs: { ...cs,
          games: cs.games.put(cs.gameIdCounter, new_game),
          gameIdCounter: cs.gameIdCounter + 1
        }
      }
    }
  }

  pure def execute_deposit(cs: ContractState, info: MessageInfo,
                           msg: DepositMsg): Result = {
    if (not(cs.games.keys().contains(msg.game))) {
      { err: "Game does not exist", cs: cs }
    } else {
      val game = cs.games.get(msg.game)
      if (msg.amount != game.amount) {
        { err: "Wrong amount", cs: cs}
      } else {
        val updatedGame = {...game, players: game.players.union(Set(info.sender))}
        val updatedGames = cs.games.set(msg.game, updatedGame)
        // cw20-Send(info.sender, cs.config.vault, game.amount)

        { err: "", cs: {...cs, games: updatedGames} }
      }
    }
  }

  //didn't check for expiration date
  pure def execute_winner(cs: ContractState, info: MessageInfo,
                                msg: WinnerMsg): Result = {
    if (cs.config.admin != info.sender) { // Verification that the function can't be used by participants, here it's admin only
      { err: "Unauthorized", cs: cs }
    } else {
      if (not(cs.games.keys().contains(msg.game))) {
      { err: "Game does not exist", cs: cs }
      } else {
        val game = cs.games.get(msg.game)
        nondet gameWinner = game.players.oneOf() // To change with nois contract
        val updatedGame = {...game, winner: gameWinner}
        val updatedGames = cs.games.set(msg.game, updatedGame)
        { err: "", cs: {...cs, games: updatedGames} }
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
    val msg_inst = { admin: ADMIN, vault: VAULT }
    state' = instantiate(info, msg_inst)
  }

  action exec_create_lotto(sender: Addr, amount: int, expires: Time): bool = {
    val info = { sender: sender }
    val res = execute_create_lotto(state, info, {amount: amount, expires: expires})
    all {
      res.err == "",
      state' = res.cs
    }
  }

  action exec_deposit(sender: Addr, gameId: GameId, amountTx :int): bool = {
    val info = { sender: sender }
    val res = execute_deposit(state, info, {game: gameId, amount: amountTx})
    all {
      res.err == "",
      state' = res.cs
    }
  }

  action exec_winner(sender: Addr, gameId: GameId): bool = {
    val info = { sender: sender }
    val res = execute_winner(state, info, {game: gameId})
    all {
      res.err == "",
      state' = res.cs
    }
    }

  action step = {
    any {

      nondet amount = Int.oneOf()
      nondet expires = Int.oneOf()
      exec_create_lotto(ADMIN, amount, expires),

      nondet sender = USER_ADDR.oneOf()
      nondet gameId = Nat.oneOf()
      nondet amountTx = Int.oneOf()
      exec_deposit(sender, gameId, amountTx),
      
      nondet gameIdWin = Nat.oneOf() //Should be triggered by time when expiration date for any game occurs
      exec_winner(ADMIN,gameIdWin)
    }
  }

  // $ quint run --main=tests --invariant=invAllGameAmountsPos lotto.qnt
  val invAllGameAmountsPos = state.games.keys().forall(gameId =>
    state.games.get(gameId).amount > 0
  )
}