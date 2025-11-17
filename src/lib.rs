type BlockHash = Vec<u8>;
type Address = String;

use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u128 {
	let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
	duration.as_secs() as u128 * 1000 + duration.subsec_millis() as u128
}

pub fn u32_bytes (u: &u32) -> [u8; 4] {
    [
        (u >> 8 * 0x0) as u8,
        (u >> 8 * 0x1) as u8,
        (u >> 8 * 0x2) as u8,
        (u >> 8 * 0x3) as u8,
    ]
}

pub fn u64_bytes (u: &u64) -> [u8; 8] {
    [
        (u >> 8 * 0x0) as u8,
        (u >> 8 * 0x1) as u8,
        (u >> 8 * 0x2) as u8,
        (u >> 8 * 0x3) as u8,

        (u >> 8 * 0x4) as u8,
        (u >> 8 * 0x5) as u8,
        (u >> 8 * 0x6) as u8,
        (u >> 8 * 0x7) as u8,
    ]
}

pub fn u128_bytes (u: &u128) -> [u8; 16] {
    [
        (u >> 8 * 0x0) as u8,
        (u >> 8 * 0x1) as u8,
        (u >> 8 * 0x2) as u8,
        (u >> 8 * 0x3) as u8,

        (u >> 8 * 0x4) as u8,
        (u >> 8 * 0x5) as u8,
        (u >> 8 * 0x6) as u8,
        (u >> 8 * 0x7) as u8,

        (u >> 8 * 0x8) as u8,
        (u >> 8 * 0x9) as u8,
        (u >> 8 * 0xa) as u8,
        (u >> 8 * 0xb) as u8,

        (u >> 8 * 0xc) as u8,
        (u >> 8 * 0xd) as u8,
        (u >> 8 * 0xe) as u8,
        (u >> 8 * 0xf) as u8,
    ]
}

pub fn difficulty_bytes_as_u128 (v: &Vec<u8>) -> u128 {
    ((v[31] as u128) << 0xf * 8) |
    ((v[30] as u128) << 0xe * 8) |
    ((v[29] as u128) << 0xd * 8) |
    ((v[28] as u128) << 0xc * 8) |
    ((v[27] as u128) << 0xb * 8) |
    ((v[26] as u128) << 0xa * 8) |
    ((v[25] as u128) << 0x9 * 8) |
    ((v[24] as u128) << 0x8 * 8) |
    ((v[23] as u128) << 0x7 * 8) |
    ((v[22] as u128) << 0x6 * 8) |
    ((v[21] as u128) << 0x5 * 8) |
    ((v[20] as u128) << 0x4 * 8) |
    ((v[19] as u128) << 0x3 * 8) |
    ((v[18] as u128) << 0x2 * 8) |
    ((v[17] as u128) << 0x1 * 8) |
    ((v[16] as u128) << 0x0 * 8)
}

mod block;
pub use crate::block::Block;
mod hashable;
pub use crate::hashable::Hashable;
mod blockchain;
pub use crate::blockchain::Blockchain;
pub mod transaction;
pub use crate::transaction::Transaction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u32_bytes_conversion() {
        let val: u32 = 0x12345678;
        let bytes = u32_bytes(&val);
        assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_u32_bytes_zero() {
        let val: u32 = 0;
        let bytes = u32_bytes(&val);
        assert_eq!(bytes, [0, 0, 0, 0]);
    }

    #[test]
    fn test_u32_bytes_max() {
        let val: u32 = u32::MAX;
        let bytes = u32_bytes(&val);
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_u64_bytes_conversion() {
        let val: u64 = 0x123456789ABCDEF0;
        let bytes = u64_bytes(&val);
        assert_eq!(bytes, [0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_u64_bytes_zero() {
        let val: u64 = 0;
        let bytes = u64_bytes(&val);
        assert_eq!(bytes, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_u64_bytes_max() {
        let val: u64 = u64::MAX;
        let bytes = u64_bytes(&val);
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn test_u128_bytes_conversion() {
        let val: u128 = 0x0F0E0D0C0B0A09080706050403020100;
        let bytes = u128_bytes(&val);
        assert_eq!(bytes, [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F
        ]);
    }

    #[test]
    fn test_u128_bytes_zero() {
        let val: u128 = 0;
        let bytes = u128_bytes(&val);
        assert_eq!(bytes, [0; 16]);
    }

    #[test]
    fn test_u128_bytes_max() {
        let val: u128 = u128::MAX;
        let bytes = u128_bytes(&val);
        assert_eq!(bytes, [0xFF; 16]);
    }

    #[test]
    fn test_difficulty_bytes_as_u128() {
        // Create a 32-byte vector with known pattern
        let mut v = vec![0u8; 32];
        // Set bytes 16-31 to a known pattern
        v[16] = 0x00;
        v[17] = 0x01;
        v[18] = 0x02;
        v[19] = 0x03;
        v[20] = 0x04;
        v[21] = 0x05;
        v[22] = 0x06;
        v[23] = 0x07;
        v[24] = 0x08;
        v[25] = 0x09;
        v[26] = 0x0A;
        v[27] = 0x0B;
        v[28] = 0x0C;
        v[29] = 0x0D;
        v[30] = 0x0E;
        v[31] = 0x0F;

        let result = difficulty_bytes_as_u128(&v);
        let expected: u128 = 0x0F0E0D0C0B0A09080706050403020100;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_difficulty_bytes_all_zeros() {
        let v = vec![0u8; 32];
        let result = difficulty_bytes_as_u128(&v);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_difficulty_bytes_all_ones() {
        let v = vec![0xFFu8; 32];
        let result = difficulty_bytes_as_u128(&v);
        assert_eq!(result, u128::MAX);
    }

    #[test]
    fn test_now_returns_reasonable_timestamp() {
        let timestamp = now();
        // Timestamp should be greater than Jan 1, 2020 (1577836800000 ms)
        assert!(timestamp > 1577836800000);
        // Timestamp should be less than Jan 1, 2100 (4102444800000 ms)
        assert!(timestamp < 4102444800000);
    }

    #[test]
    fn test_now_increments() {
        let t1 = now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = now();
        assert!(t2 > t1);
    }

    #[test]
    fn test_byte_conversion_round_trip_u32() {
        let original: u32 = 0xDEADBEEF;
        let bytes = u32_bytes(&original);
        // Reconstruct from bytes (little endian)
        let reconstructed = (bytes[0] as u32)
            | ((bytes[1] as u32) << 8)
            | ((bytes[2] as u32) << 16)
            | ((bytes[3] as u32) << 24);
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_byte_conversion_round_trip_u64() {
        let original: u64 = 0xDEADBEEFCAFEBABE;
        let bytes = u64_bytes(&original);
        // Reconstruct from bytes (little endian)
        let reconstructed = (bytes[0] as u64)
            | ((bytes[1] as u64) << 8)
            | ((bytes[2] as u64) << 16)
            | ((bytes[3] as u64) << 24)
            | ((bytes[4] as u64) << 32)
            | ((bytes[5] as u64) << 40)
            | ((bytes[6] as u64) << 48)
            | ((bytes[7] as u64) << 56);
        assert_eq!(original, reconstructed);
    }
}