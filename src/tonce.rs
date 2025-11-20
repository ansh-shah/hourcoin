/// Tonce (Time-Only-Used-Once) module for Hourcoin
///
/// The tonce system implements a time-based mining challenge that:
/// 1. Takes the timestamp of the previous accepted block
/// 2. Hashes it and extracts the least significant 5 bits
/// 3. For the first 60 seconds, only accepts blocks whose timestamp hash is divisible by this tonce
/// 4. After 60 seconds, reduces tonce to 1 (accepts any block - becomes a race)
///
/// This creates a randomized difficulty for miners during each hour-long mining round.

use crate::u128_bytes;

const TONCE_CHALLENGE_DURATION_MS: u128 = 60_000; // 60 seconds in milliseconds

/// Represents a tonce challenge for a mining round
#[derive(Debug, Clone)]
pub struct TonceChallenge {
    /// The previous block's acceptance timestamp
    pub prev_block_timestamp: u128,
    /// The tonce divisor (1-31, derived from 5 bits)
    pub tonce: u8,
    /// Whether the challenge period has expired
    pub challenge_expired: bool,
}

impl TonceChallenge {
    /// Create a new tonce challenge based on the previous block's timestamp
    pub fn new(prev_block_timestamp: u128) -> Self {
        let tonce = Self::calculate_tonce(prev_block_timestamp);
        TonceChallenge {
            prev_block_timestamp,
            tonce,
            challenge_expired: false,
        }
    }

    /// Calculate the tonce value from a timestamp
    ///
    /// Hashes the timestamp and extracts the least significant 5 bits (1-31)
    /// A tonce of 0 would mean everything is divisible, so we ensure it's at least 1
    fn calculate_tonce(timestamp: u128) -> u8 {
        let timestamp_bytes = u128_bytes(&timestamp);
        let hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &timestamp_bytes);

        // Get the last byte and extract the least significant 5 bits
        let last_byte = hash[31];
        let tonce = last_byte & 0b00011111; // Mask to get last 5 bits

        // Ensure tonce is at least 1 (0 would make everything pass)
        if tonce == 0 {
            1
        } else {
            tonce
        }
    }

    /// Check if a timestamp passes the tonce challenge
    ///
    /// For the first 60 seconds after the previous block:
    /// - Calculate the hash of the proposed timestamp
    /// - Check if it's divisible by the tonce
    ///
    /// After 60 seconds:
    /// - Any timestamp passes (race to submit)
    pub fn validate_timestamp(&mut self, timestamp: u128, current_time: u128) -> bool {
        // Check if challenge period has expired
        let time_since_prev_block = current_time.saturating_sub(self.prev_block_timestamp);

        if time_since_prev_block >= TONCE_CHALLENGE_DURATION_MS {
            self.challenge_expired = true;
            self.tonce = 1; // Reduce to 1 - race condition
            return true; // Accept any timestamp after challenge period
        }

        // Within challenge period - check divisibility
        self.is_timestamp_divisible(timestamp)
    }

    /// Check if a timestamp hash is divisible by the tonce
    fn is_timestamp_divisible(&self, timestamp: u128) -> bool {
        let timestamp_bytes = u128_bytes(&timestamp);
        let hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &timestamp_bytes);

        // Convert last 4 bytes to u32 for divisibility check
        let hash_value = u32::from_be_bytes([hash[28], hash[29], hash[30], hash[31]]);

        hash_value % (self.tonce as u32) == 0
    }

    /// Get the time remaining in the challenge period (in seconds)
    pub fn seconds_remaining(&self, current_time: u128) -> u64 {
        let time_since_prev_block = current_time.saturating_sub(self.prev_block_timestamp);

        if time_since_prev_block >= TONCE_CHALLENGE_DURATION_MS {
            0
        } else {
            ((TONCE_CHALLENGE_DURATION_MS - time_since_prev_block) / 1000) as u64
        }
    }

    /// Get the tonce value for this challenge
    pub fn get_tonce(&self) -> u8 {
        self.tonce
    }

    /// Check if the challenge period has expired
    pub fn is_expired(&self, current_time: u128) -> bool {
        let time_since_prev_block = current_time.saturating_sub(self.prev_block_timestamp);
        time_since_prev_block >= TONCE_CHALLENGE_DURATION_MS
    }
}

