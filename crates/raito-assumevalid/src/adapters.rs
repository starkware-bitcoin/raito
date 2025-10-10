use bitcoin::block::Header;

use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_serialize::CairoSerialize;

use bitcoin::hashes::Hash;
use raito_cairo_serialize::{DigestString, U256String};
use raito_spv_mmr::sparse_roots::SparseRoots;
use raito_spv_verify::ChainState;

use cairo_air::CairoProof;

/// View for assumevalid Args struct that matches Cairo's structure
#[derive(CairoSerialize)]
struct AssumeValidArgsView {
    chain_state: ChainStateView,
    blocks: Vec<BlockView>,
    block_mmr: SparseRootsView,
    chain_state_proof: Option<CairoProof<Blake2sMerkleHasher>>,
}

/// View matching Cairo `ChainState` layout
#[derive(CairoSerialize)]
struct ChainStateView {
    block_height: u32,
    total_work: U256String,
    best_block_hash: DigestString,
    current_target: U256String,
    epoch_start_time: u32,
    prev_timestamps: Vec<u32>,
}

#[derive(CairoSerialize)]
pub struct SparseRootsView {
    pub roots: Vec<U256String>,
}

/// View for a single block matching Cairo's Block structure
#[derive(CairoSerialize)]
struct BlockView {
    header: HeaderView,
    data: Option<DigestString>,
}

/// Reuse HeaderView from header module
#[derive(CairoSerialize)]
struct HeaderView {
    pub version: u32,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
}

/// Main adapter function for assumevalid Args
pub fn to_runner_args_hex(
    chain_state: ChainState,
    headers: &[Header],
    block_mmr: &SparseRoots,
    chain_state_proof: Option<CairoProof<Blake2sMerkleHasher>>,
) -> Vec<String> {
    // Convert headers to BlockView (merkle_root is already in the header)
    let blocks: Vec<BlockView> = headers
        .iter()
        .map(|header| BlockView {
            header: HeaderView {
                version: header.version.to_consensus() as u32,
                time: header.time,
                bits: header.bits.to_consensus(),
                nonce: header.nonce,
            },
            data: Some(DigestString(hex::encode(
                header.merkle_root.as_byte_array(),
            ))),
        })
        .collect();

    let chain_state_view = ChainStateView {
        block_height: chain_state.block_height,
        total_work: U256String(chain_state.total_work.to_string()),
        best_block_hash: DigestString(chain_state.best_block_hash.to_string()),
        current_target: U256String(chain_state.current_target.to_string()),
        epoch_start_time: chain_state.epoch_start_time,
        prev_timestamps: chain_state.prev_timestamps.clone(),
    };

    let block_mmr_view = SparseRootsView {
        roots: block_mmr
            .roots
            .iter()
            .map(|root| U256String(root.to_string()))
            .collect(),
    };

    let args_view = AssumeValidArgsView {
        chain_state: chain_state_view,
        blocks,
        block_mmr: block_mmr_view,
        chain_state_proof,
    };

    let mut felts = Vec::new();
    args_view.serialize(&mut felts);

    felts
        .into_iter()
        .map(|felt| format!("0x{felt:x}"))
        .collect()
}
