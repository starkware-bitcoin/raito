use consensus::types::block::Block;
use consensus::types::chain_state::{ChainState, ChainStateHashTrait};
use consensus::validation::header::validate_block_header;

#[derive(Drop, Serde)]
struct Result {
    prev_block_height: u32,
    block_height: u32,
    prev_chain_state_hash: felt252,
    /// Hash of the chain state after the blocks have been applied.
    chain_state_hash: felt252
}

#[derive(Drop, Serde)]
struct Args {
    /// Current (initial) chain state.
    chain_state: ChainState,
    /// Batch of blocks that have to be applied to the current chain state.
    blocks: Array<Block>    
}


#[executable]
fn main(args: Args) -> Result {
    let Args { chain_state, blocks } = args;        

    let mut current_chain_state = chain_state;
    // Validate the blocks and update the current chain state
    for block in blocks {
        match validate_block_header(current_chain_state, block) {
            Ok(new_chain_state) => { current_chain_state = new_chain_state; },
            Err(err) => panic!("Error: '{}'", err),
        }
    }
    
    Result {
        prev_block_height: chain_state.block_height,
        block_height: current_chain_state.block_height,
        prev_chain_state_hash: chain_state.hash(),
        chain_state_hash: current_chain_state.hash()        
    }
}
