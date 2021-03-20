use crate::config::{Committee, Parameters, Protocol};
use crate::core::Core;
use crate::fallback::Fallback;
use crate::vaba::VABA;
use crate::error::ConsensusResult;
use crate::leader::LeaderElector;
use crate::mempool::{MempoolDriver, NodeMempool};
use crate::messages::Block;
use crate::synchronizer::Synchronizer;
use crypto::{PublicKey, SignatureService};
use network::{NetReceiver, NetSender};
use store::Store;
use tokio::sync::mpsc::{channel, Sender};
// use tokio::time::{Duration, sleep};
use threshold_crypto::PublicKeySet;

#[cfg(test)]
#[path = "tests/consensus_tests.rs"]
pub mod consensus_tests;

pub struct Consensus;

impl Consensus {
    pub async fn run<Mempool: 'static + NodeMempool>(
        name: PublicKey,
        committee: Committee,
        parameters: Parameters,
        signature_service: SignatureService,
        pk_set: PublicKeySet,   // The set of tss public keys
        store: Store,
        mempool: Mempool,
        commit_channel: Sender<Block>,
        protocol: Protocol,
    ) -> ConsensusResult<()> {
        let (tx_core, rx_core) = channel(1000);
        let (tx_network, rx_network) = channel(1000);

        // Make the network sender and receiver.
        let address = committee.address(&name).map(|mut x| {
            x.set_ip("0.0.0.0".parse().unwrap());
            x
        })?;
        let network_receiver = NetReceiver::new(address, tx_core.clone());
        tokio::spawn(async move {
            network_receiver.run().await;
        });

        let mut network_sender = NetSender::new(rx_network);
        tokio::spawn(async move {
            network_sender.run().await;
        });

        // The leader elector algorithm.
        let leader_elector = LeaderElector::new(committee.clone());

        // Make the mempool driver which will mediate our requests to the mempool.
        let mempool_driver = MempoolDriver::new(mempool, tx_core.clone(), store.clone());

        // Make the synchronizer. This instance runs in a background thread
        // and asks other nodes for any block that we may be missing.
        let synchronizer = Synchronizer::new(
            name,
            committee.clone(),
            store.clone(),
            /* network_channel */ tx_network.clone(),
            /* core_channel */ tx_core,
            parameters.sync_retry_delay,
        )
        .await;

        // sleep(Duration::from_millis((committee.size() as u64) * parameters.timeout_delay)).await;

        match protocol {
            Protocol::HotStuff => {  // Run HotStuff
                let mut core = Core::new(
                    name,
                    committee,
                    parameters,
                    signature_service,
                    store,
                    leader_elector,
                    mempool_driver,
                    synchronizer,
                    /* core_channel */ rx_core,
                    /* network_channel */ tx_network,
                    commit_channel,
                );
                tokio::spawn(async move {
                    core.run().await;
                });
            },
            Protocol::HotStuffWithAsyncFallback => {  // Run HotStuff with Async Fallback
                let mut hotstuff_with_fallback = Fallback::new(
                    name,
                    committee,
                    parameters,
                    signature_service,
                    pk_set,
                    store,
                    leader_elector,
                    mempool_driver,
                    synchronizer,
                    /* core_channel */ rx_core,
                    /* network_channel */ tx_network,
                    commit_channel,
                );
                tokio::spawn(async move {
                    hotstuff_with_fallback.run().await;
                });
            },
            Protocol::ChainedVABA => {  // Run HotStuff with Async Fallback
                let mut vaba = VABA::new(
                    name,
                    committee,
                    parameters,
                    signature_service,
                    pk_set,
                    store,
                    leader_elector,
                    mempool_driver,
                    synchronizer,
                    /* core_channel */ rx_core,
                    /* network_channel */ tx_network,
                    commit_channel,
                );
                tokio::spawn(async move {
                    vaba.run().await;
                });
            },
            _ => return Ok(()),
        }
    
        Ok(())
    }
}
