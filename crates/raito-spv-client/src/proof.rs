//! Types representing the compressed SPV proof and helpers to decode Cairo outputs
//! and compute chain state digests used during verification.

use std::str::FromStr;

use bitcoin::hashes::Hash;
use bitcoin::{block::Header as BlockHeader, BlockHash, Transaction};
use cairo_air::CairoProof;
use num_bigint::BigUint;
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
    /// Decode `TaskResult` from the remainder of the Cairo public output felts.
    pub fn decode(mut output: Vec<FieldElement>) -> anyhow::Result<Self> {
        let chain_state_hash = decode_hash(&mut output)?;
        let block_mmr_hash = decode_hash(&mut output)?;
        let prev_bootloader_hash = decode_truncated_hash(&mut output)?;
        let prev_program_hash = decode_truncated_hash(&mut output)?;
        Ok(Self {
            chain_state_hash,
            block_mmr_hash,
            bootloader_hash: prev_bootloader_hash,
            program_hash: prev_program_hash,
        })
    }
}

fn decode_hash(output: &mut Vec<FieldElement>) -> anyhow::Result<String> {
    // In Cairo serde u256 low goes first, high goes second
    let lo: u128 = output.remove(0).try_into().unwrap();
    let hi: u128 = output.remove(0).try_into().unwrap();
    let bytes = [hi.to_be_bytes(), lo.to_be_bytes()].concat();
    Ok(format!("0x{}", hex::encode(bytes)))
}

fn decode_truncated_hash(output: &mut Vec<FieldElement>) -> anyhow::Result<String> {
    let bytes = output.remove(0).to_bytes_be();
    Ok(format!("0x{}", hex::encode(bytes)))
}

impl ChainState {
    /// Compute the Blake2s digest of the canonical serialization of the chain state.
    ///
    /// The serialization mirrors the Cairo-side little-endian encoding.
    pub fn blake2s_digest(&self) -> anyhow::Result<String> {
        let best_block_hash_words = self
            .best_block_hash
            .as_byte_array()
            .chunks_exact(4)
            .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<_>>();

        // Construct the payload for the hash function, all integers are little-endian
        let mut words = Vec::new();
        words.push(self.block_height);
        words.extend_from_slice(&big_uint_to_u256_words(&self.total_work)?);
        words.extend_from_slice(&best_block_hash_words);
        words.extend_from_slice(&big_uint_to_u256_words(&self.current_target)?);
        words.push(self.epoch_start_time);
        words.extend_from_slice(&self.prev_timestamps);

        // Serialize to bytes, using little-endian encoding
        let bytes = words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>();

        // Compute the hash
        let mut hasher = Blake2sHasher::new();
        hasher.update(&bytes);
        let mut digest_bytes = hasher.finalize().0.to_vec();

        // Reverse bytes in each 4-byte chunk, to comply with Cairo's little-endian encoding
        digest_bytes.chunks_exact_mut(4).for_each(|chunk| {
            chunk.reverse();
        });
        let res = format!("0x{}", hex::encode(digest_bytes));
        Ok(res)
    }
}

fn big_uint_to_u256_words(value: &str) -> Result<Vec<u32>, anyhow::Error> {
    let number = BigUint::from_str(value).map_err(|_| anyhow::anyhow!("Invalid number"))?;
    let mut digits = number.to_u32_digits();
    digits.extend(vec![0; 8 - digits.len()]);
    digits.reverse();
    Ok(digits)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_chain_state_hash() {
        let chain_state = ChainState {
            block_height: 0,
            total_work: "4295032833".to_string(),
            best_block_hash: BlockHash::from_str(
                "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            )
            .unwrap(),
            current_target: "26959535291011309493156476344723991336010898738574164086137773096960"
                .to_string(),
            epoch_start_time: 1231006505,
            prev_timestamps: vec![1231006505],
        };
        let res = chain_state.blake2s_digest().unwrap();
        let expected = "0x6002eaa4410bd0b15e778656f84fc895fd091827e27ce697ba4231076c70c43b";
        assert_eq!(res, expected);
    }

    #[test]
    fn test_decode_hash() {
        let mut output = vec![
            FieldElement::from_dec_str("336341903543133962954146260045611975739").unwrap(),
            FieldElement::from_dec_str("127621031286465709630765493168293005461").unwrap(),
        ];
        let res = decode_hash(&mut output).unwrap();
        let expected = "0x6002eaa4410bd0b15e778656f84fc895fd091827e27ce697ba4231076c70c43b";
        assert_eq!(res, expected);
    }
}
