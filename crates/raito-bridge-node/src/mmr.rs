use std::path::Path;
use std::sync::Arc;
use tokio::fs;

use accumulators::hasher::stark_blake::StarkBlakeHasher;
use accumulators::hasher::Hasher;
use accumulators::mmr::{PeaksOptions, MMR};
use accumulators::store::memory::InMemoryStore;
use accumulators::store::sqlite::SQLiteStore;
use accumulators::store::Store;
use bitcoin::block::Header as BlockHeader;
use bitcoin::hashes::Hash;

/// MMR accumulator state
#[derive(Debug)]
pub struct Accumulator {
    hasher: Arc<dyn Hasher>,
    #[allow(dead_code)]
    store: Arc<dyn Store>,
    mmr: MMR,
}

impl Default for Accumulator {
    fn default() -> Self {
        let store = Arc::new(InMemoryStore::default());
        let hasher = Arc::new(StarkBlakeHasher::default());
        Self::new(store, hasher, None)
    }
}

impl Accumulator {
    /// Create a new default MMR
    pub fn new(store: Arc<dyn Store>, hasher: Arc<dyn Hasher>, mmr_id: Option<String>) -> Self {
        let mmr = MMR::new(store.clone(), hasher.clone(), mmr_id);
        Self { hasher, store, mmr }
    }

    /// Create MMR from file
    pub async fn from_file(path: &Path, mmr_id: &str) -> Result<Self, anyhow::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let store =
            Arc::new(SQLiteStore::new(path.to_str().unwrap(), Some(true), Some(mmr_id)).await?);
        let hasher = Arc::new(StarkBlakeHasher::default());
        Ok(Self::new(store, hasher, Some(mmr_id.to_string())))
    }

    /// Add a leaf to the MMR
    pub async fn add(&mut self, leaf: String) -> anyhow::Result<()> {
        self.mmr.append(leaf).await?;
        Ok(())
    }

    pub async fn add_block_header(&mut self, block_header: BlockHeader) -> anyhow::Result<()> {
        let leaf = block_header_digest(self.hasher.clone(), block_header)?;
        self.add(leaf).await
    }

    pub async fn get_block_count(&self) -> anyhow::Result<u32> {
        self.mmr
            .leaves_count
            .get()
            .await
            .map(|v| v as u32)
            .map_err(|e| anyhow::anyhow!("Failed to get block count: {}", e))
    }

    /// Get the roots of the MMR in sparse format (compatible with Cairo implementation)
    pub async fn get_sparse_roots(&self) -> anyhow::Result<Vec<String>> {
        let mut elements_count = self.mmr.elements_count.get().await?;
        let roots = self
            .mmr
            .get_peaks(PeaksOptions {
                elements_count: Some(elements_count),
                formatting_opts: None,
            })
            .await?;

        let null_root = format!("0x{:064x}", 0);

        let mut max_height = elements_count.ilog2() + 1;
        let mut root_idx = 0;
        let mut result = vec![];

        while elements_count != 0 || max_height != 0 {
            // Number of elements of the perfect binary tree of the current max height
            let elements_per_height = (1 << max_height) - 1;
            if elements_count >= elements_per_height {
                result.insert(0, roots[root_idx].clone());
                root_idx += 1;
                elements_count -= elements_per_height;
            } else {
                result.insert(0, null_root.clone());
            }
            if max_height != 0 {
                max_height -= 1;
            }
        }

        if result.last().unwrap() != &null_root {
            result.push(null_root);
        }

        Ok(result)
    }
}

