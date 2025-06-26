//! `Digest` struct and trait implementations.

use core::fmt::{Display, Error, Formatter};
use core::hash::{Hash, HashStateTrait};
use core::integer::u128_byte_reverse;
use core::num::traits::zero::Zero;
use core::to_byte_array::AppendFormattedToByteArray;
use crate::numeric::{u256_to_u32x8, u32x8_to_u256};

/// 256-bit hash digest.
/// Represented as an array of 4-byte words.
#[derive(Copy, Drop, Debug, Default, Serde)]
pub struct Digest {
    pub value: [u32; 8],
}

#[generate_trait]
pub impl DigestImpl of DigestTrait {
    #[inline(always)]
    fn new(array: [u32; 8]) -> Digest {
        Digest { value: array }
    }
}

/// `Zero` trait implementation for `Digest`.
impl DigestZero of Zero<Digest> {
    fn zero() -> Digest {
        Digest { value: [0_u32; 8] }
    }

    fn is_zero(self: @Digest) -> bool {
        self.value == @[0_u32; 8]
    }

    fn is_non_zero(self: @Digest) -> bool {
        !self.is_zero()
    }
}

/// `Display` trait implementation for `Digest`.
impl DigestDisplay of Display<Digest> {
    fn fmt(self: @Digest, ref f: Formatter) -> Result<(), Error> {
        let hash: u256 = (*self).into();
        hash.append_formatted_to_byte_array(ref f.buffer, 16);
        Result::Ok(())
    }
}

/// `PartialEq` trait implementation for `Digest`.
impl DigestPartialEq of PartialEq<Digest> {
    fn eq(lhs: @Digest, rhs: @Digest) -> bool {
        lhs.value == rhs.value
    }
}

/// `Into` implementation that converts a `Digest` value into a `ByteArray`.
pub impl DigestIntoByteArray of Into<Digest, ByteArray> {
    fn into(self: Digest) -> ByteArray {
        let mut bytes: ByteArray = Default::default();
        for word in self.value.span() {
            bytes.append_word((*word).into(), 4);
        }
        bytes
    }
}

/// Converts a `u256` value into a `Digest` type and reverses bytes order.
/// `u256` is big-endian like in explorer, while `Digest` is little-endian order.
pub impl U256IntoDigest of Into<u256, Digest> {
    fn into(self: u256) -> Digest {
        Digest {
            value: u256_to_u32x8(
                u256 { low: u128_byte_reverse(self.high), high: u128_byte_reverse(self.low) },
            ),
        }
    }
}

/// `Into` implementation that converts a `Digest` value into a `u256` type and reverses bytes
/// order.
/// `Digest` is little-endian order, while `u256` is big-endian like in explorer.
pub impl DigestIntoU256 of Into<Digest, u256> {
    fn into(self: Digest) -> u256 {
        let u256 { low, high } = u32x8_to_u256(self.value);
        u256 { low: u128_byte_reverse(high), high: u128_byte_reverse(low) }
    }
}

/// `Hash` trait implementation for `Digest`.
pub impl DigestHash<S, +HashStateTrait<S>, +Drop<S>> of Hash<Digest, S> {
    fn update_state(state: S, value: Digest) -> S {
        let u256_digest: u256 = value.into();

        let state = state.update(u256_digest.high.into());
        let state = state.update(u256_digest.low.into());
        state
    }
}

#[cfg(test)]
mod tests {
    use crate::hex::from_hex;
    use super::Digest;

    #[test]
    fn test_u256_into_hash() {
        let u256_value = u256 {
            low: 0x1234567890abcdef1234567890abcdef_u128,
            high: 0xfedcba0987654321fedcba0987654321_u128,
        };

        let result_hash = u256_value.into();

        let expected_hash = Digest {
            value: [
                0xefcdab90, 0x78563412, 0xefcdab90, 0x78563412, 0x21436587, 0x09badcfe, 0x21436587,
                0x09badcfe,
            ],
        };

        assert_eq!(result_hash, expected_hash, "invalid results");
    }

    #[test]
    fn test_hash_to_u256() {
        let hash_value = Digest {
            value: [
                0xfedcba09, 0x87654321, 0xfedcba09, 0x87654321, 0x12345678, 0x90abcdef, 0x12345678,
                0x90abcdef,
            ],
        };

        let result_u256 = hash_value.into();

        let expected_u256 = u256 {
            high: 0xefcdab9078563412efcdab9078563412_u128,
            low: 0x2143658709badcfe2143658709badcfe_u128,
        };

        assert_eq!(result_u256, expected_u256, "invalid results");
    }

    #[test]
    fn test_hash_to_u256_to_hash() {
        let hash_value = Digest {
            value: [
                0xfedcba09, 0x87654321, 0xfedcba09, 0x87654321, 0x12345678, 0x90abcdef, 0x12345678,
                0x90abcdef,
            ],
        };

        let u256_value: u256 = hash_value.into();
        let result_hash: Digest = u256_value.into();

        assert_eq!(result_hash, hash_value, "invalid results");
    }

    #[test]
    fn test_u256_to_hash_to_u256() {
        let u256_value = u256 {
            high: 0xefcdab9078563412efcdab9078563412_u128,
            low: 0x00112233445566778899aabbccddeeff_u128,
        };

        let hash_value: Digest = u256_value.into();
        let result_u256: u256 = hash_value.into();

        assert_eq!(result_u256, u256_value, "invalid results");
    }

    #[test]
    fn test_hash_into_bytearray() {
        let hash = Digest {
            value: [
                0x12345678_u32, 0x9abcdef0_u32, 0x11223344_u32, 0x55667788_u32, 0xaabbccdd_u32,
                0xeeff0011_u32, 0x22334455_u32, 0x66778899_u32,
            ],
        };

        let byte_array: ByteArray = hash.into();

        let expected_byte_array = from_hex(
            "123456789abcdef01122334455667788aabbccddeeff00112233445566778899",
        );

        assert_eq!(byte_array, expected_byte_array, "invalid results");
    }
}
