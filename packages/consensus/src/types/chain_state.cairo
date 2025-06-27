//! Chain state is a minimal subset of data required to unambiguously
//! define a particular blockchain starting at the genesis.
//!
//! Chain state alone is not enough to do full block validation, however
//! it is sufficient to validate block headers.

use core::fmt::{Display, Error, Formatter};
use core::hash::{Hash, HashStateExTrait, HashStateTrait};
use utils::blake2s_hasher::{Blake2sHasher, blake2s_digest_to_u256_le};
use utils::hash::Digest;
use utils::numeric::u256_to_u32x8;

/// Represents the state of the blockchain.
#[derive(Drop, Copy, Debug, PartialEq, Serde)]
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
    /// Returns the Blake2s digest of the chain state.
    /// NOTE: returned u256 value is little-endian.
    fn blake2s_digest(self: @ChainState) -> u256 {
        let mut hasher = Blake2sHasher::new();

        // TODO(m-kus): reorder the fields to make the structure more aligned?
        let a0 = *self.block_height;
        let [a1, a2, a3, a4, a5, a6, a7, a8] = u256_to_u32x8(*self.total_work);
        let [a9, a10, a11, a12, a13, a14, a15, b0] = *self.best_block_hash.value;

        // Compress the first block
        hasher
            .compress_block([a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15]);

        let [b1, b2, b3, b4, b5, b6, b7, b8] = u256_to_u32x8(*self.current_target);
        let b9 = *self.epoch_start_time;

        let mut prev_timestamps = *self.prev_timestamps;
        let res = if let Some(tail) = prev_timestamps.multi_pop_front::<6>() {
            let [b10, b11, b12, b13, b14, b15] = tail.unbox();
            // Compress the second block
            hasher
                .compress_block(
                    [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15],
                );
            // Finalize the hash digest
            hasher.finalize_block(prev_timestamps)
        } else {
            let mut buffer = array![b0, b1, b2, b3, b4, b5, b6, b7, b8, b9];
            buffer.append_span(prev_timestamps);
            // Finalize the hash digest
            hasher.finalize_block(buffer.span())
        };

        blake2s_digest_to_u256_le(res)
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
