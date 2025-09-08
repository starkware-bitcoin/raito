//! Types representing the compressed SPV proof and helpers to decode Cairo outputs
//! and compute chain state digests used during verification.

use bitcoin::hashes::Hash;
use bitcoin::{block::Header as BlockHeader, BlockHash, Transaction};
use cairo_air::CairoProof;
use raito_spv_core::block_mmr::BlockInclusionProof;
use serde::{Deserialize, Serialize};
use starknet_ff::FieldElement;
use stwo_prover::core::vcs::blake2_hash::Blake2sHasher;
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
    pub total_work: String,
    /// The hash of the best block in the chain
    pub best_block_hash: BlockHash,
    /// The current target difficulty as a compact decimal string
    pub current_target: String,
    /// The start time (UNIX seconds) of the current difficulty epoch
    pub epoch_start_time: u32,
    /// The timestamps (UNIX seconds) of the previous 11 blocks
    pub prev_timestamps: Vec<u32>,
}

/// Output of the bootloader program
#[derive(Debug, Clone)]
pub struct BootloaderOutput {
    /// Number of tasks (must be always 1)
    pub n_tasks: u32,
    /// Size of the task output in felts (including the size field)
    pub task_output_size: u32,
    /// Hash of the payload program.
    pub task_program_hash: String,
    /// Output of the payload program.
    pub task_result: TaskResult,
}

/// Output of the payload program
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// Hash of the chain state after the blocks have been applied.
    pub chain_state_hash: String,
    /// Hash of the roots of the Merkle Mountain Range of the block hashes.
    pub block_mmr_hash: String,
    /// Hash of the previous bootloader program that was recursively verified.
    /// We do not hardcode the bootloader hash in the assumevalid program,
    /// letting the final verifier to check that it is as expected.
    pub bootloader_hash: String,
    /// Hash of the assumevalid program that was recursively verified.
    /// We cannot know the hash of the program from within the program, so we have to carry it over.
    /// This also allows composing multiple programs (e.g. if we'd need to upgrade at a certain
    /// block height).
    pub program_hash: String,
}

impl BootloaderOutput {
    /// Decode `BootloaderOutput` from the Cairo public output felts emitted by the bootloader.
    pub fn decode(mut output: Vec<FieldElement>) -> anyhow::Result<Self> {
        let n_tasks = output
            .remove(0)
            .try_into()
            .map_err(|_| anyhow::anyhow!("Expected number of tasks to be a u32"))?;
        let task_output_size = output
            .remove(0)
            .try_into()
            .map_err(|_| anyhow::anyhow!("Expected task output size to be a u32"))?;
        let task_program_hash = decode_truncated_hash(&mut output)?;
        let task_result = TaskResult::decode(output)?;
        Ok(Self {
            n_tasks,
            task_output_size,
            task_program_hash,
            task_result,
        })
    }
}

impl TaskResult {
    /// Decode `TaskResult` from the Cairo public output felts emitted by the payload program.
    pub fn decode(mut output: Vec<FieldElement>) -> anyhow::Result<Self> {
        let chain_state_hash = decode_truncated_hash(&mut output)?;
        let block_mmr_hash = decode_truncated_hash(&mut output)?;
        let bootloader_hash = decode_truncated_hash(&mut output)?;
        let program_hash = decode_truncated_hash(&mut output)?;
        Ok(Self {
            chain_state_hash,
            block_mmr_hash,
            bootloader_hash,
            program_hash,
        })
    }
}

impl ChainState {
    /// Compute the Blake2s digest of the chain state.
    pub fn blake2s_digest(&self) -> anyhow::Result<String> {
        let mut hasher = Blake2sHasher::new();
        hasher.update(&self.block_height.to_le_bytes());
        hasher.update(&self.total_work.as_bytes());
        hasher.update(&self.best_block_hash.to_byte_array());
        hasher.update(&self.current_target.as_bytes());
        hasher.update(&self.epoch_start_time.to_le_bytes());
        for timestamp in &self.prev_timestamps {
            hasher.update(&timestamp.to_le_bytes());
        }
        let digest = hasher.finalize();
        Ok(format!("0x{}", hex::encode(digest)))
    }
}

/// Decode a truncated hash from a list of Cairo field elements.
///
/// The hash is encoded as a sequence of field elements, each representing 8 bytes.
/// The last element may be truncated if the hash length is not a multiple of 8.
fn decode_truncated_hash(output: &mut Vec<FieldElement>) -> anyhow::Result<String> {
    let mut hash_bytes = Vec::new();
    for felt in output.drain(..) {
        let bytes = felt.to_bytes_be();
        hash_bytes.extend_from_slice(&bytes);
    }
    // Remove leading zeros
    while hash_bytes.first() == Some(&0) {
        hash_bytes.remove(0);
    }
    Ok(format!("0x{}", hex::encode(hash_bytes)))
} 