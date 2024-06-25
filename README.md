# Nominated Proof of Stake (NPOS) Substrate-based Chain Tutorial
Substrate node template is a node program for quickly developing Substrate application chains. It has a built-in **Proof of Authority (PoA)** consensus algorithm. The block generation algorithm is Aura , which means that the nodes and order of block generation are fixed. An online public blockchain application needs to achieve a certain degree of decentralization and ensure security through random block generation. Substrate's built-in Proof of Stake (PoS) consensus algorithm is designed to meet such needs.
## Create a new Parachain project with `pop-cli`
You can learn more about `pop-cli` here: https://github.com/r0gue-io/pop-cli
- Install the `pop-cli`: cargo install --locked --git https://github.com/r0gue-io/pop-cli
- Create a new parachain boilerplate code with `pop new parachain substrate-npos`
- Build your newly created local parachain with `cd substrate-npos && pop run parachain`
- Run your local relaychain network using Zombinet with `pop up parachain -f ./zombienet.toml`
Now you are ready to kickstart working on the later steps.

## Overview of Additional Modules

| Pallet Name | Functionality |
| ---- | ---- |
| Staking | used to manage verification, nomination, cancellation of verification, nomination, verifier score statistics and reward distribution, etc.; |
| Session | Manage the session keys of the validator (Session Keys), control the length and rotation of the Session; |
| Authorship | Used in runtime to track the current block producer and uncles. The staking module uses this information to count the points used for rewards; |
| Utility | Implements the auxiliary function of sending transactions in batches, which is needed by validators to withdraw rewards through front-end apps. |
| [Offences](https://docs.rs/pallet-offences/latest/src/pallet_offences/lib.rs.html#51) | This pallet simply defined storage value and Pallet dispatchable functions to store and track the reported offences. |
| [Historical](https://docs.rs/pallet-session/latest/pallet_session/historical/index.html) | An opt-in utility for tracking historical sessions in FRAME-session. Rather than store the full session data for any given session, we instead **commit to the roots of merkle tries containing the session data**. These **roots and proofs of inclusion can be generated at any time during the current session**. Afterwards, **the proofs can be fed to a consensus module when reporting misbehavior**. |
> offenses, babe, grandpa, im-online: work together to deal with illegal writing of validators. Just like a validator produces multiple blocks in the current slot, GRANDPA voters vote multiple times for different blocks in the same round. A large number of The validator is offline for a long time, etc.

## Resources
- Substrate NPOS implementation guide - Chinese version: https://zhuanlan.zhihu.com/p/161293660
- Advanced Staking Concepts on Polkadot Wiki: https://wiki.polkadot.network/docs/learn-staking-advanced
- Substrate Stencil (NPOS blockchain with Substrate): https://github.com/kaichaosun/substrate-stencil
