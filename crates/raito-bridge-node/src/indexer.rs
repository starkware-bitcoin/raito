use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::{error, info};

const SPARSE_ROOTS_QUEUE_SIZE: usize = 10000;

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

        // TODO: recreate MMR from the stored block headers
        let mut mmr = Accumulator::default();
        let mut block_height = 0;

        let (sparse_roots_tx, mut sparse_roots_rx) =
            mpsc::channel::<SparseRoots>(SPARSE_ROOTS_QUEUE_SIZE);

        // Initialize the sparse roots sink
        let mut sink = SparseRootsSink::new(self.config.sink_config.clone()).await?;

        loop {
            tokio::select! {
                res = bitcoin_client.get_next_block_header(block_height) => {
                    match res {
                        Ok((block_header, block_hash)) => {
                            mmr.add_block_header(block_header).await?;
                            // TODO: store block header (add to the queue)
                            let roots = mmr.get_sparse_roots().await?;
                            sparse_roots_tx.send(SparseRoots { block_height, roots }).await?;
                            info!("Block #{} {} processed", block_height, block_hash);
                            block_height += 1;
                        },
                        Err(e) => {
                            return Err(e)
                        }
                    }
                },
                Some(sparse_roots) = sparse_roots_rx.recv() => {
                    if let Err(e) = sink.write_sparse_roots(&sparse_roots).await {
                        error!("Failed to write sparse roots for block {}: {}", sparse_roots.block_height, e);
                        return Err(e);
                    }
                }
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
