//! Hex helpers.

use crate::hash::Digest;

/// Gets bytes from hex (base16).
pub fn from_hex(hex_string: ByteArray) -> ByteArray {
    let num_characters = hex_string.len();
    assert!(num_characters % 2 == 0, "Invalid hex string length");

    let mut bytes: ByteArray = Default::default();
    let mut i = 0;

    while i != num_characters {
        let hi = hex_char_to_nibble(hex_string[i]);
        let lo = hex_char_to_nibble(hex_string[i + 1]);
        bytes.append_byte(hi * 16 + lo);
        i += 2;
    }

    bytes
}

/// Converts bytes to hex (base16).
pub fn to_hex(data: @ByteArray) -> ByteArray {
    let alphabet: @ByteArray = @"0123456789abcdef";
    let mut result: ByteArray = Default::default();

    let mut i = 0;
    while i != data.len() {
        let value: u32 = data[i].into();
        let (l, r) = core::traits::DivRem::div_rem(value, 16);
        result.append_byte(alphabet.at(l).unwrap());
        result.append_byte(alphabet.at(r).unwrap());
        i += 1;
    }

    result
}

// Gets `Digest` from reversed `ByteArray`.
pub fn hex_to_hash_rev(hex_string: ByteArray) -> Digest {
    let mut result: Array<u32> = array![];
    let mut i = 0;
    let mut unit: u32 = 0;
    let len = hex_string.len();
    while i != len {
        if (i != 0 && i % 8 == 0) {
            result.append(unit);
            unit = 0;
        }
        let hi = hex_char_to_nibble(hex_string[len - i - 2]);
        let lo = hex_char_to_nibble(hex_string[len - i - 1]);
        unit = (unit * 256) + (hi * 16 + lo).into();
        i += 2;
    }
    result.append(unit);

    Digest {
        value: [
            *result[0], *result[1], *result[2], *result[3], *result[4], *result[5], *result[6],
            *result[7],
        ],
    }
}

pub fn hex_char_to_nibble(hex_char: u8) -> u8 {
    if hex_char >= 48 && hex_char <= 57 {
        // 0-9
        hex_char - 48
    } else if hex_char >= 65 && hex_char <= 70 {
        // A-F
        hex_char - 55
    } else if hex_char >= 97 && hex_char <= 102 {
        // a-f
        hex_char - 87
    } else {
        assert!(false, "Invalid hex character: {hex_char}");
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::hash::Digest;
    use super::{from_hex, hex_to_hash_rev, to_hex};

    #[test]
    fn test_bytes_from_hex() {
        assert_eq!(@"hello starknet", @from_hex("68656c6c6f20737461726b6e6574"));
        assert_eq!(@"hello starknet", @from_hex("68656C6C6F20737461726B6E6574"));
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(@"68656c6c6f20737461726b6e6574", @to_hex(@"hello starknet"));
    }

    #[test]
    fn test_hex_to_hash_rev() {
        let hash: Digest = Digest {
            value: [
                0x12345678_u32, 0x9abcdef0_u32, 0x11223344_u32, 0x55667788_u32, 0xaabbccdd_u32,
                0xeeff0011_u32, 0x22334455_u32, 0x66778899_u32,
            ],
        };
        let hex = "99887766554433221100ffeeddccbbaa8877665544332211f0debc9a78563412";
        assert_eq!(hash, hex_to_hash_rev(hex));
    }
}
