use consensus::types::block::Block;
use consensus::types::chain_state::{ChainState, ChainStateHashTrait};
use consensus::validation::header::validate_block_header;
use stwo_cairo_air::{CairoProof, VerificationOutput, get_verification_output, verify_cairo};

/// Hash of the bootloader program.
/// See https://github.com/m-kus/cairo-bootloader/blob/main/resources/stwo-bootloader.json
const BOOTLOADER_PROGRAM_HASH: felt252 =
    2494357015748342749807586816445211447933813471303162239123730354369899819316;

#[derive(Drop, Serde)]
struct Args {
    /// Current (initial) chain state.
    chain_state: ChainState,
    /// Batch of blocks that have to be applied to the current chain state.
    blocks: Array<Block>,
    /// Proof of the previos chain state transition.
    /// If set to None, the chain state is assumed to be the genesis state.
    chain_state_proof: Option<CairoProof>,
}

#[derive(Drop, Serde)]
struct Result {
    /// Hash of the chain state after the blocks have been applied.
    chain_state_hash: felt252,
    /// Hash of the program that was recursively verified.
    prev_program_hash: felt252,
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
    let Args { chain_state, blocks, chain_state_proof } = args;

    let mut prev_result = if let Some(proof) = chain_state_proof {
        get_prev_result(proof)
    } else {
        assert(chain_state == Default::default(), 'Invalid genesis state');
        Result { chain_state_hash: chain_state.hash(), prev_program_hash: 0 }
    };

    // Check that the provided chain state matches the final state hash of the previous run.
    assert(prev_result.chain_state_hash == chain_state.hash(), 'Invalid initial state');
    let mut current_chain_state = chain_state;

    // Validate the blocks and update the current chain state
    for block in blocks {
        match validate_block_header(current_chain_state, block) {
            Ok(new_chain_state) => { current_chain_state = new_chain_state; },
            Err(err) => panic!("Error: '{}'", err),
        }
    }

    Result {
        chain_state_hash: current_chain_state.hash(),
        prev_program_hash: prev_result.prev_program_hash,
    }
}

/// Verify Cairo proof, extract and validate the task output.
fn get_prev_result(proof: CairoProof) -> Result {
    let VerificationOutput { program_hash, output } = get_verification_output(proof: @proof);

    // Check that the program hash is the bootloader program hash
    assert(program_hash == BOOTLOADER_PROGRAM_HASH, 'Unexpected bootloader');

    // Verify the proof
    verify_cairo(proof).expect('Invalid proof');

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
        task_output_size == 4, 'Unexpected task output size',
    ); // 1 felt for program hash, 2 for output, 1 for the size

    // Check that the task program hash is the same as the previous program hash
    // In case of the genesis state, the previous program hash must be 0
    if task_result.chain_state_hash != Default::default() {
        assert(task_result.prev_program_hash == task_program_hash, 'Program hash mismatch');
    } else {
        assert(task_result.prev_program_hash == 0, 'Invalid genesis program hash');
    }

    Result { chain_state_hash: task_result.chain_state_hash, prev_program_hash: task_program_hash }
}
