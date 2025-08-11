//! Merkle Mountain Range (MMR) accumulator implementation for Bitcoin block headers with proof generation.

use std::path::Path;
use std::sync::Arc;
use tokio::fs;

use accumulators::hasher::stark_blake::StarkBlakeHasher;
use accumulators::hasher::Hasher;
use accumulators::mmr::{
    elements_count_to_leaf_count, leaf_count_to_mmr_size, map_leaf_index_to_element_index,
    PeaksOptions, Proof, ProofOptions, MMR,
};
use accumulators::store::memory::InMemoryStore;
use accumulators::store::sqlite::SQLiteStore;
use accumulators::store::Store;
use bitcoin::block::Header as BlockHeader;
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

use crate::sparse_roots::SparseRoots;

/// MMR accumulator state for Bitcoin block headers
#[derive(Debug)]
pub struct BlockMMR {
    hasher: Arc<dyn Hasher>,
    #[allow(dead_code)]
    store: Arc<dyn Store>,
    mmr: MMR,
}

/// Proof data structure for demonstrating inclusion of a block in the MMR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInclusionProof {
    /// MMR peak hashes at the time of proof generation
    pub peaks_hashes: Vec<String>,
    /// Sibling hashes needed to reconstruct the path to the root
    pub siblings_hashes: Vec<String>,
    /// Leaf index of the block in the MMR (same as block height)
    pub leaf_index: usize,
    /// Total number of leaves in the MMR
    pub leaf_count: usize,
}

/// Default accumulator is an in-memory accumulator with StarkBlake hasher
impl Default for BlockMMR {
    fn default() -> Self {
        let store = Arc::new(InMemoryStore::default());
        let hasher = Arc::new(StarkBlakeHasher::default());
        Self::new(store, hasher, None)
    }
}

impl BlockMMR {
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

    /// Create in-memory MMR from peaks hashes and elements count
    pub async fn from_peaks(
        peaks_hashes: Vec<String>,
        leaf_count: usize,
    ) -> Result<Self, anyhow::Error> {
        let store = Arc::new(InMemoryStore::default());
        let hasher = Arc::new(StarkBlakeHasher::default());
        let mmr = MMR::create_from_peaks(
            store.clone(),
            hasher.clone(),
            None,
            peaks_hashes,
            leaf_count_to_mmr_size(leaf_count),
        )
        .await?;
        Ok(Self { hasher, store, mmr })
    }

    /// Add a leaf to the MMR
    pub async fn add(&mut self, leaf: String) -> anyhow::Result<()> {
        self.mmr.append(leaf).await?;
        Ok(())
    }

    /// Add a block header to the MMR
    pub async fn add_block_header(&mut self, block_header: &BlockHeader) -> anyhow::Result<()> {
        let leaf = block_header_digest(self.hasher.clone(), block_header)?;
        self.add(leaf).await
    }

    /// Get the number of blocks in the MMR (number of leaves)
    pub async fn get_block_count(&self) -> anyhow::Result<u32> {
        self.mmr
            .leaves_count
            .get()
            .await
            .map(|v| v as u32)
            .map_err(|e| anyhow::anyhow!("Failed to get block count: {}", e))
    }

    /// Get the roots of the MMR in sparse format (compatible with Cairo implementation)
    pub async fn get_sparse_roots(&self, chain_height: Option<u32>) -> anyhow::Result<SparseRoots> {
        let elements_count = match chain_height {
            Some(chain_height) => leaf_count_to_mmr_size(chain_height as usize + 1),
            None => self.mmr.elements_count.get().await?,
        };
        let roots = self
            .mmr
            .get_peaks(PeaksOptions {
                elements_count: Some(elements_count),
                formatting_opts: None,
            })
            .await?;
        SparseRoots::try_from_peaks(roots, elements_count)
    }

    /// Generate an inclusion proof for a given block height.
    /// If `block_count` is provided, the proof will be generated for a previous state of the MMR.
    pub async fn generate_proof(
        &self,
        block_height: u32,
        chain_height: Option<u32>,
    ) -> anyhow::Result<BlockInclusionProof> {
        let element_index = map_leaf_index_to_element_index(block_height as usize);
        let options = ProofOptions {
            elements_count: chain_height.map(|c| leaf_count_to_mmr_size(c as usize + 1)),
            ..Default::default()
        };
        let proof = self
            .mmr
            .get_proof(element_index, Some(options))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate proof: {}", e))?;
        let leaf_count = elements_count_to_leaf_count(proof.elements_count)?;
        Ok(BlockInclusionProof {
            peaks_hashes: proof.peaks_hashes,
            siblings_hashes: proof.siblings_hashes,
            leaf_index: block_height as usize,
            leaf_count,
        })
    }

