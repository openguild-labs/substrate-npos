# Nominated Proof of Stake (NPOS) Substrate-based Chain Tutorial

![Tutorial_Design](https://github.com/openguild-labs/substrate-npos/assets/56880684/c719b312-3117-4c75-a963-fcc5610b6fff)

Substrate node template is a node program for quickly developing Substrate application chains. It has a built-in **Proof of Authority (PoA)** consensus algorithm. The block generation algorithm is Aura , which means that the nodes and order of block generation are fixed. An online public blockchain application needs to achieve a certain degree of decentralization and ensure security through random block generation. Substrate's built-in Proof of Stake (PoS) consensus algorithm is designed to meet such needs.
## Create a new Parachain project with `pop-cli`
You can learn more about `pop-cli` here: https://github.com/r0gue-io/pop-cli
- Install the `pop-cli`: cargo install --locked --git https://github.com/r0gue-io/pop-cli
- Create a new parachain boilerplate code with `pop new parachain substrate-npos`
- Build your newly created local parachain with `cd substrate-npos && pop run parachain`
- Run your local relaychain network using Zombinet with `pop up parachain -f ./zombienet.toml`
Now you are ready to kickstart working on the later steps.

## Deliverables
- [x] Migrate from AURA to hybrid consensus GRANDPA + BABE
- [x] Implement runtime API & block import queue in the outer node service
- [ ] Implement `pallet-staking` and integrate with existing consensus mechanism
- [ ] Implement `pallet-session` for session management
- [ ] Implement `pallet-nomniation-pool` for nominator registration
- [ ] Implement election functionalities for active validator selection
- [ ] Demo of the functionalities on the Polkadot JS Apps

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

## Concepts
### Chain Specification
Chain Spec contains a series of configuration information, which is used by the node program to connect to the specified blockchain network, connectable boot node information, and the status of the initial block. The specific information is as follows:

- Meta information such as the name and id of the chain;
- The type of chain chainType, commonly used ones are Development to start a local single-node network, Local to start a local multi-node test network, and Live to be a public test network or official network;
- Start the boot node bootNodes, that is, after the program starts, what are the initial connectable network nodes, used to obtain block information and other connectable node information;
- Monitor the service address telemetryEndpoints. After the node is started, it will regularly send the node's own status such as CPU usage, memory consumption, current block information, etc. to this address;
- protocolId is used to identify non-libp2p protocols used in point-to-point network transmission. The Substrate network transport layer relies on libp2p. In addition to the protocols specified by libp2p, the non-libp2p protocols specified in Substrate need to use protocolId to distinguish different blockchain networks. For more information, please refer to [the document sc_network](https://link.zhihu.com/?target=https%3A//crates.parity.io/sc_network/index.html) ;
- properties can contain some optional attribute information, such as tokenSymbol representing token characters and tokenDecimals, and the front end can obtain the corresponding information for display;
- Genesis is the initial block information, which is the collection of initial information of all available modules.
### Runtime Transaciont Type
A definition of types required to submit transactions from within the runtime. [`UnCheckedExtrinsic`](https://paritytech.github.io/substrate/master/sp_runtime/generic/struct.UncheckedExtrinsic.html) is an extrinsic right from the external world. This is unchecked and so can contain a signature. More information and examples for the `SendTransactionTypes` can be found in [Add offchain worker - Substrate official documentation](https://docs.substrate.io/tutorials/build-application-logic/add-offchain-workers/).
```rust
// https://crates.parity.io/frame_system/offchain/trait.SendTransactionTypes.html
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    RuntimeCall: From<C>,
{
    // The runtime expects the UncheckedExtrinsic type
    // https://paritytech.github.io/substrate/master/sp_runtime/generic/struct.UncheckedExtrinsic.html
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = RuntimeCall;
}
```

---

# Step 1: Migrate from AURA to Hybrid Consensus BABE + GRANDPA
Before we implement the NPOS (Nominated Proof of Stake) which is PoS (Proof of Stake) with the nomination mechanism, we will work on migrate the Substrate node template from its base consensus mechanism PoA to PoS first. Below are steps to migrate the Substrate-based chain template to the PoS blockchain.
- To read more about the consensus in blockchain and Polkadot, [visit the official Substrate documentation](https://docs.substrate.io/learn/consensus/) or on [Polkadot Wiki - Learn Consensus](https://wiki.polkadot.network/docs/learn-consensus)

## Common attributes of AURA and BABE
- The Aura and BABE consensus models require you to have a known set of validator nodes that are permitted to produce blocks
- In both of these consensus models, time is divided up into **discrete slots**. During each slot only some of the validators can produce a block
 
## What is AURA and its limitation?
[**Authority-based round-robin scheduling (AURA)**](https://docs.rs/pallet-aura/latest/pallet_aura/) is a default block authoring algorithm used in Substrate Node Template for block production. In the Aura consensus model, validators that can author blocks rotate in a round-robin fashion. Read more about the [Round-Robin (RR) scheduling algorithm](https://en.wikipedia.org/wiki/Round-robin_scheduling). Hence, making AURA algorithm deterministic.

## Migrate and configure from AURA to BABE
[**Blind assignment of blockchain extension (BABE) slot-based scheduling**](https://wiki.polkadot.network/docs/learn-consensus#block-production-babe) is the another blockchain authoring algorithm used by Polkadot. Compared to AURA, BABE is more random and non-deterministic because the validator who is assigned to produce a block in the epoch is randomly chosen using [VRF (Verifiable Randome Function)](https://wiki.polkadot.network/docs/learn-cryptography#vrf).

- Each validator is assigned a weight for an epoch. This epoch is broken up into slots and the validator evaluates its VRF at each slot. For each slot that **the validator's VRF output is below its weight, it is allowed to author a block.**
- Because multiple validators might be able to produce a block during the same slot, **forks are more common in BABE than they are in Aura**, even in good network conditions.
- Substrate's implementation of BABE also has a **fallback mechanism for when no authorities are chosen in a given slot**. These **secondary slot** assignments allow BABE to achieve a constant block time.

## Code Breakdown
### Add related pallets to `Cargo.toml`
Before diving deeper, here are pallets that we will add at this stage: 
```toml
pallet-babe = { version = "31.0.0", default-features = false }
pallet-offences = { version = "31.0.0", default-features = false }

[features]
std=[
	"pallet-babe/std",
	"pallet-session/std",
	"pallet-offences/std",
]
```
### Declare pallet in the construct_runtime! macro
The use of each pallet is described below. We also need to add these pallet in the `construct_runtime!` macro:
```rust
construct_runtime! {
	...,
	// NPOS relevant pallets
	Babe: pallet_babe = 34,
        Historical: pallet_session::historical::{Pallet} = 35,
        Offences: pallet_offences = 36,
}
```
### Configure BABE pallet
Read more about [the configuration of BABE pallet.](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html)
```rust
pub const MILLISECS_PER_BLOCK: Moment = 3000;
pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
pub const EPOCH_DURATION_IN_SLOTS: u64 = {
    // Calculate the number of slots required to produce one block
    const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

    // By that way, we can calculate the epoch duration in one block
    (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
};

parameter_types! {
	const EpochDuration : u64 = EPOCH_DURATION_IN_BLOCKS;
	const ExpectedBlockTime : u64 = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 = 24 * 28 * 6 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
    type EpochDuration = EpochDuration;
    type ExpectedBlockTime = ExpectedBlockTime;
    // 
    type EpochChangeTrigger = pallet_babe::SameAuthoritiesForever;
    // Note: DisabledValidators trait is implemented for the pallet_session
    type DisabledValidators = Session;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<100>;
    type MaxNominators = ConstU32<100>;
    type KeyOwnerProof =
        <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
    type EquivocationReportSystem =
        pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}
```
Firstly, we define some constant variables and assign those to the type parameters `EpochDuration` and `ExpectedBlockTime`. These type parameters are quite straightforward:
- [`EpochDuration`](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html#required-associated-types): The amount of time, in slots, that each epoch should last. NOTE: Currently it is not possible to change the epoch duration after the chain has started. Attempting to do so will brick block production.
- [`ExpectedBlockTime`](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html#required-associated-types): The expected time for BABE to produce a new block. This is not a fixed block time like AURA which the block author is deterministically assigned usign turn-based RR algorithm. Hence, the expected block time is an average block time based on the number of slot duration and **the security parameter c (where 1 - c represents the probability of a slot being empty).**
- [`EpochChangeTrigger`](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html#required-associated-types): BABE requires some logic to be triggered on every block to query for whether an epoch has ended and to perform the transition to the next epoch. The below code use [`SameAuthoritiesForever`](https://docs.rs/pallet-babe/latest/pallet_babe/struct.SameAuthoritiesForever.html) which recycles the same authorities forever. 
```diff
impl EpochChangeTrigger for ExternalTrigger {
	fn trigger<T: Config>(_: BlockNumberFor<T>) {} // nothing - trigger is external.
}

/// A type signifying to BABE that it should perform epoch changes
/// with an internal trigger, recycling the same authorities forever.
pub struct SameAuthoritiesForever;

impl EpochChangeTrigger for SameAuthoritiesForever {
	fn trigger<T: Config>(now: BlockNumberFor<T>) {
		if <Pallet<T>>::should_epoch_change(now) {
+			let authorities = <Pallet<T>>::authorities();
+			let next_authorities = authorities.clone();
+			<Pallet<T>>::enact_epoch_change(authorities, next_authorities, None);
		}
	}
}
```
We can check the implementation of the `SameAuthoritieisForever` struct to understand how the same authorities is recycled for the next epoch. 
- [`DisabledValidators`](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html#required-associated-types): A type parameter accepts struct that implement the `DisabledValidators` trait. In this case, we provide `Session` pallet because it implements the trait. By checking the source code of the `Session` pallet, we can see [how's the trait is applied for the Session pallet struct](https://paritytech.github.io/polkadot-sdk/master/src/pallet_session/lib.rs.html#18-934:~:text=impl%3CT%3A%20Config%3E%20frame_support%3A%3Atraits%3A%3ADisabledValidators%20for%20Pallet%3CT%3E%20%7B%0A%09fn%20is_disabled(index%3A%20u32)%20%2D%3E%20bool%20%7B%0A%09%09%3CPallet%3CT%3E%3E%3A%3Adisabled_validators().binary_search(%26index).is_ok()%0A%09%7D%0A%0A%09fn%20disabled_validators()%20%2D%3E%20Vec%3Cu32%3E%20%7B%0A%09%09%3CPallet%3CT%3E%3E%3A%3Adisabled_validators()%0A%09%7D%0A%7D)
```rust
impl<T: Config> frame_support::traits::DisabledValidators for Pallet<T> {
	fn is_disabled(index: u32) -> bool {
		<Pallet<T>>::disabled_validators().binary_search(&index).is_ok()
	}

	fn disabled_validators() -> Vec<u32> {
		<Pallet<T>>::disabled_validators()
	}
}
```
- [`KeyOwnerProof`](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html#required-associated-types): This type parameter is more complicated. Based on the documentation and these below discussion:
	- ["What is the KeyOwnerProofSystem in BABE config?" - Substrate Stack Exchange](https://substrate.stackexchange.com/questions/42/what-is-the-keyownerproofsystem-in-babe-config)
 	- ["In substrate, what is the equivocation handling system and KeyOwnerProof/Identification/ProofSystem in babe config?"](https://stackoverflow.com/questions/71004838/in-substrate-what-is-the-equivocation-handling-system-and-keyownerproof-identif)

We got that the `KeyOwnerProofSystem` is a system that when we need to provide a [EquivocationProof](https://docs.rs/sp-consensus-slots/latest/sp_consensus_slots/struct.EquivocationProof.html) to the Runtime for checking if the equivocation happens.
> ANSWER: Let's say you have found an equivocation for Babe. Aka you are saying that Alice build two blocks on the same height in session 100. You will now need to prove to the runtime that there is an equivocation. You send the proof to the runtime. While the runtime checks the proof, it needs to ensure that Alice is really part of the validator set in session 100. This check is done using the KeyProofSystem. In case of `pallet_session::historical`, _it records all the validator sets for the last X sessions (can be configured) to prove these kind of requests_.

There is a better explanation of the equivocation in the below section about the type parameter `EquivocationReportSystem`.

```rust
type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
```
The proof of key ownership, used for validating equivocation reports. The proof must include the session index and validator count of the session at which the equivocation occurred. From that information, we can explain this type parameter is to declare the type of `Proof` coming from a `KeyOwnerProofSystem` of the `pallet_session::historical::{Pallet}` which records all the validator sets of the last X sessions, validators are indentified using the `pallet_babe::AuthorityId`.

- [`EquivocationReportSystem`](https://paritytech.github.io/polkadot-sdk/master/src/pallet_babe/lib.rs.html#168-171): The equivocation handling subsystem, defines methods to check/report an offence and for submitting a transaction to report an equivocation (from an offchain context). Learn more about [Validator Equivocation](https://wiki.polkadot.network/docs/maintain-guides-avoid-slashing#equivocation).
	- BABE equivocation: Equivocation events can occur when a validator produces two or more of the same block; under this condition.
 	- GRANDPA Equivocation: Equivocation may also happen when a validator signs two or more of the same consensus vote; under this condition.
In our case, it is `BABE equivocation`
```
type EquivocationReportSystem = pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
```
The implementation for the `EquivocationReportSystem` in `BABE` can be found in [polkadot-sdk/substrate/frame/babe/src/equivocation.rs](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/babe/src/equivocation.rs#L124). Read more about the function signatures and type declaration of [`OffenceReportSystem`](https://docs.rs/sp-staking/latest/src/sp_staking/offence.rs.html#18-276)

```rust
/// BABE equivocation offence report system.
///
/// This type implements `OffenceReportSystem` such that:
/// - Equivocation reports are published on-chain as unsigned extrinsic via
///   `offchain::SendTransactionTypes`.
/// - On-chain validity checks and processing are mostly delegated to the user provided generic
///   types implementing `KeyOwnerProofSystem` and `ReportOffence` traits.
/// - Offence reporter for unsigned transactions is fetched via the the authorship pallet.
pub struct EquivocationReportSystem<T, R, P, L>(sp_std::marker::PhantomData<(T, R, P, L)>);

impl<T, R, P, L>
	OffenceReportSystem<Option<T::AccountId>, (EquivocationProof<HeaderFor<T>>, T::KeyOwnerProof)>
	for EquivocationReportSystem<T, R, P, L>
where
	T: Config + pallet_authorship::Config + frame_system::offchain::SendTransactionTypes<Call<T>>,
	R: ReportOffence<
		T::AccountId,
		P::IdentificationTuple,
		EquivocationOffence<P::IdentificationTuple>,
	>,
	P: KeyOwnerProofSystem<(KeyTypeId, AuthorityId), Proof = T::KeyOwnerProof>,
	P::IdentificationTuple: Clone,
	L: Get<u64>,
{
	type Longevity = L;

	fn publish_evidence(
		evidence: (EquivocationProof<HeaderFor<T>>, T::KeyOwnerProof),
	) -> Result<(), ()> {...}

	fn check_evidence(
		evidence: (EquivocationProof<HeaderFor<T>>, T::KeyOwnerProof),
	) -> Result<(), TransactionValidityError> {...}

	fn process_evidence(
		reporter: Option<T::AccountId>,
		evidence: (EquivocationProof<HeaderFor<T>>, T::KeyOwnerProof),
	) -> Result<(), DispatchError> {...}
}
```

There are three other type that requires a further explanation `Offences, Historical, ReportLongevity`
- [`Offences`](https://docs.rs/pallet-offences/latest/src/pallet_offences/lib.rs.html#51): This pallet simply defined storage value and Pallet dispatchable functions to store and track the reported offences. 
- [`Historical`](https://docs.rs/pallet-session/latest/pallet_session/historical/index.html): An opt-in utility for tracking historical sessions in FRAME-session. Rather than store the full session data for any given session, we instead **commit to the roots of merkle tries containing the session data**. These **roots and proofs of inclusion can be generated at any time during the current session**. Afterwards, **the proofs can be fed to a consensus module when reporting misbehavior**.
- `ReportLongevity`: A constant variable defines the lifetime of the report (longevity), request by the `OffenceReportSystem ( type Longevity = Get<u64> )`
```rs
pub const ReportLongevity: u64 = 24 * 28 * 6 * EpochDuration::get();
```

## Add GRANDPA finality gadget to Runtime Modules
```diff
// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime {
        // System support stuff.
        System: frame_system = 0,
        ParachainSystem: cumulus_pallet_parachain_system = 1,
        Timestamp: pallet_timestamp = 2,
        ParachainInfo: parachain_info = 3,

        // Monetary stuff.
        Balances: pallet_balances = 10,
        TransactionPayment: pallet_transaction_payment = 11,

        // Governance
        Sudo: pallet_sudo = 15,

        // Collator support. The order of these 4 are important and shall not change.
        Authorship: pallet_authorship = 20,
        CollatorSelection: pallet_collator_selection = 21,
        Session: pallet_session = 22, // also used in for the staking feature

        // XCM helpers.
        XcmpQueue: cumulus_pallet_xcmp_queue = 30,
        PolkadotXcm: pallet_xcm = 31,
        CumulusXcm: cumulus_pallet_xcm = 32,
        MessageQueue: pallet_message_queue = 33,

        // NPOS relevant pallets
        Babe: pallet_babe = 34,
+        Grandpa: pallet_grandpa = 35,
        Historical: pallet_session::historical::{Pallet} = 36,
        Offences: pallet_offences = 37,
        // Staking: pallet_staking,
    }
);
```
[GRANDPA Consensus module for runtime.](https://docs.rs/pallet-grandpa/latest/pallet_grandpa/) This manages the GRANDPA authority set ready for the native code. These authorities are only for GRANDPA finality, not for consensus overall. In the future, it will also handle misbehavior reports, and on-chain finality notifications.
```rs
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = MaxNominators;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
    type EquivocationReportSystem =
        pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}
```
Because GRANDPA and BABE is usually used together, the type parameters used in `pallet-grandpa` is also similar to type parameters of `pallet-babe`.
---
# Step 2: Runtime API & Outer Node Service
![Screenshot 2024-05-12 at 13 17 47](https://github.com/openguild-labs/substrate-npos/assets/56880684/6c2b4ee5-ea4d-4db3-bbaf-73d7a7f0e9c1)

## Related code commits
- [Configure Runtime API and Chainspec for BABE](https://github.com/openguild-labs/substrate-npos/commit/fa0a4d6cf3c2148e9bc6469223307c100ab1549c)

## Additional dependencies
Below are mixes of core libraries and primitive libraries. To understand the library structure of Substrate. [Read this documentation](https://docs.substrate.io/build/libraries/)
| Crate name    | Description | Type |
| -------- | ------- | ---- |
| [sc-consensus-babe](https://docs.rs/sc-consensus-babe/latest/sc_consensus_babe/) | Subtrate core utilities for BABE | Core |
| [sc-consensus-epochs](https://docs.rs/sc-consensus-epochs/0.36.0/sc_consensus_epochs/index.html#) | Generic utilities for epoch-based consensus engines. | Core | 
| [sc-consensus-babe-rpc](https://crates.io/crates/sc-consensus-babe-rpc) | RPC api for babe. | Core |
| [sc-consensus-grandpa](https://docs.rs/sc-consensus-grandpa/0.22.0/sc_consensus_grandpa/index.html) | Integration of the GRANDPA finality gadget into substrate. | Core |
| [sp-consensus-grandpa](https://crates.io/crates/sp-consensus-grandpa) | Primitives for GRANDPA integration, suitable for WASM compilation. | Primitive |
| [sc_rpc_spec_v2](https://docs.rs/sc-rpc-spec-v2/latest/sc_rpc_spec_v2/index.html) | Substrate JSON-RPC interface v2. | Core |
| [sp_consensus](https://docs.rs/sp-consensus/0.35.0/sp_consensus/index.html) | Common utilities for building and using consensus engines in substrate. | Primitive |
| [sc_rpc_api](https://docs.rs/sc-rpc-api/latest/sc_rpc_api/) | A collection of RPC methods and subscriptions supported by all substrate clients. | Core |
| [sc_consensus_slots](https://paritytech.github.io/polkadot-sdk/master/sc_consensus_slots/index.html) | Some consensus algorithms have a concept of slots, which are intervals in time during which certain events can and/or must occur. This crate provides generic functionality for slots. | Core |
| [sp_transaction_storage_proof](https://docs.rs/sp-transaction-storage-proof/latest/sp_transaction_storage_proof/) | Storage proof primitives. Contains types and basic code to extract storage proofs for indexed transactions. | Primitive | 

## What is [Runtime API](https://docs.substrate.io/reference/runtime-apis/) in Substrate?
Substrate node consist of outer node services and a runtime and this separation of responsibilities is an important concept for designing Susbtrate-based chains and building upgradable logic. However, the outer node services and the runtime must communicate with each other to complete many critical operations, including reading and writing data and performing state transitions. The outer node services communicate with the runtime by calling runtime application programming interfaces to perform specific tasks.
- [How to add Custom RPC?](https://substrate.stackexchange.com/questions/5554/how-to-add-custom-rpcs)

## Configure collator key set
### Add BABE to the [SessionKeys](https://doc.deepernetwork.org/v3/concepts/session-keys/#:~:text=Substrate%20provides%20the%20Session%20pallet,used%20for%20their%20intended%20purpose.)
Substrate provides the [Session pallet](https://doc.deepernetwork.org/rustdocs/latest/pallet_session/index.html), which allows validators to manage their session keys.

Session keys are "hot keys" that are used by validators to sign consensus-related messages. 
- They are not meant to be used as account keys that control funds and should only be used for their intended purpose. 
- They can be changed regularly; your Controller only needs to create new session keys by signing a session's public key and broadcasting this certificate via an extrinsic. 
- Session keys are also defined generically and made concrete in the runtime.
```diff
impl_opaque_keys! {
    pub struct SessionKeys {
-        pub aura: Aura,
+        pub babe: Babe,
+        pub grandpa: Grandpa,
    }
}
```
You can declare any number of Session keys. The above code declares the BABE session key used in `pallet_session` because we need the validator to sign BABE consensus-related messages.

### Add Runtime API for BABE
We need to add additional [`sp_consensus_babe`](https://paritytech.github.io/polkadot-sdk/master/sp_consensus_babe/index.html) package to our runtime. This package contains the primitives types and utilities for the BABE consensus. 
```toml
sp-consensus-babe = { workspace = true }

[features]
std=[
	"sp-consensus-babe/std",
]
```
First, we implement the trait `BabeApi` for the `Runtime` struct with unimplemented function body. We will finish the implementation of these functions later with explanation.
```rs
 impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> sp_consensus_babe::BabeConfiguration {
            unimplemented!();
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Babe::current_epoch_start()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            Babe::current_epoch()
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            Babe::next_epoch()
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            unimplemented!();
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }
    }
```
- [`sp_consensus_babe::OpaqueKeyOwnershipProof`](https://paritytech.github.io/polkadot-sdk/master/sp_consensus_babe/struct.OpaqueKeyOwnershipProof.html): An opaque type used to represent the key ownership proof at the runtime API boundary. The inner value is an encoded representation of the actual key ownership proof which will be parameterized when defining the runtime. At the runtime API boundary this type is unknown and as such we keep this opaque representation, implementors of the runtime API will have to make sure that all usages of OpaqueKeyOwnershipProof refer to the same type.
### Add Runtime API for Grandpa
```rs
impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            authority_id: GrandpaId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            use codec::Encode;

            Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
                .map(|p| p.encode())
                .map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
        }
    }
```

### Configure chain specification for BABE / GRANDPA collators
For now, we don't remove code related to AURA yet, we will work on it after BABE is successfully implemented in our Runtime and Node. It would be clearer if you visit the code directly to view the changes but here is some of the changes made to the `node/src/chain_spec.rs`
```diff
pub fn template_session_keys(
-    aura: AuraId,
+    babe: BabeId,
+    grandpa: GrandpaId,
) -> parachain_template_runtime::SessionKeys {
-    parachain_template_runtime::SessionKeys { aura: keys }
+    parachain_template_runtime::SessionKeys { babe, grandpa }
}

fn testnet_genesis(
-    invulnerables: Vec<(AccountId, AuraId)>,
+    invulnerables: Vec<(AccountId, BabeId, GrandpaId)>,
    endowed_accounts: Vec<AccountId>,
    root: AccountId,
    id: ParaId,
) -> serde_json::Value {...}
```
Learn more about [Setting up the Collator](https://hackmd.io/@s_iGZLIITG6WjSgnFX0pcg/the-collator-setup-guide).

### Configure Outer Node Service
```rs

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial(
    config: &Configuration,
) -> Result<
    Service<
        impl Fn(
            sc_rpc::DenyUnsafe,
            sc_rpc::SubscriptionTaskExecutor,
        ) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>,
    >,
    sc_service::Error,
> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let heap_pages = config
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let wasm = WasmExecutor::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    let executor = ParachainExecutor::new_with_wasm_executor(wasm);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            true,
        )?;
    let client = Arc::new(client);

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

    let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
    let justification_import = grandpa_block_import.clone();

    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::configuration(&*client)?,
        grandpa_block_import,
        client.clone(),
    )?;

    let slot_duration = babe_link.config().slot_duration();

    let babe_import_queue_params = sc_consensus_babe::ImportQueueParams {
        link: babe_link.clone(),
        block_import: babe_block_import.clone(),
        justification_import: Some(Box::new(justification_import)),
        client: client.clone(),
        select_chain: select_chain.clone(),
        create_inherent_data_providers: move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

            Ok((slot, timestamp))
        },
        spawner: &task_manager.spawn_essential_handle(),
        registry: config.prometheus_registry(),
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(RejectAllTxPool::default()),
    };

    let (import_queue, babe_worker_handle) =
        sc_consensus_babe::import_queue(babe_import_queue_params)?;

    let import_setup = (babe_block_import, grandpa_link, babe_link);
    let (rpc_extensions_builder, rpc_setup) = {
        let (_, grandpa_link, _babe_link) = &import_setup;

        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = sc_consensus_grandpa::SharedVoterState::empty();
        let shared_voter_state2 = shared_voter_state.clone();

        let finality_proof_provider = sc_consensus_grandpa::FinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let keystore = keystore_container.keystore();
        let chain_spec = config.chain_spec.cloned_box();

        let rpc_extensions_builder = move |deny_unsafe, subscription_executor| {
            let deps = FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: select_chain.clone(),
                chain_spec: chain_spec.cloned_box(),
                deny_unsafe,
                babe: BabeDeps {
                    babe_worker_handle: babe_worker_handle.clone(),
                    keystore: keystore.clone(),
                },
                grandpa: GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor,
                    finality_provider: finality_proof_provider.clone(),
                },
            };

            create_full(deps).map_err(Into::into)
        };

        (rpc_extensions_builder, shared_voter_state2)
    };

    Ok(PartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain,
        other: (
            block_import,
            rpc_extensions_builder,
            import_setup,
            rpc_setup,
            telemetry,
            telemetry_worker_handle,
        ),
    })
}
```

## Resources
- Substrate NPOS implementation guide - Chinese version: https://zhuanlan.zhihu.com/p/161293660
- Advanced Staking Concepts on Polkadot Wiki: https://wiki.polkadot.network/docs/learn-staking-advanced
- Substrate Stencil (NPOS blockchain with Substrate): https://github.com/kaichaosun/substrate-stencil
