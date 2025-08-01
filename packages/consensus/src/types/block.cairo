//! Bitcoin block and its components.
//!
//! The data is expected to be prepared in advance and passed as program arguments.

use core::fmt::{Display, Error, Formatter};
use utils::blake2s_hasher::{Blake2sDigest, Blake2sHasher};
use utils::double_sha256::double_sha256_word_array;
use utils::hash::Digest;
use utils::word_array::{WordArray, WordArrayTrait};
use super::transaction::Transaction;

/// Represents a block in the blockchain.
#[derive(Drop, Copy, Debug, PartialEq, Default, Serde)]
pub struct Block {
    /// Block header.
    pub header: Header,
    /// Transaction data: either merkle root or list of transactions.
    pub data: TransactionData,
}

/// Represents block contents.
#[derive(Drop, Copy, Debug, PartialEq, Serde)]
pub enum TransactionData {
    /// Merkle root of all transactions in the block.
    /// This variant is used for header-only validation mode (light client).
    MerkleRoot: Digest,
    /// List of all transactions included in the block.
    /// This variant is used for the full consensus validation mode.
    Transactions: Span<Transaction>,
}

/// Represents a block header.
/// https://learnmeabitcoin.com/technical/block/
///
/// NOTE that some of the fields are missing, that's intended.
/// The point of the client is to calculate next chain state from the previous
/// chain state and block data in a provable way.
/// The proof can be later used to verify that the chain state is valid.
/// In order to do the calculation we just need data about the block that is strictly necessary,
/// but not the data we can calculate like merkle root or data that we already have
/// like previous_block_hash (in the previous chain state).
#[derive(Drop, Copy, Debug, PartialEq, Default, Serde)]
pub struct Header {
    /// The version of the block.
    pub version: u32,
    /// The timestamp of the block.
    pub time: u32,
    /// The difficulty target for mining the block.
    /// Not strictly necessary since it can be computed from target,
    /// but it is cheaper to validate than compute.
    pub bits: u32,
    /// The nonce used in mining the block.
    pub nonce: u32,
}

#[generate_trait]
pub impl BlockHashImpl of BlockHash {
    /// Computes the hash of the block header given the missing fields.
    fn hash(self: @Header, prev_block_hash: Digest, merkle_root: Digest) -> Digest {
        let mut words: WordArray = Default::default();

        words.append_u32_le(*self.version);
        words.append_span(prev_block_hash.value.span());
        words.append_span(merkle_root.value.span());

        words.append_u32_le(*self.time);
        words.append_u32_le(*self.bits);
        words.append_u32_le(*self.nonce);

        double_sha256_word_array(words)
    }

