//! Bitcoin blockchain indexer that builds MMR accumulator and generates sparse roots for new blocks.

use tokio::sync::broadcast;
use tracing::{error, info};

use crate::{
    app::AppClient,
    bitcoin::BitcoinClient,
    sparse_roots::{SparseRootsSink, SparseRootsSinkConfig},
};

/// Bitcoin block indexer that builds MMR accumulator and generates sparse roots
pub struct Indexer {
    /// Indexer configuration
    config: IndexerConfig,
    /// App client
    app_client: AppClient,
    /// Shutdown signal receiver
    rx_shutdown: broadcast::Receiver<()>,
}

#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Bitcoin RPC URL
    pub rpc_url: String,
    /// Bitcoin RPC user:password (optional)
    pub rpc_userpwd: Option<String>,
    /// Output directory for sparse roots JSON files
    pub sink_config: SparseRootsSinkConfig,
}

impl Indexer {
    pub fn new(
        config: IndexerConfig,
        app_client: AppClient,
        rx_shutdown: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            config,
            app_client,
            rx_shutdown,
        }
    }

    async fn run_inner(&mut self) -> Result<(), anyhow::Error> {
        info!("Block indexer started");

        let mut bitcoin_client =
            BitcoinClient::new(self.config.rpc_url.clone(), self.config.rpc_userpwd.clone())?;
        info!("Bitcoin RPC client initialized");

        let mut block_height = self.app_client.get_block_count().await?;
        info!("Current MMR blocks count: {}", block_height);

        // Initialize the sparse roots sink
        let mut sink = SparseRootsSink::new(self.config.sink_config.clone()).await?;

        loop {
            tokio::select! {
                res = bitcoin_client.wait_block_header(block_height) => {
                    match res {
                        Ok((block_header, block_hash)) => {
                            // Add new block to the MMR accumulator and get resulting sparse roots
                            let roots = self.app_client.add_block(block_header).await?;
                            sink.write_sparse_roots(&roots).await?;
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
