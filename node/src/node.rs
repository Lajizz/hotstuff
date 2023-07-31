use crate::config::Export as _;
use crate::config::{Committee, Parameters, Secret};
use consensus::{Block, Consensus, ConsensusError};
use crypto::SignatureService;
use log::{info,error};
use mempool::{Mempool, MempoolError, Payload};
use store::{Store, StoreError};
use thiserror::Error;
use tokio::sync::mpsc::{channel, Receiver};
use libycsb::GClient;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("Failed to read config file '{file}': {message}")]
    ReadError { file: String, message: String },

    #[error("Failed to write config file '{file}': {message}")]
    WriteError { file: String, message: String },

    #[error("Store error: {0}")]
    StoreError(#[from] StoreError),

    #[error(transparent)]
    ConsensusError(#[from] ConsensusError),

    #[error(transparent)]
    MempoolError(#[from] MempoolError),
}

pub struct Node {
    pub commit: Receiver<Block>,
    pub store: Store
}

impl Node {
    pub async fn new(
        committee_file: &str,
        key_file: &str,
        store_path: &str,
        parameters: Option<&str>,
    ) -> Result<Self, NodeError> {
        let (tx_commit, rx_commit) = channel(1000);
        let (tx_consensus, rx_consensus) = channel(1000);
        let (tx_consensus_mempool, rx_consensus_mempool) = channel(1000);

        // Read the committee and secret key from file.
        let committee = Committee::read(committee_file)?;
        let secret = Secret::read(key_file)?;
        let name = secret.name;
        let secret_key = secret.secret;

        // Load default parameters if none are specified.
        let parameters = match parameters {
            Some(filename) => Parameters::read(filename)?,
            None => Parameters::default(),
        };

        // Make the data store.
        let store = Store::new(store_path)?;

        // Run the signature service.
        let signature_service = SignatureService::new(secret_key);

        // Make a new mempool.
        Mempool::run(
            name,
            committee.mempool,
            parameters.mempool,
            store.clone(),
            signature_service.clone(),
            tx_consensus.clone(),
            rx_consensus_mempool,
        )?;

        // Run the consensus core.
        Consensus::run(
            name,
            committee.consensus,
            parameters.consensus,
            store.clone(),
            signature_service,
            tx_consensus,
            rx_consensus,
            tx_consensus_mempool,
            tx_commit,
        )
        .await?;

        info!("Node {} successfully booted", name);
        Ok(Self { commit: rx_commit, store: store})
    }

    pub fn print_key_file(filename: &str) -> Result<(), NodeError> {
        Secret::new().write(filename)
    }

    pub async fn analyze_block(&mut self) {
        let mut gclient = GClient::new();
        while let Some(block) = self.commit.recv().await {
            // This is where we can further process committed block.
            for digest in &block.payload {
                match self.store.read(digest.to_vec()).await {
                    Ok(Some(data)) => {

                        let payload:Payload = bincode::deserialize(&data).unwrap();
                        
                        info!("execute print {}", data.len());
                        for tx in payload.transactions {
                            let response = gclient.execute_cmds(tx).await;
                            info!("response {:?}", response);
                        }
                    }
                    Ok(None) => (),
                    Err(e) => error!("{}", e),
                }
                info!("Execute B{}({})", block.round, base64::encode(digest))
            }
        }
    }
}
