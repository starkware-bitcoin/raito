use consensus::types::block::Block;
use consensus::types::chain_state::{ChainState, ChainStateHashTrait};
use consensus::validation::header::validate_block_header;
use stwo_cairo_air::{CairoProof, VerificationOutput, get_verification_output, verify_cairo};
use utils::hash::Digest;

#[derive(Drop, Serde)]
struct Args {
    /// Current (initial) chain state.
    chain_state: ChainState,
    /// Batch of blocks that have to be applied to the current chain state.
    blocks: Array<Block>,
}

#[derive(Drop, Serde)]
struct Result {
    /// Hash of the initial chain state.
    initial_hash: felt252,
    /// Hash of the final chain state.
    final_hash: felt252,
}

#[executable]
fn agg(proof: CairoProof) -> VerificationOutput {
    let verification_output = get_verification_output(proof: @proof);

    if let Err(err) = verify_cairo(proof) {
        panic!("Verification failed: {:?}", err);
    }

    verification_output
}

#[executable]
fn main(args: Args) -> Result {
    // Force cairo-prove to use canonical PP variant
    // core::internal::require_implicit::<core::pedersen::Pedersen>();

    let Args { mut chain_state, blocks } = args;

    for block in blocks {
        match validate_block_header(chain_state, block) {
            Ok(new_chain_state) => { chain_state = new_chain_state; },
            Err(err) => panic!("Error: '{}'", err),
        }
    }

    Result { initial_hash: chain_state.hash(), final_hash: chain_state.hash() }
}


#[derive(Drop, Serde)]
struct FoldArgs {
    chain_state: ChainState,
    blocks: Array<Block>,
    chain_state_proof: Option<CairoProof>,
}

#[derive(Drop, Serde)]
struct FoldResult {
    final_block_hash: Digest,
}

#[executable]
fn fold(args: FoldArgs) -> FoldResult {
    let FoldArgs { mut chain_state, blocks, chain_state_proof } = args;

    if chain_state.block_height == 0 {
        assert!(chain_state_proof.is_none());
    } else {
        let _chain_state_proof = chain_state_proof.expect('No proof for non-genesis block!');
        // TODO: verify proof
        
        // let VerificationOutput {
        //     program_hash, output,
        // } = get_verification_output(proof: @chain_state_proof);

        // // TODO: assert on program hash
        // assert!(program_hash != 0);

        // let mut output = output.span();
        // let FoldResult {
        //     final_block_hash,
        // } = Serde::deserialize(ref output).expect('Can\'t deserialize proof output!');

        // assert!(
        //     final_block_hash == chain_state.best_block_hash,
        //     "Final block hash: {} does not match the chain_state block hash: {}!",
        //     final_block_hash,
        //     chain_state.best_block_hash,
        // );
    }

    for block in blocks {
        match validate_block_header(chain_state, block) {
            Ok(new_chain_state) => { chain_state = new_chain_state; },
            Err(err) => panic!(
                "Error while verifying block:\n{:?}\bchain_state:\n{:?}:\n'{}'",
                block,
                chain_state,
                err,
            ),
        }
    }

    FoldResult { final_block_hash: chain_state.best_block_hash }
}