pub fn block_header_digest(
    hasher: Arc<dyn Hasher>,
    block_header: BlockHeader,
) -> anyhow::Result<String> {
    let data = vec![
        hex::encode(&block_header.version.to_consensus().to_be_bytes()),
        hex::encode(&block_header.prev_blockhash.to_byte_array()),
        hex::encode(&block_header.merkle_root.to_byte_array()),
        hex::encode(&block_header.time.to_be_bytes()),
        hex::encode(&block_header.bits.to_consensus().to_be_bytes()),
        hex::encode(&block_header.nonce.to_be_bytes()),
    ]
    .into_iter()
    .map(|s| format!("0x{}", s))
    .collect();
    hasher
        .hash(data)
        .map_err(|e| anyhow::anyhow!("Failed to hash block header: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mmr_add() {
        let mut mmr = Accumulator::default();
        let leaf = "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66".to_string();

        // Add first leaf
        mmr.add(leaf.clone()).await.unwrap();
        let roots = mmr.get_sparse_roots().await.unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(
            roots[0],
            "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66"
        );
        assert_eq!(
            roots[1],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );

        // Add second leaf
        mmr.add(leaf.clone()).await.unwrap();
        let roots = mmr.get_sparse_roots().await.unwrap();
        assert_eq!(roots.len(), 3);
        assert_eq!(
            roots[0],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            roots[1],
            "0x693aa1ab81c6362fe339fc4c7f6d8ddb1e515701e58c5bb2fb54a193c8287fdc"
        );
        assert_eq!(
            roots[2],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );

        // Add third leaf
        mmr.add(leaf.clone()).await.unwrap();
        let roots = mmr.get_sparse_roots().await.unwrap();
        assert_eq!(roots.len(), 3);
        assert_eq!(
            roots[0],
            "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66"
        );
        assert_eq!(
            roots[1],
            "0x693aa1ab81c6362fe339fc4c7f6d8ddb1e515701e58c5bb2fb54a193c8287fdc"
        );
        assert_eq!(
            roots[2],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );

        // Add fourth leaf
        mmr.add(leaf.clone()).await.unwrap();
        let roots = mmr.get_sparse_roots().await.unwrap();
        assert_eq!(roots.len(), 4);
        assert_eq!(
            roots[0],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            roots[1],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            roots[2],
            "0x488a5ed31744187c70a57c092e2c86742518ec5acea240726789d8b1af2b1e0d"
        );
        assert_eq!(
            roots[3],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );

        // Add fifth leaf
        mmr.add(leaf.clone()).await.unwrap();
        let roots = mmr.get_sparse_roots().await.unwrap();
        assert_eq!(roots.len(), 4);
        assert_eq!(
            roots[0],
            "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66"
        );
        assert_eq!(
            roots[1],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            roots[2],
            "0x488a5ed31744187c70a57c092e2c86742518ec5acea240726789d8b1af2b1e0d"
        );
        assert_eq!(
            roots[3],
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_block_header_blake_digest() {
        let hasher = Arc::new(StarkBlakeHasher::default());
        let block_header: BlockHeader = serde_json::from_str(
            r#"
            {
                "version": 1,
                "prev_blockhash": "000000002a22cfee1f2c846adbd12b3e183d4f97683f85dad08a79780a84bd55",
                "merkle_root": "7dac2c5666815c17a3b36427de37bb9d2e2c5ccec3f8633eb91a4205cb4c10ff",
                "time": 1231731025,
                "bits": 486604799,
                "nonce": 1889418792
            }
            "#,
        )
        .unwrap();
        let digest = block_header_digest(hasher, block_header).unwrap();
        assert_eq!(
            digest,
            "0x50b005dd2964720fcd066875bc1cf13a06703a5c8efe8b02a1fd7ea902050f09"
        );
    }

    #[test]
    fn test_block_header_blake_digest_genesis() {
        let hasher = Arc::new(StarkBlakeHasher::default());
        let block_header: BlockHeader = serde_json::from_str(
            r#"
            {
                "version": 1,
                "prev_blockhash": "0000000000000000000000000000000000000000000000000000000000000000",
                "merkle_root": "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
                "time": 1231006505,
                "bits": 486604799,
                "nonce": 2083236893
            }
            "#,
        )
        .unwrap();
        let digest = block_header_digest(hasher, block_header).unwrap();
        assert_eq!(
            digest,
            "0x5fd720d341e64d17d3b8624b17979b0d0dad4fc17d891796a3a51a99d3f41599"
        );
    }
}
