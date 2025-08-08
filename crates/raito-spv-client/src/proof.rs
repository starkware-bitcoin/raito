use bitcoin::{block::Header as BlockHeader, BlockHash, Target, Transaction, Work};
use cairo_air::CairoProof;
use raito_spv_core::block_mmr::BlockInclusionProof;
use serde::{Deserialize, Serialize};
use stwo_prover::core::vcs::blake2_merkle::Blake2sMerkleHasher;

/// A compact, self-contained proof that a Bitcoin transaction is included
/// in a specific block and that the block is part of a valid chain state.
#[derive(Serialize, Deserialize)]
pub struct CompressedSpvProof {
    /// The current state of the chain
    pub chain_state: ChainState,
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

/// Snapshot of the consensus chain state used to validate block inclusion
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainState {
    /// The height of the best block in the chain
    pub block_height: u32,
    /// The total accumulated work of the chain as a decimal string
    /// FIXME: Work
    pub total_work: String,
    /// The hash of the best block in the chain
    pub best_block_hash: BlockHash,
    /// The current target difficulty as a compact decimal string
    /// FIXME: Target
    pub current_target: String,
    /// The start time (UNIX seconds) of the current difficulty epoch
    pub epoch_start_time: u32,
    /// The timestamps (UNIX seconds) of the previous 11 blocks
    pub prev_timestamps: Vec<u32>,
}
