//! Numeric helpers.

use crate::bit_shifts::shr_u64;

const POW_2_32: u128 = 0x100000000;
const POW_2_64: u128 = 0x10000000000000000;
const POW_2_96: u128 = 0x1000000000000000000000000;
const NZ_POW2_32_128: NonZero<u128> = 0x100000000;
const NZ_POW2_32_64: NonZero<u64> = 0x100000000;

/// Computes the next power of two of a `u64` variable.
/// Returns 2^x, where x is the smallest integer such that 2^x >= n.
pub fn u64_next_power_of_two(mut n: u64) -> u64 {
    if n == 0 {
        return 1;
    }

    n -= 1;
    n = n | shr_u64(n, 1_u32);
    n = n | shr_u64(n, 2_u32);
    n = n | shr_u64(n, 4_u32);
    n = n | shr_u64(n, 8_u32);
    n = n | shr_u64(n, 16_u32);
    n = n | shr_u64(n, 32_u32);

    n + 1
}


/// Converts a `u256` value into a `[u32; 8]` array.
pub fn u256_to_u32x8(value: u256) -> [u32; 8] {
    let u256 { low, high } = value;

    let (abc, d) = DivRem::div_rem(high, NZ_POW2_32_128);
    let (ab, c) = DivRem::div_rem(abc, NZ_POW2_32_128);
    let ab: u64 = ab.try_into().unwrap();
    let (a, b) = DivRem::div_rem(ab, NZ_POW2_32_64);

    let (efg, h) = DivRem::div_rem(low, NZ_POW2_32_128);
    let (ef, g) = DivRem::div_rem(efg, NZ_POW2_32_128);
    let ef: u64 = ef.try_into().unwrap();
    let (e, f) = DivRem::div_rem(ef, NZ_POW2_32_64);

    [
        a.try_into().unwrap(), b.try_into().unwrap(), c.try_into().unwrap(), d.try_into().unwrap(),
        e.try_into().unwrap(), f.try_into().unwrap(), g.try_into().unwrap(), h.try_into().unwrap(),
    ]
}


/// Converts a `[u32; 8]` array into a `u256` value.
pub fn u32x8_to_u256(value: [u32; 8]) -> u256 {
    let [a, b, c, d, e, f, g, h] = value;

    let high: u128 = a.into() * POW_2_96 + b.into() * POW_2_64 + c.into() * POW_2_32 + d.into();
    let low: u128 = e.into() * POW_2_96 + f.into() * POW_2_64 + g.into() * POW_2_32 + h.into();

    u256 { low, high }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_next_power_of_two() {
        let input: u64 = 3;
        let expected_output: u64 = 4;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 5;
        let expected_output: u64 = 8;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 11;
        let expected_output: u64 = 16;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 20;
        let expected_output: u64 = 32;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 61;
        let expected_output: u64 = 64;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 100;
        let expected_output: u64 = 128;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 189;
        let expected_output: u64 = 256;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 480;
        let expected_output: u64 = 512;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 777;
        let expected_output: u64 = 1024;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 1025;
        let expected_output: u64 = 2048;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 4095;
        let expected_output: u64 = 4096;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);

        let input: u64 = 1500000000000000000;
        let expected_output: u64 = 2305843009213693952;
        let result = u64_next_power_of_two(input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_u256_to_u32x8() {
        let input: u256 = u256 {
            high: 0x000102030405060708090a0b0c0d0e0f, low: 0x00112233445566778899aabbccddeeff,
        };
        let expected_output: [u32; 8] = [
            0x00010203, 0x04050607, 0x08090a0b, 0x0c0d0e0f, 0x00112233, 0x44556677, 0x8899aabb,
            0xccddeeff,
        ];
        let result = u256_to_u32x8(input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_u32x8_to_u256() {
        let input: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let expected_output: u256 = u256 {
            high: 0x00000001000000020000000300000004, low: 0x00000005000000060000000700000008,
        };
        let result = u32x8_to_u256(input);
        assert_eq!(result, expected_output);
    }
}
