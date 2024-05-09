//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use parachain_template_runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash, Nonce};

use sc_client_api::backend::AuxStore;
use sc_consensus_babe::BabeWorkerHandle;
use sc_consensus_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::KeystorePtr;

pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// A handle to the BABE worker for issuing requests.
    pub babe_worker_handle: BabeWorkerHandle<Block>,
    /// The keystore that manages the keys of the node.
    pub keystore: KeystorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,

    /// The SelectChain Strategy
    pub select_chain: SC,
    /// A copy of the chain spec.
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
    /// BABE specific dependencies
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies
    pub grandpa: GrandpaDeps<B>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC, B>(
    deps: FullDeps<C, P, SC, B>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + sc_client_api::BlockBackend<Block>
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + AuxStore
        + Send
        + Sync
        + 'static,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: BlockBuilder<Block>,
    C::Api: BabeApi<Block>,
    P: TransactionPool + Sync + Send + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashingFor<Block>>,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use sc_consensus_babe_rpc::{Babe, BabeApiServer};
    use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
    use sc_rpc_spec_v2::chain_spec::{ChainSpec, ChainSpecApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcExtension::new(());
    let FullDeps {
        client,
        pool,
        deny_unsafe,
        select_chain,
        chain_spec,
        babe,
        grandpa,
    } = deps;

    let BabeDeps {
        babe_worker_handle,
        keystore,
    } = babe;
    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;
    let chain_name = chain_spec.name().to_string();
    let genesis_hash = client
        .block_hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists");
    let properties = chain_spec.properties();
    // Add ChainSpec RPC API to the RPC module extension
    module.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;
    module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
    module.merge(TransactionPayment::new(client).into_rpc())?;

    module.merge(
        Babe::new(
            client.clone(),
            babe_worker_handle,
            keystore,
            select_chain,
            deny_unsafe,
        )
        .into_rpc(),
    )?;
    module.merge(
        Grandpa::new(
            subscription_executor,
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            finality_provider,
        )
        .into_rpc(),
    )?;

    Ok(module)
}