    /// Verify an inclusion proof for a given block height and block header
    /// NOTE that this only guarantees that the block was included in the MMR with the known peaks hashes.
    /// In order to verify the correctness you have to compute the root hash of the MMR and compare it with the commitеed root.
    pub async fn verify_proof(
        &self,
        block_header: &BlockHeader,
        proof: BlockInclusionProof,
    ) -> anyhow::Result<bool> {
        let BlockInclusionProof {
            peaks_hashes,
            siblings_hashes,
            leaf_index,
            leaf_count,
        } = proof;
        let element_hash = block_header_digest(self.hasher.clone(), block_header)?;
        let proof = Proof {
            element_index: map_leaf_index_to_element_index(leaf_index),
            element_hash: element_hash.clone(),
            siblings_hashes,
            peaks_hashes,
            elements_count: leaf_count_to_mmr_size(leaf_count),
        };
        let options = ProofOptions {
            elements_count: Some(proof.elements_count),
            ..Default::default()
        };
        self.mmr
            .verify_proof(proof, element_hash, Some(options))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to verify proof: {}", e))
    }

    /// Get the root hash of the MMR (compatible with Cairo implementation)
    pub async fn get_root_hash(&self, block_count: Option<u32>) -> anyhow::Result<String> {
        let SparseRoots {
            block_height: _,
            roots,
        } = self.get_sparse_roots(block_count).await?;
        self.hasher
            .hash(roots)
            .map_err(|e| anyhow::anyhow!("Failed to get root hash: {}", e))
    }
}

/// Compute the digest of a block header using the specified hasher
///
/// # Arguments
/// * `hasher` - The hasher implementation to use
/// * `block_header` - The Bitcoin block header to hash
///
/// # Returns
/// * `String` - The hex-encoded hash digest
/// * `anyhow::Error` - If hashing fails
pub fn block_header_digest(
    hasher: Arc<dyn Hasher>,
    block_header: &BlockHeader,
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
        let mut mmr = BlockMMR::default();
        let leaf = "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66".to_string();

        // Add first leaf
        mmr.add(leaf.clone()).await.unwrap();
        let SparseRoots {
            block_height,
            roots,
        } = mmr.get_sparse_roots(None).await.unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(block_height, 0);
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
        let SparseRoots {
            block_height,
            roots,
        } = mmr.get_sparse_roots(None).await.unwrap();
        assert_eq!(roots.len(), 3);
        assert_eq!(block_height, 1);
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
        let SparseRoots {
            block_height,
            roots,
        } = mmr.get_sparse_roots(None).await.unwrap();
        assert_eq!(roots.len(), 3);
        assert_eq!(block_height, 2);
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
        let SparseRoots {
            block_height,
            roots,
        } = mmr.get_sparse_roots(None).await.unwrap();
        assert_eq!(roots.len(), 4);
        assert_eq!(block_height, 3);
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
        let SparseRoots {
            block_height,
            roots,
        } = mmr.get_sparse_roots(None).await.unwrap();
        assert_eq!(roots.len(), 4);
        assert_eq!(block_height, 4);
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
        let digest = block_header_digest(hasher, &block_header).unwrap();
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
        let digest = block_header_digest(hasher, &block_header).unwrap();
        assert_eq!(
            digest,
            "0x5fd720d341e64d17d3b8624b17979b0d0dad4fc17d891796a3a51a99d3f41599"
        );
    }

    #[tokio::test]
    async fn test_inclusion_proof() {
        let mut mmr = BlockMMR::default();
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
        // Add 10 blocks
        for _ in 0..10 {
            mmr.add_block_header(&block_header).await.unwrap();
        }
        // Generate a proof for the fifth block
        let proof = mmr.generate_proof(5, None).await.unwrap();
        // Create an ephemeral MMR from the peaks hashes and elements count
        let view_mmr = BlockMMR::from_peaks(proof.peaks_hashes.clone(), proof.leaf_count)
            .await
            .unwrap();
        // Verify the proof
        assert!(view_mmr.verify_proof(&block_header, proof).await.unwrap());

        // Generate a proof for a previous MMR state
        let proof = mmr.generate_proof(1, Some(4)).await.unwrap();
        // Create an ephemeral MMR from the peaks hashes and elements count
        let view_mmr = BlockMMR::from_peaks(proof.peaks_hashes.clone(), proof.leaf_count)
            .await
            .unwrap();
        // Verify the proof
        assert!(view_mmr.verify_proof(&block_header, proof).await.unwrap());
    }

    #[tokio::test]
    async fn test_root_hash() {
        let mut mmr = BlockMMR::default();
        let leaf = "0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66".to_string();
        // Add 15 blocks
        for _ in 0..15 {
            mmr.add(leaf.clone()).await.unwrap();
        }
        // Get the root hash
        let root_hash = mmr.get_root_hash(None).await.unwrap();
        assert_eq!(
            root_hash,
            "0x19f148fb4f9b5e5bac1c12594b8e4b2d4b94d12c073b92e2b3d83349909613b6"
        );
    }
}
