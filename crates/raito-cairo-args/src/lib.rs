//! Serialize Rust structures into Cairo/Scarb runner-compatible argument felts
//!
//! This library provides a custom Serde serializer that converts Rust data structures
//! into the specific format expected by Cairo programs and Scarb runner arguments.

pub mod error;
pub mod serializer;

#[cfg(feature = "adapters")]
pub mod adapters;

// Re-export main public API
pub use error::CairoSerializeError;
pub use serializer::{to_felts, to_runner_args_hex};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_felt_and_list() {
        #[derive(Serialize)]
        struct S {
            a: u32,
            b: Vec<u32>,
        }
        let s = S {
            a: 3,
            b: vec![10, 20],
        };
        let args = to_runner_args_hex(&s).unwrap();
        // struct flattens: a, list len, items...
        assert_eq!(args, vec!["0x3", "0x2", "0xa", "0x14"]);
    }

    #[test]
    fn test_u256_decimal_string() {
        #[derive(Serialize)]
        struct S {
            x: String,
        }
        let s = S {
            x: "340282366920938463463374607431768211455".to_string(),
        }; // 2^128-1 hi=0, lo=2^128-1 fits into u256
        let felts = to_felts(&s).unwrap();
        // list: [lo, hi] with lo first
        assert_eq!(felts.len(), 2);
        assert_eq!(felts[0], num_bigint::BigUint::from_bytes_be(&[0xff; 16]));
        assert_eq!(felts[1], num_bigint::BigUint::from(0u32));
    }

    #[test]
    fn test_digest_64hex() {
        #[derive(Serialize)]
        struct S {
            h: String,
        }
        let s = S {
            h: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f".to_string(),
        };
        let felts = to_felts(&s).unwrap();
        assert_eq!(felts.len(), 8);
        // Check first and last words
        assert_eq!(
            felts[0],
            num_bigint::BigUint::from(0x6f as u32 | (0xe2 << 8) | (0x8c << 16) | (0x0a << 24))
        ); // little check through reverse+be
        assert!(felts.iter().all(|w| w.bits() <= 32));
    }

    #[test]
    fn test_bytearray_0xhex() {
        #[derive(Serialize)]
        struct S {
            data: String,
        }
        // 35 bytes -> 1 chunk (31) + remainder (4)
        let s = S {
            data: "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20212223"
                .to_string(),
        };
        let felts = to_felts(&s).unwrap();
        // [num_chunks=1, chunk0, rem_word, rem_len=4]
        assert_eq!(felts.len(), 4);
        assert_eq!(felts[0], num_bigint::BigUint::from(1u8));
        assert_eq!(felts[3], num_bigint::BigUint::from(4u8));
    }
}