/// Helper to find a timestamp that satisfies the tonce challenge
///
/// Used by miners to find valid timestamps for block submission
pub fn find_valid_timestamp(tonce: u8, start_time: u128, max_attempts: u32) -> Option<u128> {
    if tonce == 0 || tonce == 1 {
        return Some(start_time); // Any timestamp works
    }

    for i in 0..max_attempts {
        let candidate_timestamp = start_time + i as u128;
        let timestamp_bytes = u128_bytes(&candidate_timestamp);
        let hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &timestamp_bytes);
        let hash_value = u32::from_be_bytes([hash[28], hash[29], hash[30], hash[31]]);

        if hash_value % (tonce as u32) == 0 {
            return Some(candidate_timestamp);
        }
    }

    None // Failed to find valid timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tonce_creation() {
        let timestamp = 1000000;
        let challenge = TonceChallenge::new(timestamp);

        assert_eq!(challenge.prev_block_timestamp, timestamp);
        assert!(challenge.tonce >= 1 && challenge.tonce <= 31);
        assert!(!challenge.challenge_expired);
    }

    #[test]
    fn test_calculate_tonce_range() {
        // Test multiple timestamps to ensure tonce is always in valid range
        for i in 0..100 {
            let timestamp = 1000000 + i * 1000;
            let challenge = TonceChallenge::new(timestamp);
            assert!(challenge.tonce >= 1 && challenge.tonce <= 31);
        }
    }

    #[test]
    fn test_challenge_expiration() {
        let prev_timestamp = 1000000;
        let mut challenge = TonceChallenge::new(prev_timestamp);

        // Within challenge period (30 seconds later)
        let current_time = prev_timestamp + 30_000;
        assert!(!challenge.is_expired(current_time));

        // After challenge period (61 seconds later)
        let current_time_after = prev_timestamp + 61_000;
        assert!(challenge.is_expired(current_time_after));
    }

    #[test]
    fn test_validation_after_expiration() {
        let prev_timestamp = 1000000;
        let mut challenge = TonceChallenge::new(prev_timestamp);

        // After 60 seconds, any timestamp should pass
        let current_time = prev_timestamp + 61_000;
        let any_timestamp = 1234567;

        assert!(challenge.validate_timestamp(any_timestamp, current_time));
        assert!(challenge.challenge_expired);
        assert_eq!(challenge.tonce, 1);
    }

    #[test]
    fn test_seconds_remaining() {
        let prev_timestamp = 1000000;
        let challenge = TonceChallenge::new(prev_timestamp);

        // 30 seconds after previous block
        let current_time = prev_timestamp + 30_000;
        let remaining = challenge.seconds_remaining(current_time);
        assert_eq!(remaining, 30);

        // After expiration
        let current_time_after = prev_timestamp + 70_000;
        let remaining_after = challenge.seconds_remaining(current_time_after);
        assert_eq!(remaining_after, 0);
    }

    #[test]
    fn test_find_valid_timestamp() {
        // Test with tonce = 1 (should return immediately)
        let result = find_valid_timestamp(1, 1000000, 100);
        assert!(result.is_some());

        // Test with higher tonce (should find something within reasonable attempts)
        let result = find_valid_timestamp(5, 1000000, 1000);
        assert!(result.is_some());

        // If found, verify it actually passes divisibility
        if let Some(ts) = result {
            let timestamp_bytes = u128_bytes(&ts);
            let hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &timestamp_bytes);
            let hash_value = u32::from_be_bytes([hash[28], hash[29], hash[30], hash[31]]);
            assert_eq!(hash_value % 5, 0);
        }
    }

    #[test]
    fn test_is_timestamp_divisible() {
        let prev_timestamp = 1000000;
        let challenge = TonceChallenge::new(prev_timestamp);

        // Find a valid timestamp for this challenge
        if let Some(valid_ts) = find_valid_timestamp(challenge.tonce, 1000000, 10000) {
            assert!(challenge.is_timestamp_divisible(valid_ts));
        }
    }

    #[test]
    fn test_different_timestamps_different_tonces() {
        let challenge1 = TonceChallenge::new(1000000);
        let challenge2 = TonceChallenge::new(2000000);

        // Different timestamps should (usually) produce different tonces
        // Note: There's a small chance they could be the same, so we test multiple
        let mut different_found = false;
        for i in 0..10 {
            let c1 = TonceChallenge::new(1000000 + i * 1000);
            let c2 = TonceChallenge::new(2000000 + i * 1000);
            if c1.tonce != c2.tonce {
                different_found = true;
                break;
            }
        }
        assert!(different_found);
    }
}
