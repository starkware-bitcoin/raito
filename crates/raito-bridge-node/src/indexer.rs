//! Bitcoin blockchain indexer that builds MMR accumulator and generates sparse roots for new blocks.

use std::{path::PathBuf, sync::Arc};

use accumulators::hasher::stark_blake::StarkBlakeHasher;
use raito_spv_mmr::block_mmr::BlockMMR;
use tokio::sync::broadcast;
use tracing::{error, info};

use raito_bitcoin_client::BitcoinClient;

use crate::{chain_state::ChainStateManager, store::AppStore};

/// Bitcoin block indexer that builds MMR accumulator and generates sparse roots
pub struct Indexer {
    /// Indexer configuration
    config: IndexerConfig,
    /// Shutdown signal receiver
    rx_shutdown: broadcast::Receiver<()>,
}

#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Bitcoin RPC URL
    pub rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    pub rpc_userpwd: Option<String>,
    /// MMR ID
    pub mmr_id: String,
    /// Path to the database storing the MMR accumulator
    pub mmr_db_path: PathBuf,
    /// Indexing lag in blocks
    pub indexing_lag: u32,
}

impl Indexer {
    pub fn new(config: IndexerConfig, rx_shutdown: broadcast::Receiver<()>) -> Self {
        Self {
            config,
            rx_shutdown,
        }
    }

    async fn run_inner(&mut self) -> Result<(), anyhow::Error> {
        info!("Block indexer started");

        let mut bitcoin_client =
            BitcoinClient::new(self.config.rpc_url.clone(), self.config.rpc_userpwd.clone())?;
        info!("Bitcoin RPC client initialized");

        // We need to specify mmr_id to have deterministic keys in the database
        let mmr_id = Some(self.config.mmr_id.clone());
        let store = Arc::new(
            AppStore::single_atomic_writer(&self.config.mmr_db_path, mmr_id.clone()).await?,
        );
        let hasher = Arc::new(StarkBlakeHasher::default());
        let mut mmr = BlockMMR::new(store.clone(), hasher, mmr_id);
        info!("MMR loaded from {}", self.config.mmr_db_path.display());

        let mut next_block_height = mmr.get_block_count().await?;
        info!("Current MMR blocks count: {}", next_block_height);

        let mut chain_state_mgr =
            ChainStateManager::restore(store.clone(), next_block_height).await?;
        info!("Chain state manager initialized");

        loop {
            tokio::select! {
                res = bitcoin_client.wait_block_header(next_block_height, self.config.indexing_lag) => {
                    match res {
                        Ok((block_header, block_hash)) => {
                            store.begin().await?;
                            mmr.add_block_header(&block_header).await.map_err(|e| anyhow::anyhow!("Failed to add block header to MMR: {}", e))?;
                            chain_state_mgr.update(next_block_height, &block_header).await.map_err(|e| anyhow::anyhow!("Failed to update chain state: {}", e))?;
                            store.commit().await?;
                            info!("Block #{} {} processed", next_block_height, block_hash);
                            next_block_height += 1;
                        },
                        Err(e) => {
                            return Err(e.into())
                        }
                    }
                },
                _ = self.rx_shutdown.recv() => {
                    return Ok(())
                }
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), ()> {
        match self.run_inner().await {
            Err(err) => {
                error!("Block indexer exited: {}", err);
                Err(())
            }
            Ok(()) => {
                info!("Block indexer terminated");
                Ok(())
            }
        }
    }
}
