use core::blake::{blake2s_compress, blake2s_finalize};
use core::box::BoxImpl;
use utils::numeric::{u256_to_u32x8, u32x8_to_u256};

/// BLAKE2s IV is the same as SHA-256 IV
/// We modify the first word to pre-configure:
/// IV[0] ^ 0x01010020 (config: no key, 32 bytes output).
const BLAKE2S_256_INITIAL_STATE: [u32; 8] = [
    0x6B08E647, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

/// Alias for the Blake2s digest.
pub type Blake2sDigest = Box<[u32; 8]>;

/// Blake2s incremental state.
#[derive(Debug, Drop, Copy)]
pub struct Blake2sState {
    pub(crate) h: Box<[u32; 8]>,
    pub(crate) byte_len: u32,
}

#[generate_trait]
pub impl Blake2sHasherImpl of Blake2sHasher {
    /// Creates a new Blake2s hasher, initialized with the Blake2s 256-bit IV.
    fn new() -> Blake2sState {
        Blake2sState { h: BoxImpl::new(BLAKE2S_256_INITIAL_STATE), byte_len: 0 }
    }

    /// Updates the hasher with 64 bytes of data.
    /// NOTE: u32 words are little-endian.
    fn compress_block(ref self: Blake2sState, data: [u32; 16]) {
        self.byte_len += 64;
        self.h = blake2s_compress(self.h, self.byte_len, BoxImpl::new(data));
    }

    /// Pads the data to 16 words and finalizes the hash.
    /// Data must contain no more than 16 words.
    /// NOTE: u32 words are little-endian (both for input and digest).
    fn finalize(ref self: Blake2sState, data: Span<u32>) -> Blake2sDigest {
        let mut buffer: Array<u32> = data.into();
        let byte_len = self.byte_len + buffer.len() * 4;
        // Pad the buffer to 16 words.
        for _ in buffer.len()..16 {
            buffer.append(0);
        }
        // If the passed data was larger than 16 words, we will fail here
        let block = buffer.span().try_into().expect('Cast to @Blake2sInput failed');
        self.h = blake2s_finalize(self.h, byte_len, *block);
        self.h
    }

    /// Finalizes without padding.
    fn finalize_block(ref self: Blake2sState, data: [u32; 16], byte_len: u32) -> Blake2sDigest {
        self.h = blake2s_finalize(self.h, self.byte_len + byte_len, BoxImpl::new(data));
        self.h
    }

    /// Consumes the hasher and returns the final state (digest).
    fn digest(self: Blake2sState) -> Blake2sDigest {
        self.h
    }
}

/// Computes the parent hash of two Blake2s digests.
pub fn blake2s_hash_pair(left: Blake2sDigest, right: Blake2sDigest) -> Blake2sDigest {
    let mut state = Blake2sHasher::new();
    let [a, b, c, d, e, f, g, h] = left.unbox();
    let [i, j, k, l, m, n, o, p] = right.unbox();
    state.finalize_block([a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p], 64)
}

/// `Into` implementation that converts a `Blake2sDigest` value into a `u256` integer.
/// NOTE: digest words remain in little-endian byte order.
pub impl Blake2sDigestIntoU256 of Into<Blake2sDigest, u256> {
    fn into(self: Blake2sDigest) -> u256 {
        u32x8_to_u256(self.unbox())
    }
}

/// `Into` implementation that converts a `u256` integer into a `Blake2sDigest`.
/// NOTE: digest words are expected to be in little-endian byte order.
pub impl Blake2sDigestFromU256 of Into<u256, Blake2sDigest> {
    fn into(self: u256) -> Blake2sDigest {
        BoxImpl::new(u256_to_u32x8(self))
    }
}

/// `PartialEq` implementation that compares two `Blake2sDigest` values.
pub impl Blake2sDigestPartialEq of PartialEq<Blake2sDigest> {
    fn eq(lhs: @Blake2sDigest, rhs: @Blake2sDigest) -> bool {
        lhs.unbox() == rhs.unbox()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2s_hasher_empty() {
        let mut hasher = Blake2sHasher::new();
        let digest = hasher.finalize(array![].span());
        let res: u256 = digest.into();
        assert_eq!(res, 0x307a216994809079d02111e17c4a354248b6551f1ea5a12cfd0d251bf9eed01e);
    }

    #[test]
    fn test_blake2s_hasher() {
        let mut hasher = Blake2sHasher::new();
        hasher.compress_block([1; 16]);
        let digest = hasher.finalize(array![2, 3, 4, 5, 6, 7].span());
        let res: u256 = digest.into();
        assert_eq!(res, 0xdc1f2e4b6ff53e2d0bbeb04629cb7012c61ff734360024087cd333cf67ac9e7d);
    }

    #[test]
    fn test_blake2s_hash_pair() {
        let left = 0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66_u256.into();
        let right = 0xc713e33d89122b85e2f646cc518c2e6ef88b06d3b016104faa95f84f878dab66_u256.into();
        let digest = blake2s_hash_pair(left, right);
        let res: u256 = digest.into();
        assert_eq!(res, 0x693aa1ab81c6362fe339fc4c7f6d8ddb1e515701e58c5bb2fb54a193c8287fdc);
    }
}
