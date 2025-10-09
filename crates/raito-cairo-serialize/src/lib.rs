//! Serialize Rust structures into Cairo/Scarb runner-compatible argument felts
//!
//! This library provides a custom encoder that converts Rust data structures
//! into the specific format expected by Cairo programs and Scarb runner arguments.

#[cfg(test)]
use anyhow::Result;
use num_bigint::BigUint;
use num_traits::Num;
use starknet_ff::FieldElement;
use stwo_cairo_serialize::serialize::CairoSerialize;

// Wrapper types for specialized Cairo serialization of string data
pub struct U256String(pub String);
pub struct ByteArrayString(pub String);
pub struct DigestString(pub String);
pub struct U256StringLittleEndian(pub String);

impl CairoSerialize for U256String {
    fn serialize(&self, output: &mut Vec<FieldElement>) {
        // Accept decimal string only, produce 32-byte big-endian
        let s = self.0.trim();
        assert!(
            !s.starts_with("0x") && !s.starts_with("0X"),
            "Hex not supported for U256String; use decimal",
        );
        let n = BigUint::from_str_radix(s, 10).expect("Invalid decimal string for U256");
        let bytes = n.to_bytes_be();
        assert!(bytes.len() <= 32, "U256 value exceeds 256 bits");
        let mut be = [0u8; 32];
        be[32 - bytes.len()..].copy_from_slice(&bytes);

        // lo = least-significant 16 bytes, hi = most-significant 16 bytes
        let (hi16, lo16) = be.split_at(16);

        let mut lo_bytes = [0u8; 32];
        lo_bytes[16..].copy_from_slice(lo16);
        let mut hi_bytes = [0u8; 32];
        hi_bytes[16..].copy_from_slice(hi16);

        output.push(FieldElement::from_bytes_be(&lo_bytes).unwrap());
        output.push(FieldElement::from_bytes_be(&hi_bytes).unwrap());
    }
}

impl CairoSerialize for U256StringLittleEndian {
    fn serialize(&self, output: &mut Vec<FieldElement>) {
        // Accept decimal string only, produce 32-byte big-endian
        let s = self.0.trim();
        assert!(
            !s.starts_with("0x") && !s.starts_with("0X"),
            "Hex not supported for U256StringHiLo; use decimal",
        );
        let n = BigUint::from_str_radix(s, 10).expect("Invalid decimal string for U256");
        let bytes = n.to_bytes_be();
        assert!(bytes.len() <= 32, "U256 value exceeds 256 bits");
        let mut be = [0u8; 32];
        be[32 - bytes.len()..].copy_from_slice(&bytes);

        // hi = most-significant 16 bytes, lo = least-significant 16 bytes
        let (lo16, hi16) = be.split_at(16);

        let mut hi_bytes = [0u8; 32];
        hi_bytes[16..].copy_from_slice(hi16);
        let mut lo_bytes = [0u8; 32];
        lo_bytes[16..].copy_from_slice(lo16);

        // Note: emit HI first, then LO to match Cairo MMR Serde (high, low)
        output.push(FieldElement::from_bytes_be(&lo_bytes).unwrap());
        output.push(FieldElement::from_bytes_be(&hi_bytes).unwrap());
    }
}

impl CairoSerialize for ByteArrayString {
    // Split into 31-byte chunks and save the remainder
    fn serialize(&self, output: &mut Vec<FieldElement>) {
        let s = self.0.as_str();
        let hex_str = if s.starts_with("0x") || s.starts_with("0X") {
            s.to_string()
        } else {
            format!("0x{}", hex::encode(s.as_bytes()))
        };

        // Remove 0x prefix
        let hex_data = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
        let bytes = hex::decode(hex_data).expect("Invalid hex string");

        // Calculate chunks and remainder
        let chunk_size = 31; // 31 bytes per chunk (248 bits, fits in felt252)
        let num_chunks = bytes.len() / chunk_size;
        let remainder_len = bytes.len() % chunk_size;

        // Serialize: num_chunks, chunks..., remainder, rem_len
        output.push(FieldElement::from(num_chunks as u128));

        // Serialize chunks
        for chunk in bytes.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let mut chunk_bytes = [0u8; 32];
                chunk_bytes[1..=chunk_size].copy_from_slice(chunk);
                output.push(FieldElement::from_bytes_be(&chunk_bytes).unwrap());
            }
        }

        // Serialize remainder
        if remainder_len > 0 {
            let remainder = &bytes[bytes.len() - remainder_len..];
            let mut rem_bytes = [0u8; 32];
            let start = 32 - remainder_len;
            rem_bytes[start..].copy_from_slice(remainder);
            output.push(FieldElement::from_bytes_be(&rem_bytes).unwrap());
        } else {
            output.push(FieldElement::from(0u8));
        }

        output.push(FieldElement::from(remainder_len as u128));
    }
}

