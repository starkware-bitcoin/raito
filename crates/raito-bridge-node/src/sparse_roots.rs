use num_bigint::BigInt;
use num_traits::Num;
use serde::{Serialize, Serializer};
use serde_json;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct SparseRootsSinkConfig {
    pub output_dir: String,
    pub shard_size: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SparseRoots {
    #[serde(skip)]
    pub block_height: u32,
    #[serde(serialize_with = "serialize_bigint_vec")]
    pub roots: Vec<String>,
}

pub struct SparseRootsSink {
    config: SparseRootsSinkConfig,
    output_dir: PathBuf,
}

impl SparseRootsSink {
    pub async fn new(config: SparseRootsSinkConfig) -> Result<Self, anyhow::Error> {
        let output_dir = PathBuf::from(&config.output_dir);

        // Create the output directory if it doesn't exist
        fs::create_dir_all(&output_dir).await?;

        info!(
            "SparseRootsSink initialized with output_dir: {:?}, shard_size: {}",
            output_dir, config.shard_size
        );

        Ok(Self { config, output_dir })
    }

    /// Calculate the shard directory path for a given block height
    fn get_shard_dir(&self, block_height: u32) -> PathBuf {
        let shard_id = block_height / self.config.shard_size;
        let shard_start = shard_id * self.config.shard_size;
        let shard_end = shard_start + self.config.shard_size;
        let shard_dir_name = format!("{shard_end}");
        self.output_dir.join(shard_dir_name)
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

// Custom serialization for Vec<String> to serialize as integers
fn serialize_bigint_vec<S>(items: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(items.len()))?;
    for item in items {
        let num_str = item.strip_prefix("0x").unwrap_or(&item);
        // TODO: figure out how to forward `truncated` flag here from hasher
        if false {
            // Cast to BigInt and back to string to handle leading zeros
            let json_number = num_str_to_json_number::<S>(num_str)?;
            seq.serialize_element(&json_number)?;
        } else {
            assert_eq!(num_str.len(), 64);
            let (hi, lo) = num_str.split_at(32);
            let hi_json_number = num_str_to_json_number::<S>(hi)?;
            let lo_json_number = num_str_to_json_number::<S>(lo)?;
            // Serialize as a dict with `hi` and `lo` keys (u256 in Cairo)
            let mut dict = serde_json::Map::new();
            dict.insert("hi".to_string(), hi_json_number.into());
            dict.insert("lo".to_string(), lo_json_number.into());
            seq.serialize_element(&dict)?;
        }
    }
    seq.end()
}

fn num_str_to_json_number<S>(num_str: &str) -> Result<serde_json::Number, S::Error>
where
    S: Serializer,
{
    let bigint = BigInt::from_str_radix(num_str, 16)
        .map_err(|e| serde::ser::Error::custom(format!("Failed to parse BigInt: {}", e)))?;
    let json_number = serde_json::Number::from_str(&bigint.to_string())
        .map_err(|e| serde::ser::Error::custom(format!("Failed to serialize BigInt: {}", e)))?;
    Ok(json_number)
}
