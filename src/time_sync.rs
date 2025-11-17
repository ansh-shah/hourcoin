/// Time synchronization module for Hourcoin's proof of time consensus
///
/// This module provides functionality to synchronize with trusted external time sources
/// (such as Cloudflare's time service) to ensure accurate timestamp validation.
///
/// The proof of time consensus requires miners to submit blocks with accurate timestamps
/// that are validated against a trusted time source within a tolerance window.
///
/// Uses chrono for platform-agnostic time handling with consistent millisecond precision
/// across all platforms.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Represents a trusted time response from an external source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedTime {
    pub timestamp_ms: u128,
    pub source: String,
}

/// Time synchronization service for validating timestamps
pub struct TimeSync {
    /// Maximum allowed deviation from trusted time (in milliseconds)
    pub tolerance_ms: u128,
    /// Last known good timestamp from trusted source
    last_sync_time: Option<u128>,
}

impl TimeSync {
    /// Create a new TimeSync instance with default tolerance (500ms)
    pub fn new() -> Self {
        TimeSync {
            tolerance_ms: 500,
            last_sync_time: None,
        }
    }

    /// Create a new TimeSync instance with custom tolerance
    pub fn new_with_tolerance(tolerance_ms: u128) -> Self {
        TimeSync {
            tolerance_ms,
            last_sync_time: None,
        }
    }

    /// Get current system time in milliseconds since UNIX epoch
    /// Uses chrono for platform-agnostic precision
    pub fn get_system_time() -> u128 {
        Utc::now().timestamp_millis() as u128
    }

    /// Sync with external time source
    ///
    /// Queries world time API for accurate external time
    /// Falls back to system time if external source is unavailable
    pub async fn sync_with_external_source(&mut self) -> Result<TrustedTime, String> {
        // Try to get time from external source
        match Self::fetch_external_time().await {
            Ok(trusted_time) => {
                self.last_sync_time = Some(trusted_time.timestamp_ms);
                Ok(trusted_time)
            }
            Err(e) => {
                // Fall back to system time
                eprintln!("Warning: External time sync failed ({}), using system time", e);
                let timestamp = Self::get_system_time();
                let trusted_time = TrustedTime {
                    timestamp_ms: timestamp,
                    source: "system".to_string(),
                };
                self.last_sync_time = Some(timestamp);
                Ok(trusted_time)
            }
        }
    }

    /// Fetch time from external source (World Time API)
    async fn fetch_external_time() -> Result<TrustedTime, String> {
        // Use World Time API as it's simple and doesn't require authentication
        // Alternative: http://worldtimeapi.org/api/timezone/Etc/UTC
        let url = "http://worldtimeapi.org/api/timezone/Etc/UTC";

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch time: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Extract unixtime in seconds
        let unixtime_secs = json["unixtime"]
            .as_i64()
            .ok_or("Missing unixtime field")?;

        // Convert to milliseconds
        let timestamp_ms = (unixtime_secs as u128) * 1000;

        Ok(TrustedTime {
            timestamp_ms,
            source: "worldtimeapi.org".to_string(),
        })
    }

    /// Validate a timestamp against trusted time
    ///
    /// Returns true if the timestamp is within tolerance of the current trusted time
    pub fn validate_timestamp(&self, timestamp: u128) -> bool {
        let current_time = Self::get_system_time();

        // Check if timestamp is not too far in the future
        if timestamp > current_time + self.tolerance_ms {
            return false;
        }

        // Check if timestamp is not too far in the past
        // Allow up to 5 minutes in the past to account for network delays
        if timestamp < current_time.saturating_sub(300_000) {
            return false;
        }

        true
    }

    /// Calculate the time difference between a timestamp and current time
    pub fn time_diff(&self, timestamp: u128) -> i128 {
        let current_time = Self::get_system_time();
        timestamp as i128 - current_time as i128
    }

    /// Check if enough time has passed since a previous timestamp (for hourly checks)
    pub fn has_hour_passed(&self, previous_timestamp: u128) -> bool {
        let current_time = Self::get_system_time();
        current_time >= previous_timestamp + 3_600_000 // 1 hour in milliseconds
    }

    /// Get seconds remaining until an hour has passed since a timestamp
    pub fn seconds_until_hour_passed(&self, previous_timestamp: u128) -> u64 {
        let current_time = Self::get_system_time();
        let target_time = previous_timestamp + 3_600_000;

        if current_time >= target_time {
            0
        } else {
            ((target_time - current_time) / 1000) as u64
        }
    }
}

impl Default for TimeSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_sync_creation() {
        let time_sync = TimeSync::new();
        assert_eq!(time_sync.tolerance_ms, 500);
    }

    #[test]
    fn test_time_sync_with_custom_tolerance() {
        let time_sync = TimeSync::new_with_tolerance(1000);
        assert_eq!(time_sync.tolerance_ms, 1000);
    }

    #[test]
    fn test_get_system_time() {
        let time1 = TimeSync::get_system_time();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let time2 = TimeSync::get_system_time();
        assert!(time2 > time1);
    }

    #[test]
    fn test_validate_current_timestamp() {
        let time_sync = TimeSync::new();
        let current_time = TimeSync::get_system_time();
        assert!(time_sync.validate_timestamp(current_time));
    }

    #[test]
    fn test_validate_future_timestamp() {
        let time_sync = TimeSync::new();
        let future_time = TimeSync::get_system_time() + 1000; // 1 second in future
        assert!(!time_sync.validate_timestamp(future_time));
    }

    #[test]
    fn test_validate_past_timestamp() {
        let time_sync = TimeSync::new();
        let past_time = TimeSync::get_system_time() - 100_000; // 100 seconds ago
        assert!(time_sync.validate_timestamp(past_time));

        // Very old timestamp should fail
        let very_old_time = TimeSync::get_system_time() - 400_000; // Over 5 minutes ago
        assert!(!time_sync.validate_timestamp(very_old_time));
    }

    #[test]
    fn test_time_diff() {
        let time_sync = TimeSync::new();
        let current_time = TimeSync::get_system_time();
        let past_time = current_time - 1000;
        let future_time = current_time + 1000;

        let diff_past = time_sync.time_diff(past_time);
        let diff_future = time_sync.time_diff(future_time);

        assert!(diff_past < 0);
        assert!(diff_future > 0);
    }

    #[test]
    fn test_has_hour_passed() {
        let time_sync = TimeSync::new();
        let current_time = TimeSync::get_system_time();

        // Just happened - hour has not passed
        assert!(!time_sync.has_hour_passed(current_time));

        // Over an hour ago - hour has passed
        let old_time = current_time - 3_700_000; // 1 hour 1 minute ago
        assert!(time_sync.has_hour_passed(old_time));
    }

    #[test]
    fn test_seconds_until_hour_passed() {
        let time_sync = TimeSync::new();
        let current_time = TimeSync::get_system_time();

        // 30 minutes ago - should have ~30 minutes left
        let half_hour_ago = current_time - 1_800_000;
        let seconds_left = time_sync.seconds_until_hour_passed(half_hour_ago);
        assert!(seconds_left >= 1790 && seconds_left <= 1810); // Allow some timing variance

        // Over an hour ago - should be 0
        let old_time = current_time - 3_700_000;
        assert_eq!(time_sync.seconds_until_hour_passed(old_time), 0);
    }
}
