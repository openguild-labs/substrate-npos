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

```rust
impl EpochChangeTrigger for ExternalTrigger {
	fn trigger<T: Config>(_: BlockNumberFor<T>) {} // nothing - trigger is external.
}

/// A type signifying to BABE that it should perform epoch changes
/// with an internal trigger, recycling the same authorities forever.
pub struct SameAuthoritiesForever;

impl EpochChangeTrigger for SameAuthoritiesForever {
	fn trigger<T: Config>(now: BlockNumberFor<T>) {
		if <Pallet<T>>::should_epoch_change(now) {
			let authorities = <Pallet<T>>::authorities();
			let next_authorities = authorities.clone();

			<Pallet<T>>::enact_epoch_change(authorities, next_authorities, None);
		}
	}
}
```
## Ressource
- https://zhuanlan.zhihu.com/p/161293660
