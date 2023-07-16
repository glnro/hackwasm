# QuintWasm Lottery
**Ensuring Safer Smart Contract Development on Neutron through automatic contract generation from Quint: Cross Chain Neutron X Nois Lottery**

## Demos

Using formal methods, the QuintWasm project is an automatic code generation tool for correct-by-construction Cosmwasm contracts. We showcase the use of the tool for a cross-chain game we developed on Neutron. We've modeled, verified and instantiated a lottery game that leverages Nois network's randomn generation callback to choose a winner.

The lottery manager is a contract deployed on Neutron and instantiated with a reference to a Nois proxy contract deployed on Neutron. When a new lottery is created, it sends a message to the Nois proxy to book randomness after the lottery expiration time. This will queue a job on the Nois network to release a randomly generated seed a few seconds after the lottery expiration in the form of a callback to the lottery contract. The lottery contract defines an entry point for the Nois callback `NoisReceive`, which is a proxy for the lottery payout. When this callback executes, it picks a random winner from the list of depositors, and payout 50% of the balance to the winner. In an original version of this game, the remaining 50% of winnings will be sent to the community pool to fund public goods.

Development began with the formal specification of the intended behavior of the game contract so to correctly express, structure, and communicate desired functionalities. The specified model enables running simulations, test generation, and immediate identification and verification of vulnerabilities. Long term, the goal would be to further develop Quint to automatically generate a contract that would be ready for deployment. This would enable Neutron to provide a high quality environment for developers to securely build apps, offering a competitive advantage compared to other smart contracting platforms. For the purpose of this demo, we wrote a proof of concept parser that takes the model definition in quint and generate the contract's state definition in Rust. In future, the Quint team could develop a DSL plugin/annotations for Cosmwasm to bring this vision to fruition.

### Authors
[@glnro](https://github.com/glnro) [@kaisbaccour](https://github.com/kaisbaccour) [@tesnimab](https://github.com/tesnimab) [@thpani](https://github.com/thpani)

[Quint](https://github.com/informalsystems/quint)

### Contract
[Neutron Contract](https://neutron.celat.one/testnet/contracts/neutron1tw9sg9e4l09l5rjglf4qfvcft470ljk5grdq3luagysyk83nzfusw2sxgq)

### Lottery Demo
[Video](https://www.youtube.com/watch?v=c-AlBQQgdKo)

### Quint Spec -> Cosmwasm Contract Conversion Demo
[Video](https://www.youtube.com/watch?v=CXgG_YAvZw0)

### Quint Model Demo
[Video](https://www.youtube.com/watch?v=xaDi2vByLvk)
