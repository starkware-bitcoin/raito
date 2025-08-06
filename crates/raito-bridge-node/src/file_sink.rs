//! File sink for sparse roots MMR peaks compatible with Cairo implementation.

use raito_spv_core::sparse_roots::SparseRoots;
use serde_json;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info};

/// Configuration for the sparse roots sink
#[derive(Debug, Clone)]
pub struct SparseRootsSinkConfig {
    /// Output directory for the sparse roots JSON files
    pub output_dir: PathBuf,
    /// Shard size for the sparse roots JSON files
    pub shard_size: u32,
}

/// Sink for writing sparse roots to a JSON file
pub struct SparseRootsSink {
    config: SparseRootsSinkConfig,
}

impl SparseRootsSink {
    /// Create a new sparse roots sink with the given configuration
    pub async fn new(config: SparseRootsSinkConfig) -> Result<Self, anyhow::Error> {
        // Create the output directory if it doesn't exist
        fs::create_dir_all(&config.output_dir).await?;

        info!(
            "SparseRootsSink initialized with output_dir: {:?}, shard_size: {}",
            config.output_dir, config.shard_size
        );

        Ok(Self { config })
    }

    /// Calculate the shard directory path for a given block height
    fn get_shard_dir(&self, block_height: u32) -> PathBuf {
        let shard_id = block_height / self.config.shard_size;
        let shard_start = shard_id * self.config.shard_size;
        let shard_end = shard_start + self.config.shard_size;
        let shard_dir_name = format!("{shard_end}");
        self.config.output_dir.join(shard_dir_name)
    }

    /// Get the file path for a specific block height
    fn get_file_path(&self, block_height: u32) -> PathBuf {
        let shard_dir = self.get_shard_dir(block_height);
        let filename = format!("block_{block_height}.json");
        shard_dir.join(filename)
    }

    /// Write sparse roots to a JSON file
    pub async fn write_sparse_roots(
        &mut self,
        sparse_roots: &SparseRoots,
    ) -> Result<(), anyhow::Error> {
        let file_path = self.get_file_path(sparse_roots.block_height);

        // Create the shard directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Serialize the sparse roots to JSON
        let json_content = serde_json::to_string_pretty(sparse_roots)?;

        // Write to file
        fs::write(&file_path, json_content).await?;

        debug!(
            "Sparse roots for block {} written to {:?}",
            sparse_roots.block_height, file_path
        );

        Ok(())
    }
}
