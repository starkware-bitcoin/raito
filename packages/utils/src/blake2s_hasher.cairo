use core::blake::{blake2s_compress, blake2s_finalize};
use core::box::BoxImpl;
use utils::numeric::u32x8_to_u256;

/// BLAKE2s IV is the same as SHA-256 IV
/// We modify the first word to pre-configure:
/// IV[0] ^ 0x01010020 (config: no key, 32 bytes output).
const BLAKE2S_256_INITIAL_STATE: [u32; 8] = [
    0x6B08E647, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

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
    fn finalize_block(ref self: Blake2sState, data: Span<u32>) -> Box<[u32; 8]> {
        let mut buffer: Array<u32> = data.into();
        let byte_len = self.byte_len + buffer.len() * 4;
        // Pad the buffer to 16 words.
        for _ in buffer.len()..16 {
            buffer.append(0);
        }
        // If the passed data was larger than 16 words, we will fail here
        let block = buffer.span().try_into().expect('Cast to @Blake2sInput failed');
        blake2s_finalize(self.h, byte_len, *block)
    }
}

/// Converts a Blake2s digest to a u256.
/// Takes the words in the reversed order.
/// If you use Rust/Python implementation, reinterpet reference digest as LE integer to compare.
pub fn blake2s_digest_to_u256_le(digest: Box<[u32; 8]>) -> u256 {
    let [a, b, c, d, e, f, g, h] = digest.unbox();
    u32x8_to_u256([h, g, f, e, d, c, b, a])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2s_hasher_empty() {
        let mut hasher = Blake2sHasher::new();
        let digest = hasher.finalize_block(array![].span());
        let res = blake2s_digest_to_u256_le(digest);
        assert_eq!(
            res, 113047845297338535936082629575907534931834714830939814641088356107939732922729,
        );
    }

    #[test]
    fn test_blake2s_hasher() {
        let mut hasher = Blake2sHasher::new();
        hasher.compress_block([1; 16]);
        let digest = hasher.finalize_block(array![2, 3, 4, 5, 6, 7].span());
        let res = blake2s_digest_to_u256_le(digest);
        assert_eq!(
            res, 46893214958280054196873295526662987612846164852081208338716593733223283895883,
        );
    }
}
