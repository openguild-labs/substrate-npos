# Nominated Proof of Stake (NPOS) Substrate-based Chain Tutorial
Substrate node template is a node program for quickly developing Substrate application chains. It has a built-in **Proof of Authority (PoA)** consensus algorithm. The block generation algorithm is Aura , which means that the nodes and order of block generation are fixed. An online public blockchain application needs to achieve a certain degree of decentralization and ensure security through random block generation. Substrate's built-in Proof of Stake (PoS) consensus algorithm is designed to meet such needs.
## Step 1: Migrate the Block Production algorithm (AURA -> BABE)
Before we implement the NPOS (Nominated Proof of Stake) which is PoS (Proof of Stake) with the nomination mechanism, we will work on migrate the Substrate node template from its base consensus mechanism PoA to PoS first. Below are steps to migrate the Substrate-based chain template to the PoS blockchain.
- To read more about the consensus in blockchain and Polkadot, [visit the official Substrate documentation](https://docs.substrate.io/learn/consensus/) or on [Polkadot Wiki - Learn Consensus](https://wiki.polkadot.network/docs/learn-consensus)

### Common attributes of AURA and BABE
- The Aura and BABE consensus models require you to have a known set of validator nodes that are permitted to produce blocks
- In both of these consensus models, time is divided up into **discrete slots**. During each slot only some of the validators can produce a block
 
### What is AURA and its limitation?
[**Authority-based round-robin scheduling (AURA)**](https://docs.rs/pallet-aura/latest/pallet_aura/) is a default block authoring algorithm used in Substrate Node Template for block production. In the Aura consensus model, validators that can author blocks rotate in a round-robin fashion. Read more about the [Round-Robin (RR) scheduling algorithm](https://en.wikipedia.org/wiki/Round-robin_scheduling). Hence, making AURA algorithm deterministic.

### Migrate from AURA to BABE
[**Blind assignment of blockchain extension (BABE) slot-based scheduling**](https://wiki.polkadot.network/docs/learn-consensus#block-production-babe) is the another blockchain authoring algorithm used by Polkadot. Compared to AURA, BABE is more random and non-deterministic because the validator who is assigned to produce a block in the epoch is randomly chosen using [VRF (Verifiable Randome Function)](https://wiki.polkadot.network/docs/learn-cryptography#vrf).

- Each validator is assigned a weight for an epoch. This epoch is broken up into slots and the validator evaluates its VRF at each slot. For each slot that **the validator's VRF output is below its weight, it is allowed to author a block.**
- Because multiple validators might be able to produce a block during the same slot, **forks are more common in BABE than they are in Aura**, even in good network conditions.
- Substrate's implementation of BABE also has a **fallback mechanism for when no authorities are chosen in a given slot**. These **secondary slot** assignments allow BABE to achieve a constant block time.

### Code Breakdown
Read more about [the configuration of BABE pallet.](https://paritytech.github.io/polkadot-sdk/master/pallet_babe/pallet/trait.Config.html)
```rust
pub const MILLISECS_PER_BLOCK: Moment = 3000;
pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
pub const EPOCH_DURATION_IN_SLOTS: u64 = {
  const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;
  (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
};

const EpochDuration : u64 = EPOCH_DURATION_IN_BLOCKS;
const ExpectedBlockTime : u64 = MILLISECS_PER_BLOCK;

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
> ANSWER: Let's say you have found an equivocation for Babe. Aka you are saying that Alice build two blocks on the same height in session 100. You will now need to prove to the runtime that there is an equivocation. You send the proof to the runtime. While the runtime checks the proof, it needs to ensure that Alice is really part of the validator set in session 100. This check is done using the KeyProofSystem. In case of `pallet_session::historical`, _it records all the validator sets for the last X sessions (can be configured) to prove these kind of requests._

```rust
type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
```
From that information, we can explain this type parameter is to declare the type of `Proof` coming from a `KeyOwnerProofSystem` of the `pallet_session::historical::{Pallet}` which records all the validator sets of the last X sessions, validators are indentified using the `pallet_babe::AuthorityId`.

## Ressource
- https://zhuanlan.zhihu.com/p/161293660