    /// Computes the Blake2s digest of the block header.
    fn blake2s_digest(
        self: @Header, prev_block_hash: Digest, merkle_root: Digest,
    ) -> Blake2sDigest {
        let mut hasher = Blake2sHasher::new();
        let [v1, v2, v3, v4, v5, v6, v7, v8] = prev_block_hash.value;
        let [v9, v10, v11, v12, v13, v14, v15, v16] = merkle_root.value;
        hasher
            .compress_block(
                [*self.version, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15],
            );
        hasher
            .finalize_block(
                [v16, *self.time, *self.bits, *self.nonce, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 16,
            )
    }
}

/// `Default` trait implementation of `TransactionData`, i.e., empty transaction data.
pub impl TransactionDataDefault of Default<TransactionData> {
    fn default() -> TransactionData {
        TransactionData::Transactions(array![].span())
    }
}

/// `Display` trait implementation for `Block`.
impl BlockDisplay of Display<Block> {
    fn fmt(self: @Block, ref f: Formatter) -> Result<(), Error> {
        let data = match *self.data {
            TransactionData::MerkleRoot(root) => format!("{}", root),
            TransactionData::Transactions(txs) => format!("{}", txs.len()),
        };
        let str: ByteArray = format!(" Block {{ header: {}, data: {} }}", *self.header, @data);
        f.buffer.append(@str);
        Result::Ok(())
    }
}

/// `Display` trait implementation for `Header`.
impl HeaderDisplay of Display<Header> {
    fn fmt(self: @Header, ref f: Formatter) -> Result<(), Error> {
        let str: ByteArray = format!(
            "Header {{ version: {}, time: {}, bits: {}, nonce: {}}}",
            *self.version,
            *self.time,
            *self.bits,
            *self.nonce,
        );
        f.buffer.append(@str);
        Result::Ok(())
    }
}

/// `Display` trait implementation for `TransactionData`.
impl TransactionDataDisplay of Display<TransactionData> {
    fn fmt(self: @TransactionData, ref f: Formatter) -> Result<(), Error> {
        match *self {
            TransactionData::MerkleRoot(root) => f.buffer.append(@format!("MerkleRoot: {}", root)),
            TransactionData::Transactions(txs) => f
                .buffer
                .append(@format!("Transactions: {}", txs.len())),
        }
        Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use utils::blake2s_hasher::Blake2sDigestIntoU256;
    use utils::hash::Digest;
    use crate::types::chain_state::ChainState;
    use super::{BlockHash, Header};

    #[test]
    fn test_block_hash() {
        let mut chain_state: ChainState = Default::default();
        chain_state
            .best_block_hash =
                0x000000002a22cfee1f2c846adbd12b3e183d4f97683f85dad08a79780a84bd55_u256
            .into();
        // Block 170
        let header = Header {
            version: 1_u32, time: 1231731025_u32, bits: 0x1d00ffff_u32, nonce: 1889418792_u32,
        };
        let merkle_root: Digest =
            0x7dac2c5666815c17a3b36427de37bb9d2e2c5ccec3f8633eb91a4205cb4c10ff_u256
            .into();

        let block_hash_result: Digest = header.hash(chain_state.best_block_hash, merkle_root);

        //0x00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee
        let expected_block_hash: Digest =
            0x00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee_u256
            .into();

        assert_eq!(expected_block_hash, block_hash_result);
    }

    #[test]
    fn test_merkle_root_diff() {
        let mut chain_state: ChainState = Default::default();
        chain_state
            .best_block_hash =
                0x000000002a22cfee1f2c846adbd12b3e183d4f97683f85dad08a79780a84bd55_u256
            .into();
        // Block 170
        let header = Header {
            version: 1_u32, time: 1231731025_u32, bits: 0x1d00ffff_u32, nonce: 1889418792_u32,
        };
        let merkle_root: Digest =
            0x6dac2c5666815c17a3b36427de37bb9d2e2c5ccec3f8633eb91a4205cb4c10ff_u256
            .into();

        let block_hash_result: Digest = header.hash(chain_state.best_block_hash, merkle_root);

        let expected_block_hash: Digest =
            0x00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee_u256
            .into();

        assert_ne!(expected_block_hash, block_hash_result);
    }

    #[test]
    fn test_best_block_hash_diff() {
        let mut chain_state: ChainState = Default::default();
        chain_state
            .best_block_hash =
                0x000000002a22cfee1f2c846adbd12b3e183d4f97683f85dad08a79780a84bd56_u256
            .into();
        // block 170
        let header = Header {
            version: 1_u32, time: 1231731025_u32, bits: 0x1d00ffff_u32, nonce: 1889418792_u32,
        };
        let merkle_root: Digest =
            0x7dac2c5666815c17a3b36427de37bb9d2e2c5ccec3f8633eb91a4205cb4c10ff_u256
            .into();

        let block_hash_result: Digest = header.hash(chain_state.best_block_hash, merkle_root);

        let expected_block_hash: Digest =
            0x00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee_u256
            .into();

        assert_ne!(expected_block_hash, block_hash_result);
    }

    #[test]
    fn test_block_hash_blake2s() {
        let header = Header {
            version: 1_u32, time: 1231731025_u32, bits: 0x1d00ffff_u32, nonce: 1889418792_u32,
        };
        let merkle_root = 0x7dac2c5666815c17a3b36427de37bb9d2e2c5ccec3f8633eb91a4205cb4c10ff_u256
            .into();
        let best_block_hash =
            0x000000002a22cfee1f2c846adbd12b3e183d4f97683f85dad08a79780a84bd55_u256
            .into();
        let block_hash_result: u256 = header.blake2s_digest(best_block_hash, merkle_root).into();
        let expected_block_hash =
            0x50b005dd2964720fcd066875bc1cf13a06703a5c8efe8b02a1fd7ea902050f09_u256;
        assert_eq!(expected_block_hash, block_hash_result);
    }
}