impl CairoSerialize for DigestString {
    // Reversed hex string into 4-byte words then into BE u32
    fn serialize(&self, output: &mut Vec<FieldElement>) {
        let s = self.0.as_str();
        let hex_str = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);

        // Convert 64-char hex to 8 u32 words (reversed for little-endian)
        let bytes = hex::decode(hex_str).expect("Invalid hex string");
        assert!(bytes.len() == 32, "expected 32-byte digest");
        let mut rev = bytes;
        rev.reverse();
        for chunk in rev.chunks(4) {
            let mut word_bytes = [0u8; 4];
            word_bytes[..chunk.len()].copy_from_slice(chunk);
            let word = u32::from_be_bytes(word_bytes) as u128;
            output.push(FieldElement::from(word));
        }
    }
}

// Backwards-compatibility: preserve `serializer::...` path
pub mod serializer {
    pub use super::{ByteArrayString, DigestString, U256String, U256StringLittleEndian};
}

#[cfg(test)]
mod tests {
    use super::*;
    use starknet_ff::FieldElement;
    use stwo_cairo_serialize::CairoSerialize;

    fn to_hex<T: CairoSerialize + ?Sized>(value: &T) -> Result<Vec<String>> {
        let mut felts = Vec::new();
        value.serialize(&mut felts);
        Ok(felts.into_iter().map(|felt| fe_to_min_hex(&felt)).collect())
    }

    fn fe_to_min_hex(fe: &FieldElement) -> String {
        let bytes = fe.to_bytes_be();
        let mut i = 0;
        while i < bytes.len() && bytes[i] == 0 {
            i += 1;
        }
        if i == bytes.len() {
            return "0x0".to_string();
        }
        let mut s = String::from("0x");
        s.push_str(&format!("{:x}", bytes[i]));
        for b in &bytes[i + 1..] {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }

    // Homogeneous wrapper for heterogeneous types implementing CairoSerialize
    enum Kind {
        U256,
        Digest,
        ByteArray,
    }

    #[test]
    fn test_all_cases() -> Result<()> {
        let all_cases: &[(Kind, &str, &[&str])] = &[
            (
                Kind::U256,
                "340282366920938463463374607431768211455",
                &["0xffffffffffffffffffffffffffffffff", "0x0"] as &[&str],
            ),
            (
                Kind::U256,
                "23232323340282366920938463463374607431768211455",
                &["0x1b81a14a66f78cd9da6f237fffffffff", "0x411c5fe"] as &[&str],
            ),
            (
                Kind::Digest,
                "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
                &[
                    "0x6fe28c0a",
                    "0xb6f1b372",
                    "0xc1a6a246",
                    "0xae63f74f",
                    "0x931e8365",
                    "0xe15a089c",
                    "0x68d61900",
                    "0x0",
                ] as &[&str],
            ),
            (
                Kind::Digest,
                "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098",
                &[
                    "0x982051fd",
                    "0x1e4ba744",
                    "0xbbbe680e",
                    "0x1fee1467",
                    "0x7ba1a3c3",
                    "0x540bf7b1",
                    "0xcdb606e8",
                    "0x57233e0e",
                ] as &[&str],
            ),
            (
                Kind::ByteArray,
                "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20212223",
                &[
                    "0x1",
                    "0x102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
                    "0x20212223",
                    "0x4",
                ] as &[&str],
            ),
        ];

        for (kind, input, expected) in all_cases {
            let actual_strings = match kind {
                Kind::U256 => to_hex(&U256String((*input).to_string()))?,
                Kind::Digest => to_hex(&DigestString((*input).to_string()))?,
                Kind::ByteArray => to_hex(&ByteArrayString((*input).to_string()))?,
            };
            let actual: Vec<&str> = actual_strings.iter().map(|s| s.as_str()).collect();
            assert_eq!(actual.as_slice(), *expected);
        }

        Ok(())
    }
}
