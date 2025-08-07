use bitcoin::{block::Header as BlockHeader, BlockHash, Target, Transaction, Work};
use cairo_air::CairoProof;
use raito_spv_core::block_mmr::BlockInclusionProof;
use serde::{Deserialize, Serialize};
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;

#[derive(Serialize, Deserialize)]
pub struct CompressedSpvProof {
    /// The current state of the chain
    pub chain_state: ChainState,
    /// Block MMR root hash
    pub block_mmr_root: Vec<u8>,
    /// Recursive STARK proof of the chain state and block MMR root validity
    pub chain_state_proof: CairoProof<Blake2sMerkleHasher>,
    /// The header of the block containing the transaction
    pub block_header: BlockHeader,
    /// MMR inclusion proof for the block header
    pub block_header_proof: BlockInclusionProof,
    /// The transaction to be proven
    pub transaction: Transaction,
    /// Encoded [PartialMerkleTree] structure, contains Merkle branch for the transaction
    pub transaction_proof: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainState {
    /// The height of the best block in the chain
    pub block_height: u32,
    /// The total work of the chain
    pub total_work: Work,
    /// The hash of the best block in the chain
    pub best_block_hash: BlockHash,
    /// The current target difficulty
    pub current_target: Target,
    /// The start time of the current epoch
    pub epoch_start_time: u32,
    /// The timestamps of the previous 11 blocks
    pub prev_timestamps: Vec<u32>,
}
