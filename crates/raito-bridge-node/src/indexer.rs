use std::path::PathBuf;

use tokio::sync::broadcast;
use tracing::{error, info};

use crate::{
    bitcoin::BitcoinClient,
    mmr::Accumulator,
    sparse_roots::{SparseRoots, SparseRootsSink, SparseRootsSinkConfig},
};

pub struct Indexer {
    config: IndexerConfig,
    rx_shutdown: broadcast::Receiver<()>,
}

#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub rpc_url: String,
    pub rpc_userpwd: Option<String>,
    pub sink_config: SparseRootsSinkConfig,
    pub mmr_db_path: PathBuf,
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
        let mut mmr = Accumulator::from_file(&self.config.mmr_db_path, "blocks").await?;
        let mut block_height = mmr.get_block_count().await?;
        info!("Current MMR blocks count: {}", block_height);

        // Initialize the sparse roots sink
        let mut sink = SparseRootsSink::new(self.config.sink_config.clone()).await?;

        loop {
            tokio::select! {
                res = bitcoin_client.wait_block_header(block_height) => {
                    match res {
                        Ok((block_header, block_hash)) => {
                            mmr.add_block_header(block_header).await?;
                            // TODO: store block header (add to the queue)
                            let roots = mmr.get_sparse_roots().await?;
                            let sparse_roots = SparseRoots { block_height, roots };
                            // TODO: handle this in a separate task
                            sink.write_sparse_roots(&sparse_roots).await?;
                            info!("Block #{} {} processed", block_height, block_hash);
                            block_height += 1;
                        },
                        Err(e) => {
                            return Err(e)
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
