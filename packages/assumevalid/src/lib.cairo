use consensus::types::block::{Block, BlockHash, Header, TransactionData};
use consensus::types::chain_state::{ChainState, ChainStateHashTrait};
use consensus::validation::header::validate_block_header;
use core::box::BoxImpl;
use stwo_cairo_air::{CairoProof, VerificationOutput, get_verification_output, verify_cairo};
use utils::blake2s_hasher::{Blake2sDigestFromU256, Blake2sDigestIntoU256};
use utils::mmr::{MMR, MMRTrait};


#[derive(Drop, Serde)]
struct Args {
    /// Current (initial) chain state.
    chain_state: ChainState,
    /// Batch of blocks that have to be applied to the current chain state.
    blocks: Array<Block>,
    /// Merkle Mountain Range of the block hashes.
    block_mmr: MMR,
    /// Proof of the previous chain state transition.
    /// If set to None, the chain state is assumed to be the genesis state.
    chain_state_proof: Option<CairoProof>,
}

#[derive(Drop, Serde)]
struct Result {
    /// Hash of the chain state after the blocks have been applied.
    chain_state_hash: u256,
    /// Hash of the roots of the Merkle Mountain Range of the block hashes.
    block_mmr_hash: u256,
    /// Hash of the bootloader program that was recursively verified.
    bootloader_hash: felt252,
    /// Hash of the program that was recursively verified.
    /// We cannot know the hash of the program from within the program, so we have to carry it over.
    /// This also allows composing multiple programs (e.g. if we'd need to upgrade at a certain
    /// block height).
    program_hash: felt252,
}

#[derive(Drop, Serde)]
struct BootloaderOutput {
    /// Number of tasks (must be always 1)
    n_tasks: usize,
    /// Size of the task output in felts (including the size field)
    task_output_size: usize,
    /// Hash of the payload program.
    task_program_hash: felt252,
    /// Output of the payload program.
    task_result: Result,
}

#[executable]
fn main(args: Args) -> Result {
    let Args { chain_state, blocks, chain_state_proof, block_mmr } = args;

    let mut prev_result = if let Some(proof) = chain_state_proof {
        let res = get_prev_result(proof);
        // Check that the provided chain state matches the final state hash of the previous run.
        assert(
            res.chain_state_hash == chain_state.blake2s_digest().into(), 'Invalid initial state',
        );
        // Check that the provided block MMR hash matches the hash of the block MMR
        assert(res.block_mmr_hash == block_mmr.blake2s_digest().into(), 'Invalid block MMR hash');
        res
    } else {
        assert(chain_state == Default::default(), 'Invalid genesis state');
        assert(block_mmr == genesis_block_mmr(), 'Invalid genesis block MMR');
        Result {
            chain_state_hash: chain_state.blake2s_digest().into(),
            block_mmr_hash: block_mmr.blake2s_digest().into(),
            bootloader_hash: 0,
            program_hash: 0,
        }
    };

    let mut current_chain_state = chain_state;
    let mut current_block_mmr = BoxImpl::new(block_mmr);

    // Validate the blocks and update the current chain state
    for block in blocks {
        // Update the block MMR
        let prev_block_hash = current_chain_state.best_block_hash;
        let merkle_root = match block.data {
            TransactionData::MerkleRoot(root) => root,
            TransactionData::Transactions(_) => panic!("Expected Merkle root"),
        };
        current_block_mmr =
            BoxImpl::new(
                current_block_mmr.add(block.header.blake2s_digest(prev_block_hash, merkle_root)),
            );

        // Validate the block header
        match validate_block_header(current_chain_state, block) {
            Ok(new_chain_state) => { current_chain_state = new_chain_state; },
            Err(err) => panic!("FAIL: error='{}'", err),
        };
    }

    println!("OK");

    Result {
        chain_state_hash: current_chain_state.blake2s_digest().into(),
        block_mmr_hash: current_block_mmr.blake2s_digest().into(),
        bootloader_hash: prev_result.bootloader_hash,
        program_hash: prev_result.program_hash,
    }
}

/// Verify Cairo proof, extract and validate the task output.
fn get_prev_result(proof: CairoProof) -> Result {
    let VerificationOutput { program_hash, output } = get_verification_output(proof: @proof);

    // Verify the proof
    match verify_cairo(proof) {
        Ok(_) => {},
        Err(e) => panic!("Invalid proof: {:?}", e),
    }

    // Deserialize the bootloader output
    let mut serialized_bootloader_output = output.span();
    let BootloaderOutput {
        n_tasks, task_output_size, task_program_hash, task_result,
    }: BootloaderOutput =
        Serde::deserialize(ref serialized_bootloader_output).expect('Invalid bootloader output');

    // Check that the bootloader output contains exactly one task
    assert(serialized_bootloader_output.is_empty(), 'Output too long');
    assert(n_tasks == 1, 'Unexpected number of tasks');
    assert(
        task_output_size == 8, 'Unexpected task output size',
    ); // 1 felt for program hash, 6 for output, 1 for the size

    // Check that the task bootloader hash and program hash is the same as
    // the previous bootloader hash and program hash. In case of the genesis state,
    // the previous hash is 0

    if task_result.bootloader_hash != 0 {
        assert(task_result.bootloader_hash == program_hash, 'Bootloader hash mismatch')
    }
    if task_result.program_hash != 0 {
        assert(task_result.program_hash == task_program_hash, 'Program hash mismatch');
    }

    Result {
        chain_state_hash: task_result.chain_state_hash,
        block_mmr_hash: task_result.block_mmr_hash,
        bootloader_hash: program_hash,
        program_hash: task_program_hash,
    }
}

/// Create MMR at height 0 (after adding genesis block to the accumulator).
fn genesis_block_mmr() -> MMR {
    let genesis_block_header = Header {
        version: 1, time: 1231006505, bits: 486604799, nonce: 2083236893,
    };
    let merkle_root = 0x4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b_u256
        .into();
    let prev_block_hash = 0_u256.into();
    let root = genesis_block_header.blake2s_digest(prev_block_hash, merkle_root);
    MMRTrait::new(array![Some(root)])
}

#[cfg(test)]
mod tests {
    use utils::blake2s_hasher::{Blake2sDigest, Blake2sDigestIntoU256, Blake2sDigestPartialEq};
    use super::*;

    #[test]
    fn test_genesis_block_mmr() {
        let mmr = genesis_block_mmr();
        let expected: Span<Option<Blake2sDigest>> = array![
            Some(0x5fd720d341e64d17d3b8624b17979b0d0dad4fc17d891796a3a51a99d3f41599_u256.into()),
            None,
        ]
            .span();
        assert_eq!(mmr.roots, expected, "genesis block MMR is not correct");
    }
}
