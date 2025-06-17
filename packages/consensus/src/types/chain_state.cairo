//! Chain state is a minimal subset of data required to unambiguously
//! define a particular blockchain starting at the genesis.
//!
//! Chain state alone is not enough to do full block validation, however
//! it is sufficient to validate block headers.

use core::fmt::{Display, Error, Formatter};
use core::hash::{Hash, HashStateExTrait, HashStateTrait};
use core::poseidon::PoseidonTrait;
use utils::hash::Digest;

/// Represents the state of the blockchain.
#[derive(Drop, Copy, Debug, PartialEq, Serde, Hash)]
pub struct ChainState {
    /// Height of the current block.
    pub block_height: u32,
    /// Total work done.
    pub total_work: u256,
    /// Best block.
    pub best_block_hash: Digest,
    /// Current target.
    pub current_target: u256,
    /// Start of the current epoch.
    pub epoch_start_time: u32,
    /// List of 11 most recent block timestamps (from oldest to newest).
    ///
    /// Note that timestamps *do not* influence the order of blocks, i.e.
    /// it's possible that one block could have an earlier timestamp
    /// than a block that came before it in the chain.
    pub prev_timestamps: Span<u32>,
}

/// `ChainState` Poseidon hash implementation.
#[generate_trait]
pub impl ChainStateHashImpl of ChainStateHashTrait {
    fn hash(self: @ChainState) -> felt252 {
        PoseidonTrait::new().update_with(*self).finalize()
    }
}

/// `Default` implementation of `ChainState` representing the initial state after genesis block.
/// https://github.com/bitcoin/bitcoin/blob/ee367170cb2acf82b6ff8e0ccdbc1cce09730662/src/kernel/chainparams.cpp#L99
impl ChainStateDefault of Default<ChainState> {
    fn default() -> ChainState {
        ChainState {
            block_height: 0,
            total_work: 4295032833,
            best_block_hash: 0x000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f_u256
                .into(),
            current_target: 0x00000000ffff0000000000000000000000000000000000000000000000000000_u256,
            epoch_start_time: 1231006505,
            prev_timestamps: [1231006505].span(),
        }
    }
}

/// `Display` trait implementation for `ChainState`.
impl ChainStateDisplay of Display<ChainState> {
    fn fmt(self: @ChainState, ref f: Formatter) -> Result<(), Error> {
        let mut prev_ts: ByteArray = Default::default();
        for ts in *self.prev_timestamps {
            prev_ts.append(@format!("{},", ts));
        }
        let str: ByteArray = format!(
            "
	block_height: {}
	total_work: {}
	best_block_hash: {}
	current_target: {}
	epoch_start_time: {}
	prev_timestamps: [{}]
}}",
            *self.block_height,
            *self.total_work,
            *self.best_block_hash,
            *self.current_target,
            *self.epoch_start_time,
            @prev_ts,
        );
        f.buffer.append(@str);
        Result::Ok(())
    }
}

/// `Hash` trait implementation for `Span<T>` where T implements `Hash` and `Copy`.
/// Required for `ChainState` to be `Hash`able.
impl SpanHash<S, +HashStateTrait<S>, +Drop<S>, T, +Hash<T, S>, +Copy<T>> of Hash<Span<T>, S> {
    fn update_state(state: S, value: Span<T>) -> S {
        let mut state = state;
        for element in value {
            state = state.update_with(*element);
        }
        state
    }
}
